import { registerSurface } from "../../lib/surfaces";
import BarShell from "./BarShell";

// Register the bar surface so App.tsx renders BarShell when the "bar-*"
// Tauri window is active.  main.tsx must import this module as a side-effect
// (e.g. `import './bar/shell'`) for the registration to take effect.
registerSurface({ id: "bar", component: BarShell });

export { default as BarShell } from "./BarShell";
export { default as BarRegion } from "./BarRegion";
export { ErrorBoundary } from "./ErrorBoundary";
export { outputFromLabel } from "./BarShell";
