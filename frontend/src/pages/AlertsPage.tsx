import AlertRuleSection from "../components/alerts/AlertRuleSection";
import WebhookSection from "../components/alerts/WebhookSection";
import { useAgents } from "../hooks/useAgents";
import { useAlertRules } from "../hooks/useAlertRules";
import { useWebhooks } from "../hooks/useWebhooks";
import styles from "./AlertsPage.module.css";

export default function AlertsPage() {
  const { agents } = useAgents();
  const { webhooks, loading: webhooksLoading, error: webhooksError, refresh: refreshWebhooks } =
    useWebhooks();
  const { rules, loading: rulesLoading, error: rulesError, refresh: refreshRules } = useAlertRules();

  return (
    <div className={styles.page}>
      <WebhookSection
        webhooks={webhooks}
        loading={webhooksLoading}
        error={webhooksError}
        onChange={refreshWebhooks}
      />
      <AlertRuleSection
        rules={rules}
        loading={rulesLoading}
        error={rulesError}
        agents={agents}
        webhooks={webhooks}
        onChange={refreshRules}
      />
    </div>
  );
}
