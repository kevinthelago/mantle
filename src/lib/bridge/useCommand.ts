import { useState, useEffect, useCallback, DependencyList } from "react";

export interface CommandState<T> {
  data: T | null;
  error: string | null;
  loading: boolean;
  /** Re-run the command and refresh state. */
  refresh: () => void;
}

/**
 * Wraps a tauri-specta snapshot command as React state.
 *
 * The command fires once on mount (and whenever `deps` change), populating
 * `data` on success or `error` on failure. `refresh` re-runs it manually.
 *
 * @example
 * const { data, loading } = useCommand(() => commands.getWorkspaces())
 */
export function useCommand<T>(
  command: () => Promise<T>,
  deps: DependencyList = [],
): CommandState<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // eslint-disable-next-line react-hooks/exhaustive-deps
  const run = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setData(await command());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  useEffect(() => {
    void run();
  }, [run]);

  return { data, error, loading, refresh: run };
}
