mod alerts;
mod discovery;
mod grpc_client;
#[cfg(feature = "embedded-ui")]
mod static_files;
mod ws_bridge;

use alerts::{AlertEvent, AlertRule, AlertsStore, WebhookTarget};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use discovery::DiscoveredAgents;
use serde::Serialize;
use shared::pb::{GetMetricsHistoryRequest, ListServicesRequest, MetricsRequest, PingRequest};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    agents: DiscoveredAgents,
    alerts: AlertsStore,
}

fn data_dir() -> PathBuf {
    std::env::var("STATE_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[derive(Serialize)]
struct PingResult {
    nonce: String,
    server_time_unix_ms: i64,
    roundtrip_ms: u128,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (agents, _mdns_daemon) = discovery::start()?;
    info!("mDNS agent discovery started");

    let alerts_config_path = data_dir().join("alerts.json");
    let alerts_store = AlertsStore::load(alerts_config_path).await;
    alerts::spawn_evaluator(alerts_store.clone(), agents.clone());
    info!("alert evaluator started");

    let state = AppState {
        agents,
        alerts: alerts_store,
    };

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/agents", get(list_agents))
        .route("/api/agents/:addr/ping", get(ping_agent))
        .route("/api/agents/:addr/metrics", get(metrics_agent))
        .route(
            "/api/agents/:addr/metrics/history",
            get(metrics_history_agent),
        )
        .route("/api/agents/:addr/services", get(services_agent))
        .route("/ws/terminal/:addr", get(terminal_ws))
        .route("/api/webhooks", get(list_webhooks).post(create_webhook))
        .route(
            "/api/webhooks/:id",
            axum::routing::put(update_webhook).delete(delete_webhook),
        )
        .route("/api/webhooks/:id/test", post(test_webhook))
        .route(
            "/api/alert-rules",
            get(list_alert_rules).post(create_alert_rule),
        )
        .route(
            "/api/alert-rules/:id",
            axum::routing::put(update_alert_rule).delete(delete_alert_rule),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    #[cfg(feature = "embedded-ui")]
    let app = app.fallback(static_files::serve);

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    info!(%addr, "starting the Axum backend");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn list_agents(State(state): State<AppState>) -> Json<serde_json::Value> {
    let agents = state.agents.read().await;
    Json(serde_json::json!(*agents))
}

async fn resolve_agent_uri(state: &AppState, addr: &str) -> Result<String, ApiError> {
    if addr != "default" {
        return Ok(format!("http://{addr}"));
    }

    state
        .agents
        .read()
        .await
        .values()
        .next()
        .cloned()
        .ok_or_else(|| ApiError(anyhow::anyhow!("no agent discovered via mDNS yet")))
}

async fn ping_agent(
    State(state): State<AppState>,
    Path(addr): Path<String>,
) -> Result<Json<PingResult>, ApiError> {
    let uri = resolve_agent_uri(&state, &addr).await?;
    let mut client = grpc_client::connect(&uri).await.map_err(ApiError)?;

    let started = std::time::Instant::now();
    let nonce = format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );

    let response = client
        .ping(PingRequest {
            nonce: nonce.clone(),
        })
        .await
        .map_err(|e| ApiError(anyhow::anyhow!("gRPC Ping failed: {e}")))?
        .into_inner();

    Ok(Json(PingResult {
        nonce: response.nonce,
        server_time_unix_ms: response.server_time_unix_ms,
        roundtrip_ms: started.elapsed().as_millis(),
    }))
}

async fn metrics_agent(
    State(state): State<AppState>,
    Path(addr): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let uri = resolve_agent_uri(&state, &addr).await?;
    let mut client = grpc_client::connect(&uri).await.map_err(ApiError)?;

    let response = client
        .get_metrics(MetricsRequest {})
        .await
        .map_err(|e| ApiError(anyhow::anyhow!("gRPC GetMetrics failed: {e}")))?
        .into_inner();

    Ok(Json(serde_json::json!({
        "cpu_usage_percent": response.cpu_usage_percent,
        "mem_used_bytes": response.mem_used_bytes,
        "mem_total_bytes": response.mem_total_bytes,
        "load_average_1m": response.load_average_1m,
        "uptime_seconds": response.uptime_seconds,
        "network_rx_bytes_per_sec": response.network_rx_bytes_per_sec,
        "network_tx_bytes_per_sec": response.network_tx_bytes_per_sec,
    })))
}

async fn metrics_history_agent(
    State(state): State<AppState>,
    Path(addr): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let uri = resolve_agent_uri(&state, &addr).await?;
    let mut client = grpc_client::connect(&uri).await.map_err(ApiError)?;

    let response = client
        .get_metrics_history(GetMetricsHistoryRequest {})
        .await
        .map_err(|e| ApiError(anyhow::anyhow!("gRPC GetMetricsHistory failed: {e}")))?
        .into_inner();

    let samples: Vec<serde_json::Value> = response
        .samples
        .into_iter()
        .map(|sample| {
            serde_json::json!({
                "timestamp_unix_ms": sample.timestamp_unix_ms,
                "cpu_usage_percent": sample.cpu_usage_percent,
                "mem_used_bytes": sample.mem_used_bytes,
                "mem_total_bytes": sample.mem_total_bytes,
                "load_average_1m": sample.load_average_1m,
                "network_rx_bytes_per_sec": sample.network_rx_bytes_per_sec,
                "network_tx_bytes_per_sec": sample.network_tx_bytes_per_sec,
            })
        })
        .collect();

    Ok(Json(serde_json::json!(samples)))
}

async fn services_agent(
    State(state): State<AppState>,
    Path(addr): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let uri = resolve_agent_uri(&state, &addr).await?;
    let mut client = grpc_client::connect(&uri).await.map_err(ApiError)?;

    let response = client
        .list_services(ListServicesRequest {})
        .await
        .map_err(|e| ApiError(anyhow::anyhow!("gRPC ListServices failed: {e}")))?
        .into_inner();

    let services: Vec<serde_json::Value> = response
        .services
        .into_iter()
        .map(|unit| {
            serde_json::json!({
                "name": unit.name,
                "description": unit.description,
                "load_state": unit.load_state,
                "active_state": unit.active_state,
                "sub_state": unit.sub_state,
            })
        })
        .collect();

    Ok(Json(serde_json::json!(services)))
}

async fn terminal_ws(
    State(state): State<AppState>,
    Path(addr): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let uri = resolve_agent_uri(&state, &addr).await?;
    let client = grpc_client::connect(&uri).await.map_err(ApiError)?;

    Ok(ws.on_upgrade(move |socket| ws_bridge::bridge(socket, client)))
}

async fn list_webhooks(State(state): State<AppState>) -> Json<Vec<WebhookTarget>> {
    Json(state.alerts.list_webhooks().await)
}

async fn create_webhook(
    State(state): State<AppState>,
    Json(webhook): Json<WebhookTarget>,
) -> Result<Json<WebhookTarget>, ApiError> {
    let created = state
        .alerts
        .create_webhook(webhook)
        .await
        .map_err(ApiError)?;
    Ok(Json(created))
}

async fn update_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(webhook): Json<WebhookTarget>,
) -> Result<Json<WebhookTarget>, ApiError> {
    state
        .alerts
        .update_webhook(&id, webhook)
        .await
        .map_err(ApiError)?
        .map(Json)
        .ok_or_else(|| ApiError(anyhow::anyhow!("webhook not found")))
}

async fn delete_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let removed = state.alerts.delete_webhook(&id).await.map_err(ApiError)?;
    if !removed {
        return Err(ApiError(anyhow::anyhow!("webhook not found")));
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn test_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let webhook = state
        .alerts
        .get_webhook(&id)
        .await
        .ok_or_else(|| ApiError(anyhow::anyhow!("webhook not found")))?;

    let event = AlertEvent {
        rule_name: "Test alert".to_string(),
        agent_addr: None,
        message: "This is a test notification from Lyra Monitor.".to_string(),
        triggered_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64,
    };

    let client = reqwest::Client::new();
    alerts::send_webhook(&client, &webhook, &event)
        .await
        .map_err(ApiError)?;

    Ok(Json(serde_json::json!({ "sent": true })))
}

async fn list_alert_rules(State(state): State<AppState>) -> Json<Vec<AlertRule>> {
    Json(state.alerts.list_rules().await)
}

fn validate_rule(rule: &AlertRule) -> Result<(), ApiError> {
    if rule.condition.requires_specific_agent() && rule.agent_addr.is_none() {
        return Err(ApiError(anyhow::anyhow!(
            "this condition requires a specific agent to be selected"
        )));
    }
    Ok(())
}

async fn create_alert_rule(
    State(state): State<AppState>,
    Json(rule): Json<AlertRule>,
) -> Result<Json<AlertRule>, ApiError> {
    validate_rule(&rule)?;
    let created = state.alerts.create_rule(rule).await.map_err(ApiError)?;
    Ok(Json(created))
}

async fn update_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(rule): Json<AlertRule>,
) -> Result<Json<AlertRule>, ApiError> {
    validate_rule(&rule)?;
    state
        .alerts
        .update_rule(&id, rule)
        .await
        .map_err(ApiError)?
        .map(Json)
        .ok_or_else(|| ApiError(anyhow::anyhow!("alert rule not found")))
}

async fn delete_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let removed = state.alerts.delete_rule(&id).await.map_err(ApiError)?;
    if !removed {
        return Err(ApiError(anyhow::anyhow!("alert rule not found")));
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

#[derive(Debug)]
struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            axum::http::StatusCode::BAD_GATEWAY,
            format!("agent error: {}", self.0),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    async fn state_with_agents(entries: &[(&str, &str)]) -> AppState {
        let map: HashMap<String, String> = entries
            .iter()
            .map(|(name, uri)| (name.to_string(), uri.to_string()))
            .collect();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("lyra-alerts-main-test-{nanos}.json"));
        AppState {
            agents: Arc::new(RwLock::new(map)),
            alerts: AlertsStore::load(path).await,
        }
    }

    #[tokio::test]
    async fn resolves_explicit_addr_without_consulting_discovery() {
        let state = state_with_agents(&[]).await;
        let uri = resolve_agent_uri(&state, "192.168.1.42:50051")
            .await
            .unwrap();
        assert_eq!(uri, "http://192.168.1.42:50051");
    }

    #[tokio::test]
    async fn resolves_default_to_the_only_discovered_agent() {
        let state =
            state_with_agents(&[("agent1._lyra-agent._tcp.local.", "http://10.0.0.5:50051")]).await;
        let uri = resolve_agent_uri(&state, "default").await.unwrap();
        assert_eq!(uri, "http://10.0.0.5:50051");
    }

    #[tokio::test]
    async fn default_fails_when_no_agent_discovered() {
        let state = state_with_agents(&[]).await;
        let result = resolve_agent_uri(&state, "default").await;
        assert!(result.is_err());
    }
}
