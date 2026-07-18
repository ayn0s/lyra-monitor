import { Link, useParams } from "react-router-dom";
import TerminalView from "../components/terminal/TerminalView";
import styles from "./TerminalPage.module.css";

export default function TerminalPage() {
  const { addr = "" } = useParams<{ addr: string }>();

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <h1 className={styles.title}>Terminal — {addr}</h1>
        <Link to={`/agents/${addr}`} className={styles.backLink}>
          ← back to metrics
        </Link>
      </header>
      <div className={styles.terminalWrap}>
        <TerminalView addr={addr} />
      </div>
    </div>
  );
}
