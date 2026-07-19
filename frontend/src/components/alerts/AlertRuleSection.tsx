import { Pause, Play } from "lucide-react";
import { useState } from "react";
import { createAlertRule, deleteAlertRule, updateAlertRule } from "../../api/client";
import type { AgentInfo, AlertCondition, AlertRule, WebhookTarget } from "../../api/types";
import {
  CONDITION_LABELS,
  CONDITION_TYPES,
  defaultConditionFor,
  describeCondition,
  requiresSpecificAgent,
} from "../../lib/alertConditions";
import styles from "./AlertRuleSection.module.css";

interface AlertRuleSectionProps {
  rules: AlertRule[];
  loading: boolean;
  error: string | null;
  agents: AgentInfo[];
  webhooks: WebhookTarget[];
  onChange: () => void;
}

interface FormState {
  name: string;
  agentAddr: string;
  condition: AlertCondition;
  cooldownSeconds: number;
  webhookIds: string[];
  enabled: boolean;
}

const EMPTY_FORM: FormState = {
  name: "",
  agentAddr: "",
  condition: defaultConditionFor("cpu_above"),
  cooldownSeconds: 300,
  webhookIds: [],
  enabled: true,
};

function conditionField(condition: AlertCondition, value: string): AlertCondition {
  switch (condition.type) {
    case "cpu_above":
    case "memory_above":
      return { ...condition, percent: Number(value) };
    case "load_above":
      return { ...condition, value: Number(value) };
    case "network_rx_above":
    case "network_tx_above":
      return { ...condition, bytes_per_sec: Number(value) };
    case "service_not_active":
      return { ...condition, service_name: value };
    case "agent_offline":
      return condition;
  }
}

