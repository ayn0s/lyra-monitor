import { useParams } from "react-router-dom";
import TerminalView from "../components/terminal/TerminalView";
import styles from "./AgentTerminalPage.module.css";

export default function AgentTerminalPage() {
  const { addr = "" } = useParams<{ addr: string }>();

  return (
    <div className={styles.terminalWrap}>
      <TerminalView addr={addr} />
    </div>
  );
}
