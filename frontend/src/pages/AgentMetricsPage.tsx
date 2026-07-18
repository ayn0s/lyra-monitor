import { useParams } from "react-router-dom";
import { useAgentMetrics } from "../hooks/useAgentMetrics";
import MetricTile from "../components/MetricTile";
import styles from "./AgentMetricsPage.module.css";

function formatBytes(bytes: number): string {
  return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
}

function formatUptime(totalSeconds: number): string {
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  return `${days}d ${hours}h ${minutes}m`;
}

export default function AgentMetricsPage() {
  const { addr = "" } = useParams<{ addr: string }>();
  const { metrics, error } = useAgentMetrics(addr);

  return (
    <div className={styles.page}>
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
