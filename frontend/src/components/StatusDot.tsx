import styles from "./StatusDot.module.css";

interface StatusDotProps {
  online: boolean;
}

export default function StatusDot({ online }: StatusDotProps) {
  return (
    <span
      className={online ? styles.online : styles.offline}
      title={online ? "online" : "offline"}
    />
  );
}
