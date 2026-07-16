import { lazy, type ComponentType } from 'react'

// A module entry resolves to a `{ default }` shape that React.lazy can render.
type ModuleLoader = () => Promise<{ default: ComponentType }>

// Modules that export a `barModule` registration contract ({ id, region,
// order, component }) rather than a default component.  Adapt `.component` to
// the default export the lazy loader expects.
const fromBarModule =
  (load: () => Promise<{ barModule: { component: ComponentType } }>): ModuleLoader =>
  () =>
    load().then((m) => ({ default: m.barModule.component }))

// All known modules — keyed by config name, value is the lazy import.
// Add third-party or user modules here by extending this map.
const MODULE_REGISTRY: Record<string, ModuleLoader> = {
  clock: () => import('./modules/clock'),
  workspaces: () => import('./modules/workspaces'),
  'window-title': () => import('./modules/window-title'),
  'power-menu': () => import('./modules/power-menu'),
  audio: fromBarModule(() => import('./modules/audio')),
  battery: fromBarModule(() => import('./modules/battery')),
  brightness: fromBarModule(() => import('./modules/brightness')),
  network: fromBarModule(() => import('./modules/network')),
  tray: fromBarModule(() => import('./modules/tray')),
}

/**
 * Resolve a module name to a lazily-loaded React component.
 * Returns `null` for unknown module names.
 */
export function resolveModule(name: string): ComponentType | null {
  const factory = MODULE_REGISTRY[name]
  if (!factory) return null
  return lazy(factory)
}

export const KNOWN_MODULES = Object.keys(MODULE_REGISTRY)
