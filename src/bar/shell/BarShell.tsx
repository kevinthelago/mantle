import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import BarRegion from './BarRegion'
import type { OutputConfig } from '../../types/config'
import './bar-shell.css'

/** Derives the output name from the Tauri window label ("bar-DP-1" → "DP-1"). */
export function outputFromLabel(label: string): string {
  return label.replace(/^bar-/, '') || '*'
}

/**
 * Root bar component.  One instance runs per physical output (per window).
 * Fetches per-output config from Rust and re-renders on hot-reload.
 */
export default function BarShell() {
  const windowLabel = getCurrentWindow().label
  const outputName = outputFromLabel(windowLabel)

  const [config, setConfig] = useState<OutputConfig | null>(null)
  const [error, setError] = useState<string | null>(null)

  const fetchConfig = useCallback(() => {
    invoke<OutputConfig>('get_output_config', { output: outputName })
      .then(setConfig)
      .catch((e) => setError(String(e)))
  }, [outputName])

  useEffect(() => {
    fetchConfig()

    // Hot-reload: backend emits "config-changed" (full Config) when the file
    // changes.  Re-fetch to let the server resolve the right OutputConfig.
    const unlisten = listen('config-changed', fetchConfig)

    return () => {
      unlisten.then((f) => f())
    }
  }, [fetchConfig])

  if (error) {
    return <div className="bar-shell bar-shell--error">{error}</div>
  }
  if (!config) return null

  return (
    <div
      className="bar-shell"
      style={
        {
          '--bar-height': `${config.height}px`,
          '--bar-background': config.theme.background,
          '--bar-foreground': config.theme.foreground,
          '--bar-opacity': config.opacity,
          '--bar-border-radius': `${config.theme.border_radius}px`,
          '--bar-font': config.theme.font,
          '--bar-font-size': `${config.theme.font_size}px`,
        } as React.CSSProperties
      }
    >
      <BarRegion modules={config.left.modules} position="left" />
      <BarRegion modules={config.center.modules} position="center" />
      <BarRegion modules={config.right.modules} position="right" />
    </div>
  )
}
