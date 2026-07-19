use super::model::{WebhookKind, WebhookTarget};
use anyhow::{bail, Result};

pub struct AlertEvent {
    pub rule_name: String,
    pub agent_addr: Option<String>,
    pub message: String,
    pub triggered_at_unix_ms: i64,
}

pub fn build_payload(kind: WebhookKind, event: &AlertEvent) -> serde_json::Value {
    match kind {
        WebhookKind::Discord => build_discord_payload(event),
        WebhookKind::Custom => build_custom_payload(event),
    }
}

fn build_discord_payload(event: &AlertEvent) -> serde_json::Value {
    serde_json::json!({
        "content": event.message,
    })
}

fn build_custom_payload(event: &AlertEvent) -> serde_json::Value {
    serde_json::json!({
        "rule": event.rule_name,
        "agent": event.agent_addr,
        "message": event.message,
        "triggered_at_unix_ms": event.triggered_at_unix_ms,
    })
}

pub async fn send(
    client: &reqwest::Client,
    target: &WebhookTarget,
    event: &AlertEvent,
) -> Result<()> {
    let payload = build_payload(target.kind, event);
    let response = client.post(&target.url).json(&payload).send().await?;

    if !response.status().is_success() {
        bail!("webhook responded with status {}", response.status());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> AlertEvent {
        AlertEvent {
            rule_name: "High CPU".to_string(),
            agent_addr: Some("172.22.150.252:50051".to_string()),
            message: "CPU at 95.0% (threshold 90%)".to_string(),
            triggered_at_unix_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn discord_payload_uses_content_field() {
        let payload = build_discord_payload(&sample_event());
        assert_eq!(payload["content"], "CPU at 95.0% (threshold 90%)");
    }

    #[test]
    fn custom_payload_includes_rule_and_agent_metadata() {
        let payload = build_custom_payload(&sample_event());
        assert_eq!(payload["rule"], "High CPU");
        assert_eq!(payload["agent"], "172.22.150.252:50051");
        assert_eq!(payload["message"], "CPU at 95.0% (threshold 90%)");
        assert_eq!(payload["triggered_at_unix_ms"], 1_700_000_000_000i64);
    }

    #[test]
    fn build_payload_dispatches_on_kind() {
        let event = sample_event();
        assert!(build_payload(WebhookKind::Discord, &event)
            .get("content")
            .is_some());
        assert!(build_payload(WebhookKind::Custom, &event)
            .get("rule")
            .is_some());
    }
}
