import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { WidgetLayerConfig } from "./types";
import { WIDGET_REGISTRY } from "./registry";
import { WidgetSlot } from "./WidgetSlot";
import styles from "./WidgetLayer.module.css";

const FALLBACK_CONFIG: WidgetLayerConfig = {
  widgets: [
    { id: "calendar", anchor: "top-right", visible: true },
    { id: "system-monitor", anchor: "top-right", visible: true },
    { id: "media-controls", anchor: "bottom-right", visible: true },
  ],
};

export function WidgetLayer() {
  const [config, setConfig] = useState<WidgetLayerConfig>(FALLBACK_CONFIG);
  const [layerVisible, setLayerVisible] = useState(true);

  useEffect(() => {
    invoke<WidgetLayerConfig>("get_widget_config")
      .then(setConfig)
      .catch(() => {/* use fallback */});

    const unlisten = listen("widget-layer:toggle", () => {
      setLayerVisible((v) => !v);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  if (!layerVisible) return null;

  return (
    <div className={styles.layer}>
      {config.widgets
        .filter((w) => w.visible)
        .map((w) => {
          const Widget = WIDGET_REGISTRY[w.id];
          if (!Widget) return null;
          const anchorClass =
            styles[w.anchor.replace("-", "_") as keyof typeof styles];
          return (
            <div
              key={w.id}
              className={`${styles.slot} ${anchorClass ?? ""}`}
            >
              <WidgetSlot widgetId={w.id}>
                <Widget />
              </WidgetSlot>
            </div>
          );
        })}
    </div>
  );
}
