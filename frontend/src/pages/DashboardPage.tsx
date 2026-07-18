import { Link } from "react-router-dom";
import { useAgents } from "../hooks/useAgents";
import StatusDot from "../components/StatusDot";
import styles from "./DashboardPage.module.css";

export default function DashboardPage() {
  const { agents, loading, error, refresh } = useAgents();

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1>Agents</h1>
          <p className={styles.subtitle}>
            Machines automatically discovered on the local network via mDNS.
          </p>
        </div>
        <button className={styles.refreshButton} onClick={refresh}>
          Refresh
        </button>
      </header>

      {error && <div className={styles.errorBanner}>Discovery error: {error}</div>}

      {loading && agents.length === 0 && <p className={styles.hint}>Searching for agents…</p>}

      {!loading && agents.length === 0 && !error && (
        <div className={styles.emptyState}>
          <p>No agent detected yet.</p>
          <p className={styles.hint}>
            Check that <code>lyra-agent</code> is running on the network and that mDNS multicast
            is not blocked (firewall, isolated Docker network, etc.).
          </p>
        </div>
      )}

      <div className={styles.grid}>
        {agents.map((agent) => (
          <Link to={`/agents/${agent.addr}`} key={agent.fullname} className={styles.card}>
            <div className={styles.cardHeader}>
              <StatusDot online />
              <span className={styles.cardTitle}>{agent.fullname.split(".")[0]}</span>
            </div>
            <div className={styles.cardAddr}>{agent.addr}</div>
          </Link>
        ))}
      </div>
    </div>
  );
}
