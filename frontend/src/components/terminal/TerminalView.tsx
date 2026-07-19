import { useEffect, useRef } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { terminalWsUrl } from "../../api/client";
import { useTheme } from "../../hooks/useTheme";
import styles from "./TerminalView.module.css";

interface TerminalViewProps {
  addr: string;
  username: string;
  password: string;
}

function resolveXtermTheme() {
  const style = getComputedStyle(document.documentElement);
  return {
    background: style.getPropertyValue("--color-bg-terminal").trim(),
    foreground: style.getPropertyValue("--color-text-terminal").trim(),
    cursor: style.getPropertyValue("--color-accent").trim(),
  };
}

export default function TerminalView({ addr, username, password }: TerminalViewProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const termRef = useRef<XTerm | null>(null);
  const { theme } = useTheme();

  useEffect(() => {
    if (!containerRef.current || !addr) return;

    const term = new XTerm({
      cursorBlink: true,
      fontFamily: "ui-monospace, Menlo, Consolas, monospace",
      fontSize: 14,
      theme: resolveXtermTheme(),
    });
    termRef.current = term;
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(containerRef.current);
    requestAnimationFrame(() => fitAddon.fit());

    const socket = new WebSocket(terminalWsUrl(addr));
    socket.binaryType = "arraybuffer";

    socket.onopen = () => {
      term.writeln(`[lyra] connecting to ${addr} as ${username}…`);
      socket.send(JSON.stringify({ type: "auth", username, password }));
      sendResize();
    };
    socket.onclose = () => term.writeln("\r\n[lyra] connection closed.");
    socket.onerror = () => term.writeln("\r\n[lyra] WebSocket connection error.");

    socket.onmessage = (event) => {
      if (typeof event.data === "string") {
        term.writeln(`\r\n${event.data}`);
        return;
      }
      term.write(new Uint8Array(event.data as ArrayBuffer));
    };

    const onDataDisposable = term.onData((data) => {
      if (socket.readyState === WebSocket.OPEN) {
        socket.send(new TextEncoder().encode(data));
      }
    });

    function sendResize() {
      if (socket.readyState !== WebSocket.OPEN) return;
      socket.send(JSON.stringify({ type: "resize", cols: term.cols, rows: term.rows }));
    }

    const onResize = () => {
      fitAddon.fit();
      sendResize();
    };
    const resizeObserver = new ResizeObserver(onResize);
    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
      onDataDisposable.dispose();
      socket.close();
      term.dispose();
      termRef.current = null;
    };
  }, [addr, username, password]);

  useEffect(() => {
    if (termRef.current) {
      termRef.current.options.theme = resolveXtermTheme();
    }
  }, [theme]);

  return <div ref={containerRef} className={styles.terminal} />;
}
