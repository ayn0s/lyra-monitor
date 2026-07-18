import { useEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import styles from "./MetricChart.module.css";

interface MetricChartProps {
  title: string;
  data: uPlot.AlignedData;
  /** Name of a CSS custom property (e.g. "--color-chart-1"), resolved at render time. */
  colorVar: string;
  valueFormatter?: (value: number) => string;
  yRange?: [number, number];
}

export default function MetricChart({
  title,
  data,
  colorVar,
  valueFormatter,
  yRange,
}: MetricChartProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const plotRef = useRef<uPlot | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const style = getComputedStyle(document.documentElement);
    const mutedColor = style.getPropertyValue("--color-text-muted").trim();
    const borderColor = style.getPropertyValue("--color-border").trim();
    const color = style.getPropertyValue(colorVar).trim();

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
        {
          label: title,
          stroke: color,
          width: 2,
          fill: `${color}22`,
          points: { show: false },
          value: (_u, v) => (v == null ? "–" : (valueFormatter?.(v) ?? String(v))),
        },
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
  }, [title, colorVar]);

  useEffect(() => {
    plotRef.current?.setData(data);
  }, [data]);

  return (
    <div className={styles.chart}>
      <div className={styles.title}>{title}</div>
      <div ref={containerRef} className={styles.plot} />
    </div>
  );
}
