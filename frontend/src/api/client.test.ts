import { afterEach, describe, expect, it, vi } from "vitest";
import { listAgents, pingAgent, terminalWsUrl, toAddr } from "./client";

describe("toAddr", () => {
  it("strips the http scheme", () => {
    expect(toAddr("http://192.168.1.10:50051")).toBe("192.168.1.10:50051");
  });

  it("strips the https scheme", () => {
    expect(toAddr("https://192.168.1.10:50051")).toBe("192.168.1.10:50051");
  });

  it("leaves a schemeless address untouched", () => {
    expect(toAddr("192.168.1.10:50051")).toBe("192.168.1.10:50051");
  });
});

describe("terminalWsUrl", () => {
  it("builds a ws:// url from the current host", () => {
    expect(terminalWsUrl("192.168.1.10:50051")).toBe(
      `ws://${window.location.host}/ws/terminal/192.168.1.10:50051`,
    );
  });
});

describe("fetch-backed API calls", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("listAgents sorts by fullname and derives addr from the URI", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        "zeta._lyra-agent._tcp.local.": "http://10.0.0.2:50051",
        "alpha._lyra-agent._tcp.local.": "http://10.0.0.1:50051",
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    const agents = await listAgents();

    expect(fetchMock).toHaveBeenCalledWith("/api/agents");
    expect(agents.map((a) => a.fullname)).toEqual([
      "alpha._lyra-agent._tcp.local.",
      "zeta._lyra-agent._tcp.local.",
    ]);
    expect(agents[0].addr).toBe("10.0.0.1:50051");
  });

  it("rejects with a descriptive error on a non-ok response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 502,
        text: async () => "agent unreachable",
      }),
    );

    await expect(pingAgent("10.0.0.1:50051")).rejects.toThrow("HTTP 502 — agent unreachable");
  });
});
