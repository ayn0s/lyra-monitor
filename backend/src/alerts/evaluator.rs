use super::model::{AlertCondition, AlertRule};
use super::store::AlertsStore;
use super::webhook::{self, AlertEvent};
use crate::discovery::DiscoveredAgents;
use crate::grpc_client;
use shared::pb::{ListServicesRequest, MetricsRequest, MetricsResponse, ServiceUnit};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::warn;

const EVAL_INTERVAL: Duration = Duration::from_secs(15);

pub fn spawn(store: AlertsStore, agents: DiscoveredAgents) {
    tokio::spawn(async move {
        let http = reqwest::Client::new();
        let mut last_fired: HashMap<(String, String), Instant> = HashMap::new();
        let mut interval = tokio::time::interval(EVAL_INTERVAL);

        loop {
            interval.tick().await;
            run_once(&store, &agents, &http, &mut last_fired).await;
        }
    });
}

async fn run_once(
    store: &AlertsStore,
    agents: &DiscoveredAgents,
    http: &reqwest::Client,
    last_fired: &mut HashMap<(String, String), Instant>,
) {
    let config = store.snapshot().await;
    let known_addrs: HashSet<String> = agents
        .read()
        .await
        .values()
        .map(|uri| uri.trim_start_matches("http://").to_string())
        .collect();

    for rule in config.rules.iter().filter(|r| r.enabled) {
        if rule.condition.requires_specific_agent() && rule.agent_addr.is_none() {
            continue;
        }

        let targets: Vec<String> = match &rule.agent_addr {
            Some(addr) => vec![addr.clone()],
            None => known_addrs.iter().cloned().collect(),
        };

        for addr in targets {
            let breach = evaluate_rule(rule, &addr, &known_addrs).await;
            let key = (rule.id.clone(), addr.clone());

            match breach {
                Some(detail) => {
                    let should_fire = last_fired
                        .get(&key)
                        .map(|fired_at| {
                            fired_at.elapsed() >= Duration::from_secs(rule.cooldown_seconds)
                        })
                        .unwrap_or(true);

                    if should_fire {
                        dispatch(rule, &addr, &detail, &config.webhooks, http).await;
                        last_fired.insert(key, Instant::now());
                    }
                }
                None => {
                    last_fired.remove(&key);
                }
            }
        }
    }
}

async fn dispatch(
    rule: &AlertRule,
    addr: &str,
    detail: &str,
    webhooks: &[super::model::WebhookTarget],
    http: &reqwest::Client,
) {
    if rule.webhook_ids.is_empty() {
        warn!(rule = %rule.name, "alert condition breached but rule has no webhook attached, nothing sent");
        return;
    }

    let event = AlertEvent {
        rule_name: rule.name.clone(),
        agent_addr: Some(addr.to_string()),
        message: format!("[{}] {addr} — {detail}", rule.name),
        triggered_at_unix_ms: now_ms(),
    };

    let mut sent = false;
    for webhook_id in &rule.webhook_ids {
        let Some(target) = webhooks.iter().find(|w| &w.id == webhook_id && w.enabled) else {
            continue;
        };

        sent = true;
        if let Err(e) = webhook::send(http, target, &event).await {
            warn!(error = %e, webhook = %target.name, rule = %rule.name, "failed to send alert webhook");
        }
    }

    if !sent {
        warn!(rule = %rule.name, "alert condition breached but all attached webhooks are missing or disabled, nothing sent");
    }
}

async fn evaluate_rule(
    rule: &AlertRule,
    addr: &str,
    known_addrs: &HashSet<String>,
) -> Option<String> {
    match &rule.condition {
        AlertCondition::AgentOffline => {
            if known_addrs.contains(addr) {
                None
            } else {
                Some(format!("agent {addr} is offline (not seen via mDNS)"))
            }
        }
        AlertCondition::ServiceNotActive { service_name } => {
            let mut client = grpc_client::connect(&format!("http://{addr}")).await.ok()?;
            let services = client
                .list_services(ListServicesRequest {})
                .await
                .ok()?
                .into_inner()
                .services;
            service_breach(service_name, &services)
        }
        condition => {
            let mut client = grpc_client::connect(&format!("http://{addr}")).await.ok()?;
            let metrics = client
                .get_metrics(MetricsRequest {})
                .await
                .ok()?
                .into_inner();
            evaluate_metric_condition(condition, &metrics)
        }
    }
}

fn evaluate_metric_condition(
    condition: &AlertCondition,
    metrics: &MetricsResponse,
) -> Option<String> {
    match condition {
        AlertCondition::CpuAbove { percent } => cpu_breach(*percent, metrics.cpu_usage_percent),
        AlertCondition::MemoryAbove { percent } => {
            memory_breach(*percent, metrics.mem_used_bytes, metrics.mem_total_bytes)
        }
        AlertCondition::LoadAbove { value } => load_breach(*value, metrics.load_average_1m),
        AlertCondition::NetworkRxAbove { bytes_per_sec } => {
            network_breach("download", *bytes_per_sec, metrics.network_rx_bytes_per_sec)
        }
        AlertCondition::NetworkTxAbove { bytes_per_sec } => {
            network_breach("upload", *bytes_per_sec, metrics.network_tx_bytes_per_sec)
        }
        AlertCondition::AgentOffline | AlertCondition::ServiceNotActive { .. } => None,
    }
}

