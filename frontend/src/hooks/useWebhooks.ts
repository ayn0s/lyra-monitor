import { useCallback, useEffect, useState } from "react";
import { listWebhooks } from "../api/client";
import type { WebhookTarget } from "../api/types";

interface UseWebhooksResult {
  webhooks: WebhookTarget[];
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

export function useWebhooks(): UseWebhooksResult {
  const [webhooks, setWebhooks] = useState<WebhookTarget[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    listWebhooks()
      .then((result) => {
        setWebhooks(result);
        setError(null);
      })
      .catch((err: unknown) => setError(String(err)))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { webhooks, loading, error, refresh };
}
