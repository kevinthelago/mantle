import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useEffect, useState } from 'react'

export interface BrightnessSnapshot {
  available: boolean
  device: string
  brightness: number
  maxBrightness: number
  percentage: number
}

const UNAVAILABLE: BrightnessSnapshot = {
  available: false,
  device: '',
  brightness: 0,
  maxBrightness: 1,
  percentage: 0,
}

export function useBrightness(): {
  snap: BrightnessSnapshot
  setBrightness: (fraction: number) => Promise<void>
} {
  const [snap, setSnap] = useState<BrightnessSnapshot>(UNAVAILABLE)

  useEffect(() => {
    let unlisten: UnlistenFn | undefined

    invoke<BrightnessSnapshot>('get_brightness_snapshot')
      .then(setSnap)
      .catch(() => setSnap(UNAVAILABLE))

    listen<BrightnessSnapshot>('brightness_update', (e) => setSnap(e.payload)).then((fn) => {
      unlisten = fn
    })

    return () => {
      unlisten?.()
    }
  }, [])

  async function setBrightness(fraction: number): Promise<void> {
    await invoke('set_brightness', { fraction: Math.max(0, Math.min(1, fraction)) })
  }

  return { snap, setBrightness }
}

export function brightnessIcon(pct: number): string {
  if (pct >= 0.67) return 'display-brightness-high'
  if (pct >= 0.34) return 'display-brightness-medium'
  return 'display-brightness-low'
}
