import { useCallback, useEffect, useState } from "react";
import { listAgents } from "../api/client";
import type { AgentInfo } from "../api/types";
import { useInterval } from "./useInterval";

interface UseAgentsResult {
  agents: AgentInfo[];
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

export function useAgents(pollMs = 4000): UseAgentsResult {
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    listAgents()
      .then((result) => {
        setAgents(result);
        setError(null);
      })
      .catch((err: unknown) => setError(String(err)))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useInterval(refresh, pollMs);

  return { agents, loading, error, refresh };
}
