import { useCallback, useEffect, useState } from "react";
import { listAlertRules } from "../api/client";
import type { AlertRule } from "../api/types";

interface UseAlertRulesResult {
  rules: AlertRule[];
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

export function useAlertRules(): UseAlertRulesResult {
  const [rules, setRules] = useState<AlertRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    listAlertRules()
      .then((result) => {
        setRules(result);
        setError(null);
      })
      .catch((err: unknown) => setError(String(err)))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { rules, loading, error, refresh };
}
