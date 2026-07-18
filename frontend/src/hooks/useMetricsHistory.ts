import { useCallback, useEffect, useState } from "react";
import { getAgentMetricsHistory } from "../api/client";
import type { MetricsSample } from "../api/types";
import { useInterval } from "./useInterval";

interface UseMetricsHistoryResult {
  history: MetricsSample[];
  error: string | null;
}

export function useMetricsHistory(
  addr: string | undefined,
  pollMs = 10000,
): UseMetricsHistoryResult {
  const [history, setHistory] = useState<MetricsSample[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    if (!addr) return;
    getAgentMetricsHistory(addr)
      .then((result) => {
        setHistory(result);
        setError(null);
      })
      .catch((err: unknown) => setError(String(err)));
  }, [addr]);

  useEffect(() => {
    setHistory([]);
    refresh();
  }, [refresh]);

  useInterval(refresh, addr ? pollMs : null);

  return { history, error };
}
