import { lazy, type ComponentType } from 'react'

// All known modules — keyed by config name, value is the lazy import.
// Add third-party or user modules here by extending this map.
const MODULE_REGISTRY: Record<string, () => Promise<{ default: ComponentType }>> = {
  clock: () => import('./modules/clock'),
  workspaces: () => import('./modules/workspaces'),
  'window-title': () => import('./modules/window-title'),
  'power-menu': () => import('./modules/power-menu'),
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
