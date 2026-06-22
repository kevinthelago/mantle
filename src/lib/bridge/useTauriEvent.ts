import { useEffect, DependencyList } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * Subscribes to a Tauri backend event and fires `handler` on each payload.
 * Automatically unlistens when the component unmounts or `deps` change.
 *
 * Use this for stream-style events emitted by the Rust side (e.g. compositor
 * workspace changes, battery updates). For one-shot queries use `useCommand`.
 *
 * @example
 * useTauriEvent<WorkspaceEvent>('workspace-changed', (ws) => {
 *   setWorkspaces(ws.workspaces)
 * })
 */
export function useTauriEvent<T>(
  event: string,
  handler: (payload: T) => void,
  deps: DependencyList = [],
): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    listen<T>(event, (e) => handler(e.payload))
      .then((fn) => {
        unlisten = fn;
      })
      .catch((err) => {
        console.error(
          `[useTauriEvent] failed to subscribe to "${event}":`,
          err,
        );
      });

    return () => {
      unlisten?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [event, ...deps]);
}