export default function AlertRuleSection({
  rules,
  loading,
  error,
  agents,
  webhooks,
  onChange,
}: AlertRuleSectionProps) {
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  const forcedAgent = requiresSpecificAgent(form.condition.type);

  const startEdit = (rule: AlertRule) => {
    setEditingId(rule.id);
    setForm({
      name: rule.name,
      agentAddr: rule.agent_addr ?? "",
      condition: rule.condition,
      cooldownSeconds: rule.cooldown_seconds,
      webhookIds: rule.webhook_ids,
      enabled: rule.enabled,
    });
    setFormError(null);
  };

  const cancelEdit = () => {
    setEditingId(null);
    setForm(EMPTY_FORM);
    setFormError(null);
  };

  const handleConditionTypeChange = (type: AlertCondition["type"]) => {
    setForm((prev) => ({ ...prev, condition: defaultConditionFor(type) }));
  };

  const toggleWebhook = (id: string) => {
    setForm((prev) => ({
      ...prev,
      webhookIds: prev.webhookIds.includes(id)
        ? prev.webhookIds.filter((w) => w !== id)
        : [...prev.webhookIds, id],
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim()) return;
    if (forcedAgent && !form.agentAddr) {
      setFormError("This condition requires a specific agent to be selected.");
      return;
    }
    if (form.webhookIds.length === 0) {
      setFormError("Select at least one webhook, otherwise this rule will never notify anyone.");
      return;
    }

    setSubmitting(true);
    setFormError(null);
    try {
      const payload = {
        name: form.name,
        agent_addr: form.agentAddr || null,
        condition: form.condition,
        cooldown_seconds: form.cooldownSeconds,
        webhook_ids: form.webhookIds,
        enabled: form.enabled,
      };
      if (editingId) {
        await updateAlertRule(editingId, { id: editingId, ...payload });
      } else {
        await createAlertRule(payload);
      }
      setEditingId(null);
      setForm(EMPTY_FORM);
      onChange();
    } catch (err) {
      setFormError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  const handleToggle = async (rule: AlertRule) => {
    await updateAlertRule(rule.id, { ...rule, enabled: !rule.enabled });
    onChange();
  };

  const handleDelete = async (id: string) => {
    await deleteAlertRule(id);
    onChange();
  };

  return (
    <section className={styles.section}>
      <h2 className={styles.title}>Alert rules</h2>
      <p className={styles.hint}>Conditions that trigger a webhook notification when breached.</p>

      {error && <div className={styles.errorBanner}>{error}</div>}

      {!loading && rules.length === 0 && !error && (
        <p className={styles.hint}>No alert rule configured yet.</p>
      )}

      {rules.length > 0 && (
        <ul className={styles.list}>
          {rules.map((rule) => (
            <li key={rule.id} className={rule.enabled ? styles.item : styles.itemPaused}>
              <div className={styles.itemMain}>
                <span className={styles.itemName}>
                  {rule.name}
                  {!rule.enabled && <span className={styles.pausedBadge}>paused</span>}
                </span>
                <span className={styles.itemDetail}>{describeCondition(rule.condition)}</span>
                <span className={styles.itemDetail}>
                  {rule.agent_addr ?? "all agents"} · cooldown {rule.cooldown_seconds}s
                </span>
                {rule.webhook_ids.length === 0 && (
                  <span className={styles.warning}>
                    No webhook attached — this rule will never notify anyone.
                  </span>
                )}
              </div>
              <div className={styles.itemActions}>
                <button type="button" className={styles.smallButton} onClick={() => handleToggle(rule)}>
                  {rule.enabled ? (
                    <>
                      <Pause size={13} /> Pause
                    </>
                  ) : (
                    <>
                      <Play size={13} /> Resume
                    </>
                  )}
                </button>
                <button type="button" className={styles.smallButton} onClick={() => startEdit(rule)}>
                  Edit
                </button>
                <button type="button" className={styles.dangerButton} onClick={() => handleDelete(rule.id)}>
                  Delete
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}

      {editingId && <p className={styles.hint}>Editing "{form.name}"…</p>}
      <form className={styles.form} onSubmit={handleSubmit}>
        <div className={styles.formRow}>
          <input
            className={styles.input}
            type="text"
            placeholder="Rule name"
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
          />
          <select
            className={styles.select}
            value={form.condition.type}
            onChange={(e) => handleConditionTypeChange(e.target.value as AlertCondition["type"])}
          >
            {CONDITION_TYPES.map((type) => (
              <option key={type} value={type}>
                {CONDITION_LABELS[type]}
              </option>
            ))}
          </select>

          {form.condition.type === "cpu_above" && (
            <input
              className={styles.inputSmall}
              type="number"
              value={form.condition.percent}
              onChange={(e) =>
                setForm({ ...form, condition: conditionField(form.condition, e.target.value) })
              }
            />
          )}
          {form.condition.type === "memory_above" && (
            <input
              className={styles.inputSmall}
              type="number"
              value={form.condition.percent}
              onChange={(e) =>
                setForm({ ...form, condition: conditionField(form.condition, e.target.value) })
              }
            />
          )}
          {form.condition.type === "load_above" && (
            <input
              className={styles.inputSmall}
              type="number"
              step="0.1"
              value={form.condition.value}
              onChange={(e) =>
                setForm({ ...form, condition: conditionField(form.condition, e.target.value) })
              }
            />
          )}
          {(form.condition.type === "network_rx_above" || form.condition.type === "network_tx_above") && (
            <input
              className={styles.inputSmall}
              type="number"
              value={form.condition.bytes_per_sec}
              onChange={(e) =>
                setForm({
                  ...form,
                  condition: conditionField(form.condition, e.target.value),
                })
              }
            />
          )}
          {form.condition.type === "service_not_active" && (
            <input
              className={styles.inputSmall}
              type="text"
              placeholder="service name (e.g. nginx.service)"
              value={form.condition.service_name}
              onChange={(e) =>
                setForm({
                  ...form,
                  condition: conditionField(form.condition, e.target.value),
                })
              }
            />
          )}
        </div>

        <div className={styles.formRow}>
          <select
            className={styles.select}
            value={form.agentAddr}
            onChange={(e) => setForm({ ...form, agentAddr: e.target.value })}
          >
            <option value="">{forcedAgent ? "Select an agent…" : "All agents"}</option>
            {agents.map((agent) => (
              <option key={agent.addr} value={agent.addr}>
                {agent.fullname.split(".")[0]} ({agent.addr})
              </option>
            ))}
          </select>

          <label className={styles.cooldownLabel}>
            cooldown (s)
            <input
              className={styles.inputSmall}
              type="number"
              min={0}
              value={form.cooldownSeconds}
              onChange={(e) => setForm({ ...form, cooldownSeconds: Number(e.target.value) })}
            />
          </label>
        </div>

        <div className={styles.formRow}>
          {webhooks.map((webhook) => (
            <label key={webhook.id} className={styles.webhookOption}>
              <input
                type="checkbox"
                checked={form.webhookIds.includes(webhook.id)}
                onChange={() => toggleWebhook(webhook.id)}
              />
              {webhook.name}
            </label>
          ))}
        </div>

        <div className={styles.formRow}>
          <button type="submit" className={styles.primaryButton} disabled={submitting}>
            {editingId ? "Save changes" : "Add rule"}
          </button>
          {editingId && (
            <button type="button" className={styles.smallButton} onClick={cancelEdit}>
              Cancel
            </button>
          )}
        </div>
      </form>
      {formError && <div className={styles.errorBanner}>{formError}</div>}
    </section>
  );
}
