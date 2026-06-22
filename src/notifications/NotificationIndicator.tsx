/**
 * Bar indicator for notifications.
 *
 * Registers itself with the bar-shell module registry using the planned
 * contract: window.__mantleBarRegistry.register(id, component, position).
 *
 * The bar-shell stream owns the registry implementation; we call it here
 * against the planned interface. At merge, the registry import will resolve
 * to bar-shell's exported module.
 */
import { useEffect } from "react";
import { useNotificationStore } from "./store";
import styles from "./NotificationIndicator.module.css";

// ── widget component ──────────────────────────────────────────────────────────

export function NotificationIndicator() {
  const history = useNotificationStore((s) => s.history);
  const active = useNotificationStore((s) => s.active);
  const dndEnabled = useNotificationStore((s) => s.dndEnabled);
  const toggleCenter = useNotificationStore((s) => s.toggleCenter);

  const unread = active.length;

  return (
    <button
      className={`${styles.indicator} ${dndEnabled ? styles.dnd : ""}`}
      onClick={toggleCenter}
      aria-label={
        dndEnabled
          ? "Notifications (Do Not Disturb)"
          : `Notifications${unread > 0 ? ` (${unread} new)` : ""}`
      }
      title={dndEnabled ? "DND enabled" : undefined}
    >
      <span className={styles.icon} aria-hidden="true">
        {dndEnabled ? "🔕" : "🔔"}
      </span>
      {unread > 0 && !dndEnabled && (
        <span className={styles.badge} aria-hidden="true">
          {unread > 9 ? "9+" : unread}
        </span>
      )}
    </button>
  );
}

// ── self-registration with bar registry ───────────────────────────────────────

/**
 * Call once at startup to register the indicator into the bar-shell's
 * module registry.  Placement: trailing (right-side) at position 100.
 *
 * If the registry isn't ready yet this retries until it is, so order of
 * initialisation between streams doesn't matter.
 */
export function registerNotificationIndicator(): () => void {
  let cancelled = false;

  function tryRegister() {
    if (cancelled) return;
    // Bar-shell contract: window.__mantleBarRegistry or the ES module export.
    const registry =
      (window as unknown as Record<string, unknown>).__mantleBarRegistry as
        | { register: (id: string, component: unknown, opts: { position: number; slot: string }) => () => void }
        | undefined;

    if (registry) {
      const unregister = registry.register(
        "notifications.indicator",
        NotificationIndicator,
        { position: 100, slot: "trailing" }
      );
      return unregister;
    }

    // Registry not yet available — retry after a tick.
    const t = setTimeout(tryRegister, 50);
    return () => clearTimeout(t);
  }

  const cleanup = tryRegister();
  return () => {
    cancelled = true;
    cleanup?.();
  };
}

// ── hook for use inside the bar-shell render tree ─────────────────────────────

export function useNotificationIndicatorRegistration() {
  useEffect(() => {
    return registerNotificationIndicator();
  }, []);
}
