import { useMemo } from "react";
import { useParams } from "react-router-dom";
import { useAgentMetrics } from "../hooks/useAgentMetrics";
import { useMetricsHistory } from "../hooks/useMetricsHistory";
import { formatBytes, formatBytesPerSecond, formatUptime } from "../lib/format";
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

  const cpuSeries = useMemo(
    () => [
      {
        label: "CPU",
        colorVar: "--color-chart-1",
        values: history.map((sample) => sample.cpu_usage_percent),
      },
    ],
    [history],
  );

  const memorySeries = useMemo(
    () => [
      {
        label: "Memory",
        colorVar: "--color-chart-2",
        values: history.map((sample) => (sample.mem_used_bytes / sample.mem_total_bytes) * 100),
      },
    ],
    [history],
  );

  const loadSeries = useMemo(
    () => [
      {
        label: "Load (1m)",
        colorVar: "--color-chart-3",
        values: history.map((sample) => sample.load_average_1m),
      },
    ],
    [history],
  );

  const networkSeries = useMemo(
    () => [
      {
        label: "Download",
        colorVar: "--color-chart-1",
        values: history.map((sample) => sample.network_rx_bytes_per_sec),
      },
      {
        label: "Upload",
        colorVar: "--color-chart-3",
        values: history.map((sample) => sample.network_tx_bytes_per_sec),
      },
    ],
    [history],
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
          <MetricTile
            label="Network"
            value={`↓ ${formatBytesPerSecond(metrics.network_rx_bytes_per_sec)}`}
            hint={`↑ ${formatBytesPerSecond(metrics.network_tx_bytes_per_sec)}`}
          />
        </div>
      ) : (
        !error && <p className={styles.hint}>Loading metrics…</p>
      )}

      {historyError && <div className={styles.errorBanner}>{historyError}</div>}

      {history.length >= 2 ? (
        <div className={styles.charts}>
          <MetricChart
            title="CPU usage"
            timestamps={timestamps}
            series={cpuSeries}
            yRange={[0, 100]}
            valueFormatter={(v) => `${v.toFixed(0)}%`}
          />
          <MetricChart
            title="Memory usage"
            timestamps={timestamps}
            series={memorySeries}
            yRange={[0, 100]}
            valueFormatter={(v) => `${v.toFixed(0)}%`}
          />
          <MetricChart
            title="Load average (1m)"
            timestamps={timestamps}
            series={loadSeries}
            valueFormatter={(v) => v.toFixed(2)}
          />
          <MetricChart
            title="Network load"
            timestamps={timestamps}
            series={networkSeries}
            valueFormatter={formatBytesPerSecond}
          />
        </div>
      ) : (
        !historyError && <p className={styles.hint}>Collecting history…</p>
      )}
    </div>
  );
}
