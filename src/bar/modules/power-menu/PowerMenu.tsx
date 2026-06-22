import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PowerAction, Config } from "../../../types/config";
import "./power-menu.css";

const ACTION_LABELS: Record<PowerAction, string> = {
  lock: "Lock",
  logout: "Log out",
  suspend: "Suspend",
  hibernate: "Hibernate",
  hybrid_sleep: "Hybrid sleep",
  reboot: "Reboot",
  poweroff: "Power off",
};

const ACTION_ICONS: Record<PowerAction, string> = {
  lock: "🔒",
  logout: "↩",
  suspend: "💤",
  hibernate: "🌙",
  hybrid_sleep: "☽",
  reboot: "↺",
  poweroff: "⏻",
};

/**
 * Power menu module.
 *
 * Renders a button that opens a dropdown with power actions.
 * Actions are executed via the logind D-Bus service (Rust side).
 */
export default function PowerMenu() {
  const [open, setOpen] = useState(false);
  const [actions, setActions] = useState<PowerAction[]>([]);
  const [pending, setPending] = useState<PowerAction | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<Config>("get_config")
      .then((cfg) => setActions(cfg.power_menu.actions))
      .catch(console.error);

    const unlisten = listen<Config>("config-changed", (e) => {
      setActions(e.payload.power_menu.actions);
    });

    return () => { unlisten.then((f) => f()); };
  }, []);

  // Close on outside click.
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (!containerRef.current?.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  // Close on Escape.
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [open]);

  const execute = async (action: PowerAction) => {
    setOpen(false);
    setPending(action);
    try {
      await invoke("power_action", { action });
    } catch (e) {
      console.error("[power-menu]", e);
    } finally {
      setPending(null);
    }
  };

  return (
    <div ref={containerRef} className="bar-module power-menu">
      <button
        className="power-menu__trigger"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label="Power menu"
        onClick={() => setOpen((o) => !o)}
      >
        {pending ? "…" : "⏻"}
      </button>

      {open && (
        <ul className="power-menu__dropdown" role="menu">
          {actions.map((action) => (
            <li key={action} role="none">
              <button
                role="menuitem"
                className="power-menu__item"
                onClick={() => execute(action)}
              >
                <span className="power-menu__icon" aria-hidden>
                  {ACTION_ICONS[action]}
                </span>
                {ACTION_LABELS[action]}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
