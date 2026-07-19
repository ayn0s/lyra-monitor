import type { AlertCondition } from "../api/types";
import { formatBytesPerSecond } from "./format";

export const CONDITION_TYPES: AlertCondition["type"][] = [
  "cpu_above",
  "memory_above",
  "load_above",
  "network_rx_above",
  "network_tx_above",
  "agent_offline",
  "service_not_active",
];

export const CONDITION_LABELS: Record<AlertCondition["type"], string> = {
  cpu_above: "CPU above",
  memory_above: "Memory above",
  load_above: "Load average (1m) above",
  network_rx_above: "Download rate above",
  network_tx_above: "Upload rate above",
  agent_offline: "Agent offline",
  service_not_active: "Service not active",
};

export function requiresSpecificAgent(type: AlertCondition["type"]): boolean {
  return type === "agent_offline" || type === "service_not_active";
}

export function defaultConditionFor(type: AlertCondition["type"]): AlertCondition {
  switch (type) {
    case "cpu_above":
      return { type, percent: 90 };
    case "memory_above":
      return { type, percent: 90 };
    case "load_above":
      return { type, value: 4 };
    case "network_rx_above":
      return { type, bytes_per_sec: 10 * 1024 * 1024 };
    case "network_tx_above":
      return { type, bytes_per_sec: 10 * 1024 * 1024 };
    case "agent_offline":
      return { type };
    case "service_not_active":
      return { type, service_name: "" };
  }
}

export function describeCondition(condition: AlertCondition): string {
  switch (condition.type) {
    case "cpu_above":
      return `CPU usage above ${condition.percent}%`;
    case "memory_above":
      return `Memory usage above ${condition.percent}%`;
    case "load_above":
      return `Load average (1m) above ${condition.value}`;
    case "network_rx_above":
      return `Download rate above ${formatBytesPerSecond(condition.bytes_per_sec)}`;
    case "network_tx_above":
      return `Upload rate above ${formatBytesPerSecond(condition.bytes_per_sec)}`;
    case "agent_offline":
      return "Agent is offline";
    case "service_not_active":
      return `Service "${condition.service_name}" is not active`;
  }
}
