use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookKind {
    Discord,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTarget {
    pub id: String,
    pub name: String,
    pub kind: WebhookKind,
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlertCondition {
    CpuAbove { percent: f64 },
    MemoryAbove { percent: f64 },
    LoadAbove { value: f64 },
    NetworkRxAbove { bytes_per_sec: u64 },
    NetworkTxAbove { bytes_per_sec: u64 },
    AgentOffline,
    ServiceNotActive { service_name: String },
}

impl AlertCondition {
    pub fn requires_specific_agent(&self) -> bool {
        matches!(
            self,
            AlertCondition::AgentOffline | AlertCondition::ServiceNotActive { .. }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub agent_addr: Option<String>,
    pub condition: AlertCondition,
    pub cooldown_seconds: u64,
    pub webhook_ids: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertsConfig {
    #[serde(default)]
    pub webhooks: Vec<WebhookTarget>,
    #[serde(default)]
    pub rules: Vec<AlertRule>,
}

pub fn generate_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{nanos:x}-{count:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_are_unique() {
        let a = generate_id();
        let b = generate_id();
        assert_ne!(a, b);
    }

    #[test]
    fn agent_offline_and_service_conditions_require_a_specific_agent() {
        assert!(AlertCondition::AgentOffline.requires_specific_agent());
        assert!(AlertCondition::ServiceNotActive {
            service_name: "nginx.service".to_string()
        }
        .requires_specific_agent());
        assert!(!AlertCondition::CpuAbove { percent: 90.0 }.requires_specific_agent());
    }
}
