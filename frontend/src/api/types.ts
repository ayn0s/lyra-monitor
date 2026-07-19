export interface AgentInfo {
  fullname: string;
  addr: string;
  uri: string;
}

export interface PingResult {
  nonce: string;
  server_time_unix_ms: number;
  roundtrip_ms: number;
}

export interface AgentMetrics {
  cpu_usage_percent: number;
  mem_used_bytes: number;
  mem_total_bytes: number;
  load_average_1m: number;
  uptime_seconds: number;
  network_rx_bytes_per_sec: number;
  network_tx_bytes_per_sec: number;
}

export interface MetricsSample {
  timestamp_unix_ms: number;
  cpu_usage_percent: number;
  mem_used_bytes: number;
  mem_total_bytes: number;
  load_average_1m: number;
  network_rx_bytes_per_sec: number;
  network_tx_bytes_per_sec: number;
}

export interface ServiceUnit {
  name: string;
  description: string;
  load_state: string;
  active_state: string;
  sub_state: string;
}

export type WebhookKind = "discord" | "custom";

export interface WebhookTarget {
  id: string;
  name: string;
  kind: WebhookKind;
  url: string;
  enabled: boolean;
}

export type AlertCondition =
  | { type: "cpu_above"; percent: number }
  | { type: "memory_above"; percent: number }
  | { type: "load_above"; value: number }
  | { type: "network_rx_above"; bytes_per_sec: number }
  | { type: "network_tx_above"; bytes_per_sec: number }
  | { type: "agent_offline" }
  | { type: "service_not_active"; service_name: string };

export interface AlertRule {
  id: string;
  name: string;
  agent_addr: string | null;
  condition: AlertCondition;
  cooldown_seconds: number;
  webhook_ids: string[];
  enabled: boolean;
}
