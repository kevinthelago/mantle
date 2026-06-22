/**
 * TS surface registry — lets other streams register their surface components
 * without editing core files (src/App.tsx, src/main.tsx).
 *
 * Registration should happen at module load time (top-level `registerSurface`
 * call in each surface's entry module). App.tsx reads `getAllSurfaces()` to
 * render whatever is registered.
 *
 * @example
 * // In packages/bar-shell/src/index.ts:
 * import { registerSurface } from '@mantle/core/surfaces'
 * registerSurface({ id: 'bar', component: Bar })
 */

import type { ComponentType } from "react";

export interface SurfaceConfig {
  /** Unique surface identifier — must match the Tauri window label. */
  id: string;
  /** React component to render inside the surface window. */
  component: ComponentType;
  /** Optional static props passed to the component. */
  defaultProps?: Record<string, unknown>;
}

const registry = new Map<string, SurfaceConfig>();

export function registerSurface(config: SurfaceConfig): void {
  if (registry.has(config.id)) {
    console.warn(
      `[surfaces] surface "${config.id}" already registered — overwriting`,
    );
  }
  registry.set(config.id, config);
}

export function getSurface(id: string): SurfaceConfig | undefined {
  return registry.get(id);
}

export function getAllSurfaces(): ReadonlyMap<string, SurfaceConfig> {
  return registry;
}

export function unregisterSurface(id: string): void {
  registry.delete(id);
}
