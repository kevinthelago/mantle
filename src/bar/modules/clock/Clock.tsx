import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ClockConfig } from "../../../types/config";
import "./clock.css";

/**
 * Clock module.
 *
 * Polls the Rust backend for the current formatted time string at the
 * interval defined in config (default 1 s).  The Rust side does the
 * formatting so it can honour the user's locale and tz config.
 */
export default function Clock() {
  const [time, setTime] = useState<string>("");
  const [config, setConfig] = useState<ClockConfig | null>(null);

  useEffect(() => {
    // Fetch initial config.
    invoke<ClockConfig>("get_clock_config")
      .then(setConfig)
      .catch(console.error);

    // React to hot-reload.
    const unlisten = listen<{ clock: ClockConfig }>("config-changed", (e) => {
      setConfig(e.payload.clock);
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  useEffect(() => {
    if (!config) return;

    const tick = () => {
      invoke<string>("clock_tick")
        .then(setTime)
        .catch(console.error);
    };

    tick();
    const id = setInterval(tick, config.interval);
    return () => clearInterval(id);
  }, [config]);

  if (!time) return null;

  return (
    <time className="bar-module clock" dateTime={new Date().toISOString()}>
      {time}
    </time>
  );
}
