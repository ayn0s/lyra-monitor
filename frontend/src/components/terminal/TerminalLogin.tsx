import { useState } from "react";
import type { FormEvent } from "react";
import styles from "./TerminalLogin.module.css";

interface TerminalLoginProps {
  addr: string;
  onSubmit: (username: string, password: string) => void;
}

export default function TerminalLogin({ addr, onSubmit }: TerminalLoginProps) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!username || !password) return;
    onSubmit(username, password);
  }

  return (
    <div className={styles.wrap}>
      <form className={styles.form} onSubmit={handleSubmit}>
        <div className={styles.title}>Sign in to {addr}</div>
        <div className={styles.subtitle}>Uses the machine's own system accounts (PAM).</div>
        <input
          className={styles.input}
          type="text"
          placeholder="Username"
          autoComplete="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          autoFocus
        />
        <input
          className={styles.input}
          type="password"
          placeholder="Password"
          autoComplete="current-password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <button type="submit" className={styles.submit}>
          Connect
        </button>
      </form>
    </div>
  );
}
