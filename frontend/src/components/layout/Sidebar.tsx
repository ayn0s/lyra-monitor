import { NavLink } from "react-router-dom";
import { useAgents } from "../../hooks/useAgents";
import StatusDot from "../StatusDot";
import styles from "./Sidebar.module.css";

const navLinkClass = ({ isActive }: { isActive: boolean }) =>
  isActive ? styles.navLinkActive : styles.navLink;

export default function Sidebar() {
  const { agents, loading, error } = useAgents();

  return (
    <aside className={styles.sidebar}>
      <div className={styles.brand}>
        <span className={styles.brandMark}>◆</span>
        <span>Lyra Monitor</span>
      </div>

      <nav className={styles.nav}>
        <NavLink to="/" end className={navLinkClass}>
          Dashboard
        </NavLink>
        <NavLink to="/alerts" className={navLinkClass}>
          Alerts
        </NavLink>
      </nav>

      <div className={styles.section}>
        <div className={styles.sectionTitle}>Discovered agents</div>

        {loading && <p className={styles.hint}>Searching mDNS…</p>}
        {error && <p className={styles.error}>Discovery unavailable</p>}
        {!loading && !error && agents.length === 0 && (
          <p className={styles.hint}>No agent visible.</p>
        )}

        <ul className={styles.agentList}>
          {agents.map((agent) => (
            <li key={agent.fullname}>
              <NavLink to={`/agents/${agent.addr}/metrics`} className={navLinkClass}>
                <StatusDot online />
                <span className={styles.agentName}>{agent.fullname.split(".")[0]}</span>
              </NavLink>
            </li>
          ))}
        </ul>
      </div>
    </aside>
  );
}
