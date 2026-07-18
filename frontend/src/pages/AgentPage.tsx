import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { pingAgent } from "../api/client";
import type { PingResult } from "../api/types";
import { useAgentMetrics } from "../hooks/useAgentMetrics";
import MetricTile from "../components/MetricTile";
import styles from "./AgentPage.module.css";

function formatBytes(bytes: number): string {
  return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
}

function formatUptime(totalSeconds: number): string {
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  return `${days}d ${hours}h ${minutes}m`;
}

export default function AgentPage() {
  const { addr = "" } = useParams<{ addr: string }>();
  const { metrics, error } = useAgentMetrics(addr);

  const [pingResult, setPingResult] = useState<PingResult | null>(null);
  const [pinging, setPinging] = useState(false);
  const [pingError, setPingError] = useState<string | null>(null);

  async function handlePing() {
    setPinging(true);
    setPingError(null);
    try {
      setPingResult(await pingAgent(addr));
    } catch (err) {
      setPingError(String(err));
    } finally {
      setPinging(false);
    }
  }

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>{addr}</h1>
          <p className={styles.subtitle}>Live system metrics</p>
        </div>
        <div className={styles.actions}>
          <button className={styles.pingButton} onClick={handlePing} disabled={pinging}>
            {pinging ? "Ping…" : "Ping"}
          </button>
          <Link to={`/agents/${addr}/terminal`} className={styles.terminalButton}>
            Open a terminal
          </Link>
        </div>
      </header>

      {pingError && <div className={styles.errorBanner}>{pingError}</div>}
      {pingResult && (
        <div className={styles.pingResult}>
          round-trip <strong>{pingResult.roundtrip_ms} ms</strong> · server timestamp{" "}
          {new Date(pingResult.server_time_unix_ms).toLocaleTimeString()}
        </div>
      )}

      {error && <div className={styles.errorBanner}>{error}</div>}

      {metrics ? (
        <div className={styles.grid}>
          <MetricTile label="CPU" value={`${metrics.cpu_usage_percent.toFixed(1)} %`} />
          <MetricTile
            label="Memory"
            value={formatBytes(metrics.mem_used_bytes)}
            hint={`of ${formatBytes(metrics.mem_total_bytes)}`}
          />
          <MetricTile label="Load (1m)" value={metrics.load_average_1m.toFixed(2)} />
          <MetricTile label="Uptime" value={formatUptime(metrics.uptime_seconds)} />
        </div>
      ) : (
        !error && <p className={styles.hint}>Loading metrics…</p>
      )}
    </div>
  );
}
