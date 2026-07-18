import { useMemo } from "react";
import { useParams } from "react-router-dom";
import { useAgentMetrics } from "../hooks/useAgentMetrics";
import { useMetricsHistory } from "../hooks/useMetricsHistory";
import { formatBytes, formatUptime } from "../lib/format";
import MetricTile from "../components/MetricTile";
import MetricChart from "../components/MetricChart";
import styles from "./AgentMetricsPage.module.css";

export default function AgentMetricsPage() {
  const { addr = "" } = useParams<{ addr: string }>();
  const { metrics, error } = useAgentMetrics(addr);
  const { history, error: historyError } = useMetricsHistory(addr);

  const timestamps = useMemo(
    () => history.map((sample) => Math.floor(sample.timestamp_unix_ms / 1000)),
    [history],
  );

  const cpuData = useMemo<[number[], number[]]>(
    () => [timestamps, history.map((sample) => sample.cpu_usage_percent)],
    [timestamps, history],
  );

  const memoryData = useMemo<[number[], number[]]>(
    () => [
      timestamps,
      history.map((sample) => (sample.mem_used_bytes / sample.mem_total_bytes) * 100),
    ],
    [timestamps, history],
  );

  const loadData = useMemo<[number[], number[]]>(
    () => [timestamps, history.map((sample) => sample.load_average_1m)],
    [timestamps, history],
  );

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

      {historyError && <div className={styles.errorBanner}>{historyError}</div>}

      {history.length >= 2 ? (
        <div className={styles.charts}>
          <MetricChart
            title="CPU usage"
            data={cpuData}
            colorVar="--color-chart-1"
            yRange={[0, 100]}
            valueFormatter={(v) => `${v.toFixed(0)}%`}
          />
          <MetricChart
            title="Memory usage"
            data={memoryData}
            colorVar="--color-chart-2"
            yRange={[0, 100]}
            valueFormatter={(v) => `${v.toFixed(0)}%`}
          />
          <MetricChart
            title="Load average (1m)"
            data={loadData}
            colorVar="--color-chart-3"
            valueFormatter={(v) => v.toFixed(2)}
          />
        </div>
      ) : (
        !historyError && <p className={styles.hint}>Collecting history…</p>
      )}
    </div>
  );
}
