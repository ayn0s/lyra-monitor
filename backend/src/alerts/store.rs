use super::model::{generate_id, AlertRule, AlertsConfig, WebhookTarget};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Clone)]
pub struct AlertsStore {
    config: Arc<RwLock<AlertsConfig>>,
    path: PathBuf,
}

impl AlertsStore {
    pub async fn load(path: PathBuf) -> Self {
        let config = read_config(&path).unwrap_or_else(|e| {
            warn!(error = %e, path = %path.display(), "failed to load alerts config, starting empty");
            AlertsConfig::default()
        });

        Self {
            config: Arc::new(RwLock::new(config)),
            path,
        }
    }

    pub async fn snapshot(&self) -> AlertsConfig {
        self.config.read().await.clone()
    }

    pub async fn list_webhooks(&self) -> Vec<WebhookTarget> {
        self.config.read().await.webhooks.clone()
    }

    pub async fn get_webhook(&self, id: &str) -> Option<WebhookTarget> {
        self.config
            .read()
            .await
            .webhooks
            .iter()
            .find(|w| w.id == id)
            .cloned()
    }

    pub async fn create_webhook(&self, mut webhook: WebhookTarget) -> Result<WebhookTarget> {
        webhook.id = generate_id();
        let mut guard = self.config.write().await;
        guard.webhooks.push(webhook.clone());
        write_config(&self.path, &guard)?;
        Ok(webhook)
    }

    pub async fn update_webhook(
        &self,
        id: &str,
        updated: WebhookTarget,
    ) -> Result<Option<WebhookTarget>> {
        let mut guard = self.config.write().await;
        let Some(existing) = guard.webhooks.iter_mut().find(|w| w.id == id) else {
            return Ok(None);
        };
        existing.name = updated.name;
        existing.kind = updated.kind;
        existing.url = updated.url;
        existing.enabled = updated.enabled;
        let result = existing.clone();
        write_config(&self.path, &guard)?;
        Ok(Some(result))
    }

    pub async fn delete_webhook(&self, id: &str) -> Result<bool> {
        let mut guard = self.config.write().await;
        let before = guard.webhooks.len();
        guard.webhooks.retain(|w| w.id != id);
        for rule in guard.rules.iter_mut() {
            rule.webhook_ids.retain(|w| w != id);
        }
        let removed = guard.webhooks.len() != before;
        if removed {
            write_config(&self.path, &guard)?;
        }
        Ok(removed)
    }

    pub async fn list_rules(&self) -> Vec<AlertRule> {
        self.config.read().await.rules.clone()
    }

    pub async fn create_rule(&self, mut rule: AlertRule) -> Result<AlertRule> {
        rule.id = generate_id();
        let mut guard = self.config.write().await;
        guard.rules.push(rule.clone());
        write_config(&self.path, &guard)?;
        Ok(rule)
    }

    pub async fn update_rule(&self, id: &str, updated: AlertRule) -> Result<Option<AlertRule>> {
        let mut guard = self.config.write().await;
        let Some(existing) = guard.rules.iter_mut().find(|r| r.id == id) else {
            return Ok(None);
        };
        existing.name = updated.name;
        existing.agent_addr = updated.agent_addr;
        existing.condition = updated.condition;
        existing.cooldown_seconds = updated.cooldown_seconds;
        existing.webhook_ids = updated.webhook_ids;
        existing.enabled = updated.enabled;
        let result = existing.clone();
        write_config(&self.path, &guard)?;
        Ok(Some(result))
    }

    pub async fn delete_rule(&self, id: &str) -> Result<bool> {
        let mut guard = self.config.write().await;
        let before = guard.rules.len();
        guard.rules.retain(|r| r.id != id);
        let removed = guard.rules.len() != before;
        if removed {
            write_config(&self.path, &guard)?;
        }
        Ok(removed)
    }
}

fn read_config(path: &Path) -> Result<AlertsConfig> {
    let content = std::fs::read_to_string(path).context("reading alerts config")?;
    serde_json::from_str(&content).context("parsing alerts config")
}

fn write_config(path: &Path, config: &AlertsConfig) -> Result<()> {
    let content = serde_json::to_string_pretty(config).context("serializing alerts config")?;
    std::fs::write(path, content).context("writing alerts config")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::model::{AlertCondition, WebhookKind};

    fn temp_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("lyra-alerts-test-{}.json", generate_id()));
        p
    }

    fn sample_webhook() -> WebhookTarget {
        WebhookTarget {
            id: String::new(),
            name: "test".to_string(),
            kind: WebhookKind::Custom,
            url: "http://localhost/hook".to_string(),
            enabled: true,
        }
    }

    #[tokio::test]
    async fn create_webhook_assigns_id_and_persists() {
        let path = temp_path();
        let store = AlertsStore::load(path.clone()).await;
        let created = store.create_webhook(sample_webhook()).await.unwrap();
        assert!(!created.id.is_empty());

        let reloaded = AlertsStore::load(path.clone()).await;
        assert_eq!(reloaded.list_webhooks().await.len(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn update_webhook_returns_none_when_missing() {
        let store = AlertsStore::load(temp_path()).await;
        let result = store
            .update_webhook("missing", sample_webhook())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn deleting_a_webhook_detaches_it_from_rules() {
        let path = temp_path();
        let store = AlertsStore::load(path.clone()).await;
        let webhook = store.create_webhook(sample_webhook()).await.unwrap();

        let rule = AlertRule {
            id: String::new(),
            name: "cpu".to_string(),
            agent_addr: None,
            condition: AlertCondition::CpuAbove { percent: 90.0 },
            cooldown_seconds: 60,
            webhook_ids: vec![webhook.id.clone()],
            enabled: true,
        };
        store.create_rule(rule).await.unwrap();

        let removed = store.delete_webhook(&webhook.id).await.unwrap();
        assert!(removed);

        let rules = store.list_rules().await;
        assert!(rules[0].webhook_ids.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn load_with_missing_file_starts_empty() {
        let store = AlertsStore::load(temp_path()).await;
        assert!(store.list_webhooks().await.is_empty());
        assert!(store.list_rules().await.is_empty());
    }
}
