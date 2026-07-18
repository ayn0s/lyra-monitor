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
}

export interface ServiceUnit {
  name: string;
  description: string;
  load_state: string;
  active_state: string;
  sub_state: string;
}
