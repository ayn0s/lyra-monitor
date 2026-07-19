import { describe, expect, it } from "vitest";
import {
  defaultConditionFor,
  describeCondition,
  requiresSpecificAgent,
} from "./alertConditions";

describe("requiresSpecificAgent", () => {
  it("requires a specific agent for offline and service conditions", () => {
    expect(requiresSpecificAgent("agent_offline")).toBe(true);
    expect(requiresSpecificAgent("service_not_active")).toBe(true);
  });

  it("does not require a specific agent for metric conditions", () => {
    expect(requiresSpecificAgent("cpu_above")).toBe(false);
    expect(requiresSpecificAgent("network_rx_above")).toBe(false);
  });
});

describe("describeCondition", () => {
  it("describes a cpu_above condition", () => {
    expect(describeCondition({ type: "cpu_above", percent: 90 })).toBe(
      "CPU usage above 90%",
    );
  });

  it("describes a memory_above condition", () => {
    expect(describeCondition({ type: "memory_above", percent: 80 })).toBe(
      "Memory usage above 80%",
    );
  });

  it("describes a load_above condition", () => {
    expect(describeCondition({ type: "load_above", value: 4 })).toBe(
      "Load average (1m) above 4",
    );
  });

  it("describes network conditions using byte-rate formatting", () => {
    expect(
      describeCondition({ type: "network_rx_above", bytes_per_sec: 1024 }),
    ).toBe("Download rate above 1.0 KB/s");
    expect(
      describeCondition({ type: "network_tx_above", bytes_per_sec: 512 }),
    ).toBe("Upload rate above 512 B/s");
  });

  it("describes an agent_offline condition", () => {
    expect(describeCondition({ type: "agent_offline" })).toBe("Agent is offline");
  });

  it("describes a service_not_active condition", () => {
    expect(
      describeCondition({ type: "service_not_active", service_name: "nginx.service" }),
    ).toBe('Service "nginx.service" is not active');
  });
});

describe("defaultConditionFor", () => {
  it("produces a condition matching the requested type", () => {
    for (const type of [
      "cpu_above",
      "memory_above",
      "load_above",
      "network_rx_above",
      "network_tx_above",
      "agent_offline",
      "service_not_active",
    ] as const) {
      expect(defaultConditionFor(type).type).toBe(type);
    }
  });
});
