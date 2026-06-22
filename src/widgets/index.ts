import { registerSurface } from "../lib/surfaces";
import { WidgetLayer } from "./WidgetLayer";

// Register the widgets surface so App.tsx can render it when
// the "widgets" Tauri window is active.
registerSurface({ id: "widgets", component: WidgetLayer });

export { WidgetLayer } from "./WidgetLayer";
export { WidgetSlot } from "./WidgetSlot";
export { WIDGET_REGISTRY } from "./registry";
export type { WidgetConfig, WidgetId, WidgetLayerConfig } from "./types";
