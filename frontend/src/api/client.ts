import type { AgentInfo, AgentMetrics, MetricsSample, PingResult, ServiceUnit } from "./types";

const API_BASE = "/api";

async function getJSON<T>(path: string): Promise<T> {
  const res = await fetch(path);
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
