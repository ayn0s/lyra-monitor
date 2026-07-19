import { useEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import styles from "./MetricChart.module.css";

export interface MetricSeries {
  label: string;
  /** Name of a CSS custom property (e.g. "--color-chart-1"), resolved at render time. */
  colorVar: string;
  values: number[];
}

interface MetricChartProps {
  title: string;
  timestamps: number[];
  series: MetricSeries[];
  valueFormatter?: (value: number) => string;
  yRange?: [number, number];
}

export default function MetricChart({
  title,
  timestamps,
  series,
  valueFormatter,
  yRange,
}: MetricChartProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const plotRef = useRef<uPlot | null>(null);
  const seriesKey = series.map((s) => `${s.label}:${s.colorVar}`).join("|");

  useEffect(() => {
    if (!containerRef.current) return;

    const style = getComputedStyle(document.documentElement);
    const mutedColor = style.getPropertyValue("--color-text-muted").trim();
    const borderColor = style.getPropertyValue("--color-border").trim();

    const data: uPlot.AlignedData = [timestamps, ...series.map((s) => s.values)];

    const opts: uPlot.Options = {
      width: containerRef.current.clientWidth,
      height: 160,
      padding: [8, 8, 0, 0],
      scales: {
        x: { time: true },
        ...(yRange ? { y: { range: () => yRange } } : {}),
      },
      series: [
        {},
        ...series.map((s) => {
          const color = style.getPropertyValue(s.colorVar).trim();
          return {
            label: s.label,
            stroke: color,
            width: 2,
            fill: `${color}22`,
            points: { show: false },
            value: (_u: uPlot, v: number | null) =>
              v == null ? "–" : (valueFormatter?.(v) ?? String(v)),
          };
        }),
      ],
      axes: [
        { stroke: mutedColor, grid: { stroke: borderColor, width: 1 } },
        {
          stroke: mutedColor,
          grid: { stroke: borderColor, width: 1 },
          values: (_u, vals) => vals.map((v) => (valueFormatter ? valueFormatter(v) : String(v))),
        },
      ],
      cursor: { points: { show: false } },
      legend: { show: true },
    };

    const plot = new uPlot(opts, data, containerRef.current);
    plotRef.current = plot;

    const resizeObserver = new ResizeObserver(() => {
      if (!containerRef.current) return;
      plot.setSize({ width: containerRef.current.clientWidth, height: 160 });
    });
    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
      plot.destroy();
      plotRef.current = null;
    };
  }, [title, seriesKey]);

  useEffect(() => {
    plotRef.current?.setData([timestamps, ...series.map((s) => s.values)]);
  }, [timestamps, series]);

  return (
    <div className={styles.chart}>
      <div className={styles.title}>{title}</div>
      <div ref={containerRef} className={styles.plot} />
    </div>
  );
}
