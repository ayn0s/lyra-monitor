import { useCallback, useEffect, useState } from "react";
import { getAgentServices } from "../api/client";
import type { ServiceUnit } from "../api/types";
import { useInterval } from "./useInterval";

interface UseAgentServicesResult {
  services: ServiceUnit[];
  loading: boolean;
  error: string | null;
}

export function useAgentServices(
  addr: string | undefined,
  pollMs = 5000,
): UseAgentServicesResult {
  const [services, setServices] = useState<ServiceUnit[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    if (!addr) return;
    getAgentServices(addr)
      .then((result) => {
        setServices(result);
        setError(null);
      })
      .catch((err: unknown) => setError(String(err)))
      .finally(() => setLoading(false));
  }, [addr]);

  useEffect(() => {
    setServices([]);
    setLoading(true);
    refresh();
  }, [refresh]);

  useInterval(refresh, addr ? pollMs : null);

  return { services, loading, error };
}
