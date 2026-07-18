import { useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { useAgentServices } from "../hooks/useAgentServices";
import StatusDot from "../components/StatusDot";
import styles from "./AgentServicesPage.module.css";

export default function AgentServicesPage() {
  const { addr = "" } = useParams<{ addr: string }>();
  const { services, loading, error } = useAgentServices(addr);
  const [filter, setFilter] = useState("");

  const filtered = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    if (!needle) return services;
    return services.filter(
      (service) =>
        service.name.toLowerCase().includes(needle) ||
        service.description.toLowerCase().includes(needle),
    );
  }, [services, filter]);

  return (
    <div className={styles.page}>
      {error && <div className={styles.errorBanner}>{error}</div>}

      {!error && (
        <input
          className={styles.filter}
          type="text"
          placeholder="Filter services…"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
      )}

      {loading && services.length === 0 && !error && (
        <p className={styles.hint}>Loading services…</p>
      )}

      {!loading && filtered.length === 0 && !error && (
        <p className={styles.hint}>
          {services.length === 0 ? "No .service unit found." : "No service matches the filter."}
        </p>
      )}

      {filtered.length > 0 && (
        <div className={styles.tableWrap}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th></th>
                <th>Name</th>
                <th>Description</th>
                <th>State</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((service) => (
                <tr key={service.name}>
                  <td>
                    <StatusDot online={service.active_state === "active"} />
                  </td>
                  <td className={styles.name}>{service.name}</td>
                  <td className={styles.description}>{service.description}</td>
                  <td className={styles.state}>
                    {service.active_state} ({service.sub_state})
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
