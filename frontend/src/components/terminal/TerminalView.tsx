import { useEffect, useRef } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { terminalWsUrl } from "../../api/client";
import styles from "./TerminalView.module.css";

interface TerminalViewProps {
  addr: string;
}

export default function TerminalView({ addr }: TerminalViewProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!containerRef.current || !addr) return;

    const term = new XTerm({
      cursorBlink: true,
      fontFamily: "ui-monospace, Menlo, Consolas, monospace",
      fontSize: 14,
      theme: {
        background: "#0d1117",
        foreground: "#c9d1d9",
        cursor: "#58a6ff",
      },
    });
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(containerRef.current);
    requestAnimationFrame(() => fitAddon.fit());

    const socket = new WebSocket(terminalWsUrl(addr));
    socket.binaryType = "arraybuffer";

    socket.onopen = () => {
      term.writeln(`[lyra] connected to ${addr}, PTY session open.`);
      sendResize();
    };
    socket.onclose = () => term.writeln("\r\n[lyra] connection closed.");
    socket.onerror = () => term.writeln("\r\n[lyra] WebSocket connection error.");

    socket.onmessage = (event) => {
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
    };
  }, [addr]);

  return <div ref={containerRef} className={styles.terminal} />;
}
