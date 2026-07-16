import type { ComponentType } from 'react'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getSurface } from './lib/surfaces'

/**
 * Maps a Tauri window label to a registered surface id.
 *
 * Per-monitor bars are labeled `bar-<connector>` (e.g. "bar-DP-1"); they all
 * resolve to the single "bar" surface.  Every other window label maps to a
 * surface of the same name ("launcher", "widgets", "notifications").
 */
export function surfaceIdForLabel(label: string): string {
  return label === 'bar' || label.startsWith('bar-') ? 'bar' : label
}

/**
 * Shown when a window has no matching surface.  In a correctly configured build
 * every window label maps to a registered surface, so this is a diagnostic
 * fallback rather than an expected state — it keeps a stray window from
 * rendering a blank, silent page.
 */
function UnknownSurface({ label }: { label: string }) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100%',
        color: '#e0e0e0',
        fontFamily: 'monospace',
        fontSize: 12,
        opacity: 0.6,
        userSelect: 'none',
      }}
    >
      mantle: no surface registered for "{label}"
    </div>
  )
}

/**
 * Surface router — the single React root shared by every mantle window.
 *
 * Each window loads the same bundle; this component reads the current Tauri
 * window label and mounts the surface registered for it (bar / launcher /
 * widgets / notifications).  Surfaces self-register at import time; main.tsx
 * imports each entry module so the registry is populated before render.
 */
export default function App() {
  const label = getCurrentWebviewWindow().label
  const surface = getSurface(surfaceIdForLabel(label))

  if (!surface) return <UnknownSurface label={label} />

  const Surface = surface.component as ComponentType<Record<string, unknown>>
  return <Surface {...(surface.defaultProps ?? {})} />
}
