import styles from "./MetricTile.module.css";

interface MetricTileProps {
  label: string;
  value: string;
  hint?: string;
}

export default function MetricTile({ label, value, hint }: MetricTileProps) {
  return (
    <div className={styles.tile}>
      <div className={styles.label}>{label}</div>
      <div className={styles.value}>{value}</div>
      {hint && <div className={styles.hint}>{hint}</div>}
    </div>
  );
}
