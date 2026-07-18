import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import TerminalView from "../components/terminal/TerminalView";
import TerminalLogin from "../components/terminal/TerminalLogin";
import styles from "./AgentTerminalPage.module.css";

interface Credentials {
  username: string;
  password: string;
}

export default function AgentTerminalPage() {
  const { addr = "" } = useParams<{ addr: string }>();
  const [credentials, setCredentials] = useState<Credentials | null>(null);

  useEffect(() => {
    setCredentials(null);
  }, [addr]);

  if (!credentials) {
    return <TerminalLogin addr={addr} onSubmit={(username, password) => setCredentials({ username, password })} />;
  }

  return (
    <div className={styles.terminalWrap}>
      <TerminalView
        addr={addr}
        username={credentials.username}
        password={credentials.password}
        key={credentials.username}
      />
    </div>
  );
}
