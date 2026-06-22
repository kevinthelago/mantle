import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { FocusedWindow } from "../../../types/config";
import "./window-title.css";

/**
 * Focused window title module.
 *
 * Subscribes to `focused-window` events from the compositor stream.
 * Falls back gracefully when no window is focused.
 */
export default function WindowTitle() {
  const [focused, setFocused] = useState<FocusedWindow | null>(null);

  useEffect(() => {
    invoke<FocusedWindow>("get_focused_window")
      .then(setFocused)
      .catch(console.error);

    const unlisten = listen<FocusedWindow>("focused-window", (event) => {
      setFocused(event.payload);
    });

    return () => { unlisten.then((f) => f()); };
  }, []);

  const title = focused?.title ?? "";

  if (!title) return null;

  return (
    <span className="bar-module window-title" title={title}>
      {title}
    </span>
  );
}