fn cpu_breach(threshold: f64, actual: f64) -> Option<String> {
    (actual > threshold).then(|| format!("CPU usage at {actual:.1}% (threshold {threshold:.1}%)"))
}

fn memory_breach(threshold_percent: f64, used: u64, total: u64) -> Option<String> {
    if total == 0 {
        return None;
    }
    let percent = (used as f64 / total as f64) * 100.0;
    (percent > threshold_percent)
        .then(|| format!("memory usage at {percent:.1}% (threshold {threshold_percent:.1}%)"))
}

fn load_breach(threshold: f64, actual: f64) -> Option<String> {
    (actual > threshold)
        .then(|| format!("load average (1m) at {actual:.2} (threshold {threshold:.2})"))
}

fn network_breach(label: &str, threshold: u64, actual: u64) -> Option<String> {
    (actual > threshold)
        .then(|| format!("{label} rate at {actual} B/s (threshold {threshold} B/s)"))
}

fn service_breach(service_name: &str, services: &[ServiceUnit]) -> Option<String> {
    match services.iter().find(|s| s.name == service_name) {
        Some(unit) if unit.active_state != "active" => Some(format!(
            "service {service_name} is {} (expected active)",
            unit.active_state
        )),
        None => Some(format!("service {service_name} not found on agent")),
        _ => None,
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_breach_fires_above_threshold() {
        assert!(cpu_breach(90.0, 95.0).is_some());
        assert!(cpu_breach(90.0, 90.0).is_none());
        assert!(cpu_breach(90.0, 80.0).is_none());
    }

    #[test]
    fn memory_breach_computes_percentage() {
        let msg = memory_breach(50.0, 800, 1000).unwrap();
        assert!(msg.contains("80.0%"));
    }

    #[test]
    fn memory_breach_avoids_division_by_zero() {
        assert!(memory_breach(50.0, 0, 0).is_none());
    }

    #[test]
    fn load_breach_fires_above_threshold() {
        assert!(load_breach(2.0, 3.5).is_some());
        assert!(load_breach(2.0, 1.0).is_none());
    }

    #[test]
    fn network_breach_fires_above_threshold() {
        assert!(network_breach("download", 1000, 2000).is_some());
        assert!(network_breach("download", 1000, 500).is_none());
    }

    #[test]
    fn service_breach_fires_when_not_active() {
        let services = vec![ServiceUnit {
            name: "nginx.service".to_string(),
            description: "".to_string(),
            load_state: "loaded".to_string(),
            active_state: "failed".to_string(),
            sub_state: "".to_string(),
        }];
        let msg = service_breach("nginx.service", &services).unwrap();
        assert!(msg.contains("failed"));
    }

    #[test]
    fn service_breach_is_none_when_active() {
        let services = vec![ServiceUnit {
            name: "nginx.service".to_string(),
            description: "".to_string(),
            load_state: "loaded".to_string(),
            active_state: "active".to_string(),
            sub_state: "running".to_string(),
        }];
        assert!(service_breach("nginx.service", &services).is_none());
    }

    #[test]
    fn service_breach_fires_when_service_missing() {
        let msg = service_breach("missing.service", &[]).unwrap();
        assert!(msg.contains("not found"));
    }

    #[test]
    fn evaluate_metric_condition_dispatches_by_variant() {
        let metrics = MetricsResponse {
            cpu_usage_percent: 95.0,
            mem_used_bytes: 900,
            mem_total_bytes: 1000,
            load_average_1m: 5.0,
            uptime_seconds: 0,
            network_rx_bytes_per_sec: 2000,
            network_tx_bytes_per_sec: 2000,
        };

        assert!(
            evaluate_metric_condition(&AlertCondition::CpuAbove { percent: 90.0 }, &metrics)
                .is_some()
        );
        assert!(evaluate_metric_condition(
            &AlertCondition::MemoryAbove { percent: 50.0 },
            &metrics
        )
        .is_some());
        assert!(
            evaluate_metric_condition(&AlertCondition::LoadAbove { value: 1.0 }, &metrics)
                .is_some()
        );
        assert!(evaluate_metric_condition(
            &AlertCondition::NetworkRxAbove {
                bytes_per_sec: 1000
            },
            &metrics
        )
        .is_some());
        assert!(evaluate_metric_condition(
            &AlertCondition::NetworkTxAbove {
                bytes_per_sec: 1000
            },
            &metrics
        )
        .is_some());
        assert!(evaluate_metric_condition(&AlertCondition::AgentOffline, &metrics).is_none());
    }
}
