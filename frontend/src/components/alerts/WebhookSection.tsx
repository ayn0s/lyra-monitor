import { useState } from "react";
import { createWebhook, deleteWebhook, testWebhook, updateWebhook } from "../../api/client";
import type { WebhookKind, WebhookTarget } from "../../api/types";
import styles from "./WebhookSection.module.css";

interface WebhookSectionProps {
  webhooks: WebhookTarget[];
  loading: boolean;
  error: string | null;
  onChange: () => void;
}

const EMPTY_FORM = { name: "", kind: "custom" as WebhookKind, url: "", enabled: true };

export default function WebhookSection({ webhooks, loading, error, onChange }: WebhookSectionProps) {
  const [form, setForm] = useState(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [testStatus, setTestStatus] = useState<Record<string, string>>({});

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim() || !form.url.trim()) return;
    setSubmitting(true);
    setFormError(null);
    try {
      await createWebhook(form);
      setForm(EMPTY_FORM);
      onChange();
    } catch (err) {
      setFormError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  const handleToggle = async (webhook: WebhookTarget) => {
    await updateWebhook(webhook.id, { ...webhook, enabled: !webhook.enabled });
    onChange();
  };

  const handleDelete = async (id: string) => {
    await deleteWebhook(id);
    onChange();
  };

  const handleTest = async (id: string) => {
    setTestStatus((prev) => ({ ...prev, [id]: "sending…" }));
    try {
      await testWebhook(id);
      setTestStatus((prev) => ({ ...prev, [id]: "sent ✓" }));
    } catch (err) {
      setTestStatus((prev) => ({ ...prev, [id]: `failed: ${String(err)}` }));
    }
  };

  return (
    <section className={styles.section}>
      <h2 className={styles.title}>Webhooks</h2>
      <p className={styles.hint}>Destinations that alert notifications are sent to.</p>

      {error && <div className={styles.errorBanner}>{error}</div>}

      {!loading && webhooks.length === 0 && !error && (
        <p className={styles.hint}>No webhook configured yet.</p>
      )}

      {webhooks.length > 0 && (
        <ul className={styles.list}>
          {webhooks.map((webhook) => (
            <li key={webhook.id} className={styles.item}>
              <div className={styles.itemMain}>
                <span className={styles.kindBadge}>{webhook.kind}</span>
                <span className={styles.itemName}>{webhook.name}</span>
                <span className={styles.itemUrl}>{webhook.url}</span>
              </div>
              <div className={styles.itemActions}>
                {testStatus[webhook.id] && (
                  <span className={styles.testStatus}>{testStatus[webhook.id]}</span>
                )}
                <button type="button" className={styles.smallButton} onClick={() => handleTest(webhook.id)}>
                  Test
                </button>
                <label className={styles.toggle}>
                  <input
                    type="checkbox"
                    checked={webhook.enabled}
                    onChange={() => handleToggle(webhook)}
                  />
                  enabled
                </label>
                <button
                  type="button"
                  className={styles.dangerButton}
                  onClick={() => handleDelete(webhook.id)}
                >
                  Delete
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}

      <form className={styles.form} onSubmit={handleSubmit}>
        <input
          className={styles.input}
          type="text"
          placeholder="Name"
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
        />
        <select
          className={styles.select}
          value={form.kind}
          onChange={(e) => setForm({ ...form, kind: e.target.value as WebhookKind })}
        >
          <option value="custom">Custom (generic JSON)</option>
          <option value="discord">Discord</option>
        </select>
        <input
          className={styles.inputWide}
          type="text"
          placeholder="Webhook URL"
          value={form.url}
          onChange={(e) => setForm({ ...form, url: e.target.value })}
        />
        <button type="submit" className={styles.primaryButton} disabled={submitting}>
          Add webhook
        </button>
      </form>
      {formError && <div className={styles.errorBanner}>{formError}</div>}
    </section>
  );
}
