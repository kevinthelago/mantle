import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useEffect, useState } from 'react'

export type NetworkState =
  | 'unknown'
  | 'asleep'
  | 'disconnected'
  | 'disconnecting'
  | 'connecting'
  | 'connectedLocal'
  | 'connectedSite'
  | 'connectedGlobal'

export type ConnectivityState = 'unknown' | 'none' | 'portal' | 'limited' | 'full'

export type ConnectionType = 'none' | 'wired' | 'wifi' | 'vpn' | { other: string }

export interface NetworkSnapshot {
  available: boolean
  state: NetworkState
  connectivity: ConnectivityState
  connectionType: ConnectionType
  ssid: string | null
  signalStrength: number | null
  interface: string | null
  ip4Address: string | null
}

const UNAVAILABLE: NetworkSnapshot = {
  available: false,
  state: 'unknown',
  connectivity: 'unknown',
  connectionType: 'none',
  ssid: null,
  signalStrength: null,
  interface: null,
  ip4Address: null,
}

export function useNetwork(): NetworkSnapshot {
  const [snap, setSnap] = useState<NetworkSnapshot>(UNAVAILABLE)

  useEffect(() => {
    let unlisten: UnlistenFn | undefined

    invoke<NetworkSnapshot>('get_network_snapshot')
      .then(setSnap)
      .catch(() => setSnap(UNAVAILABLE))

    listen<NetworkSnapshot>('network_update', (e) => setSnap(e.payload)).then((fn) => {
      unlisten = fn
    })

    return () => {
      unlisten?.()
    }
  }, [])

  return snap
}

export function isConnected(state: NetworkState): boolean {
  return state === 'connectedLocal' || state === 'connectedSite' || state === 'connectedGlobal'
}

export function networkIcon(snap: NetworkSnapshot): string {
  if (!snap.available || !isConnected(snap.state)) return 'network-offline'
  if (snap.connectionType === 'wifi') {
    const strength = snap.signalStrength ?? 0
    if (strength >= 75) return 'network-wireless-signal-excellent'
    if (strength >= 50) return 'network-wireless-signal-good'
    if (strength >= 25) return 'network-wireless-signal-ok'
    return 'network-wireless-signal-weak'
  }
  if (snap.connectionType === 'wired') return 'network-wired'
  if (snap.connectionType === 'vpn') return 'network-vpn'
  return 'network-transmit-receive'
}

/** Signal strength as 0–4 bars (for rendering). */
export function signalBars(strength: number | null): 0 | 1 | 2 | 3 | 4 {
  if (strength === null) return 0
  if (strength >= 75) return 4
  if (strength >= 50) return 3
  if (strength >= 25) return 2
  if (strength > 0) return 1
  return 0
}
