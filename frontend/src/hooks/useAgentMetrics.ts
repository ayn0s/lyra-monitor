import { useCallback, useEffect, useState } from "react";
import { getAgentMetrics } from "../api/client";
import type { AgentMetrics } from "../api/types";
import { useInterval } from "./useInterval";

interface UseAgentMetricsResult {
  metrics: AgentMetrics | null;
  error: string | null;
}

export function useAgentMetrics(addr: string | undefined, pollMs = 2000): UseAgentMetricsResult {
  const [metrics, setMetrics] = useState<AgentMetrics | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    if (!addr) return;
    getAgentMetrics(addr)
      .then((result) => {
        setMetrics(result);
        setError(null);
      })
      .catch((err: unknown) => setError(String(err)));
  }, [addr]);

  useEffect(() => {
    setMetrics(null);
    refresh();
  }, [refresh]);

  useInterval(refresh, addr ? pollMs : null);

  return { metrics, error };
}
