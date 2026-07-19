import type {
  AgentInfo,
  AgentMetrics,
  AlertRule,
  MetricsSample,
  PingResult,
  ServiceUnit,
  WebhookTarget,
} from "./types";

const API_BASE = "/api";

async function getJSON<T>(path: string): Promise<T> {
  const res = await fetch(path);
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`HTTP ${res.status}${text ? ` — ${text}` : ""}`);
  }
  return res.json() as Promise<T>;
}

async function sendJSON<T>(path: string, method: string, body?: unknown): Promise<T> {
  const res = await fetch(path, {
    method,
    headers: body !== undefined ? { "Content-Type": "application/json" } : undefined,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`HTTP ${res.status}${text ? ` — ${text}` : ""}`);
  }
  return res.json() as Promise<T>;
}

export function toAddr(uri: string): string {
  return uri.replace(/^https?:\/\//, "");
}

export async function listAgents(): Promise<AgentInfo[]> {
  const raw = await getJSON<Record<string, string>>(`${API_BASE}/agents`);
  return Object.entries(raw)
    .map(([fullname, uri]) => ({ fullname, uri, addr: toAddr(uri) }))
    .sort((a, b) => a.fullname.localeCompare(b.fullname));
}

export function pingAgent(addr: string): Promise<PingResult> {
  return getJSON<PingResult>(`${API_BASE}/agents/${addr}/ping`);
}

export function getAgentMetrics(addr: string): Promise<AgentMetrics> {
  return getJSON<AgentMetrics>(`${API_BASE}/agents/${addr}/metrics`);
}

export function getAgentMetricsHistory(addr: string): Promise<MetricsSample[]> {
  return getJSON<MetricsSample[]>(`${API_BASE}/agents/${addr}/metrics/history`);
}

export function getAgentServices(addr: string): Promise<ServiceUnit[]> {
  return getJSON<ServiceUnit[]>(`${API_BASE}/agents/${addr}/services`);
}

export function terminalWsUrl(addr: string): string {
  const protocol = window.location.protocol === "https:" ? "wss" : "ws";
  return `${protocol}://${window.location.host}/ws/terminal/${addr}`;
}

export function listWebhooks(): Promise<WebhookTarget[]> {
  return getJSON<WebhookTarget[]>(`${API_BASE}/webhooks`);
}

export function createWebhook(webhook: Omit<WebhookTarget, "id">): Promise<WebhookTarget> {
  return sendJSON<WebhookTarget>(`${API_BASE}/webhooks`, "POST", { id: "", ...webhook });
}

export function updateWebhook(id: string, webhook: WebhookTarget): Promise<WebhookTarget> {
  return sendJSON<WebhookTarget>(`${API_BASE}/webhooks/${id}`, "PUT", webhook);
}

export function deleteWebhook(id: string): Promise<void> {
  return sendJSON(`${API_BASE}/webhooks/${id}`, "DELETE");
}

export function testWebhook(id: string): Promise<void> {
  return sendJSON(`${API_BASE}/webhooks/${id}/test`, "POST");
}

export function listAlertRules(): Promise<AlertRule[]> {
  return getJSON<AlertRule[]>(`${API_BASE}/alert-rules`);
}

export function createAlertRule(rule: Omit<AlertRule, "id">): Promise<AlertRule> {
  return sendJSON<AlertRule>(`${API_BASE}/alert-rules`, "POST", { id: "", ...rule });
}

export function updateAlertRule(id: string, rule: AlertRule): Promise<AlertRule> {
  return sendJSON<AlertRule>(`${API_BASE}/alert-rules/${id}`, "PUT", rule);
}

export function deleteAlertRule(id: string): Promise<void> {
  return sendJSON(`${API_BASE}/alert-rules/${id}`, "DELETE");
}
