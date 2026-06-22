import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useEffect, useState } from 'react'

export type BatteryState =
  | 'unknown'
  | 'charging'
  | 'discharging'
  | 'empty'
  | 'fullyCharged'
  | 'pendingCharge'
  | 'pendingDischarge'

export interface BatterySnapshot {
  available: boolean
  present: boolean
  percentage: number
  state: BatteryState
  timeToEmpty: number
  timeToFull: number
}

const UNAVAILABLE: BatterySnapshot = {
  available: false,
  present: false,
  percentage: 0,
  state: 'unknown',
  timeToEmpty: 0,
  timeToFull: 0,
}

export function useBattery(): BatterySnapshot {
  const [snap, setSnap] = useState<BatterySnapshot>(UNAVAILABLE)

  useEffect(() => {
    let unlisten: UnlistenFn | undefined

    invoke<BatterySnapshot>('get_battery_snapshot')
      .then(setSnap)
      .catch(() => setSnap(UNAVAILABLE))

    listen<BatterySnapshot>('battery_update', (e) => setSnap(e.payload)).then((fn) => {
      unlisten = fn
    })

    return () => {
      unlisten?.()
    }
  }, [])

  return snap
}

export function formatTimeRemaining(seconds: number): string {
  if (seconds <= 0) return ''
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

export function batteryIcon(state: BatteryState, pct: number): string {
  if (state === 'charging' || state === 'pendingCharge') return 'battery-charging'
  if (pct >= 90) return 'battery-full'
  if (pct >= 60) return 'battery-good'
  if (pct >= 30) return 'battery-low'
  return 'battery-caution'
}
