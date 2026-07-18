import { useState } from "react";
import { NavLink, Outlet, useParams } from "react-router-dom";
import { pingAgent } from "../../api/client";
import type { PingResult } from "../../api/types";
import styles from "./AgentLayout.module.css";

const tabClass = ({ isActive }: { isActive: boolean }) =>
  isActive ? styles.tabActive : styles.tab;

export default function AgentLayout() {
  const { addr = "" } = useParams<{ addr: string }>();

  const [pingResult, setPingResult] = useState<PingResult | null>(null);
  const [pinging, setPinging] = useState(false);
  const [pingError, setPingError] = useState<string | null>(null);

  async function handlePing() {
    setPinging(true);
    setPingError(null);
    try {
      setPingResult(await pingAgent(addr));
    } catch (err) {
      setPingError(String(err));
    } finally {
      setPinging(false);
    }
  }

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>{addr}</h1>
          <p className={styles.subtitle}>Machine details</p>
        </div>
        <div className={styles.actions}>
          <button className={styles.pingButton} onClick={handlePing} disabled={pinging}>
            {pinging ? "Ping…" : "Ping"}
          </button>
        </div>
      </header>

      {pingError && <div className={styles.errorBanner}>{pingError}</div>}
      {pingResult && (
        <div className={styles.pingResult}>
          round-trip <strong>{pingResult.roundtrip_ms} ms</strong> · server timestamp{" "}
          {new Date(pingResult.server_time_unix_ms).toLocaleTimeString()}
        </div>
      )}

      <nav className={styles.tabs}>
        <NavLink to={`/agents/${addr}/metrics`} className={tabClass}>
          Metrics
        </NavLink>
        <NavLink to={`/agents/${addr}/services`} className={tabClass}>
          Services
        </NavLink>
        <NavLink to={`/agents/${addr}/terminal`} className={tabClass}>
          Terminal
        </NavLink>
      </nav>

      <div className={styles.outlet}>
        <Outlet />
      </div>
    </div>
  );
}
