import React from 'react'

import { isConnected, networkIcon, signalBars, useNetwork } from './useNetwork'
import styles from './Network.module.css'

function SignalBars({ strength }: { strength: number | null }): React.ReactElement {
  const count = signalBars(strength)
  const heights = [4, 7, 10, 13]
  return (
    <span className={styles.bars} aria-hidden>
      {heights.map((h, i) => (
        <span
          key={i}
          className={styles.bar}
          style={{ height: h }}
          data-active={String(i < count)}
        />
      ))}
    </span>
  )
}

export function NetworkModule(): React.ReactElement | null {
  const snap = useNetwork()

  if (!snap.available) return null

  const connected = isConnected(snap.state)
  const icon = networkIcon(snap)
  const label =
    snap.connectionType === 'wifi'
      ? (snap.ssid ?? '…')
      : snap.connectionType === 'wired'
        ? (snap.interface ?? 'Wired')
        : snap.connectionType === 'vpn'
          ? 'VPN'
          : connected
            ? 'Connected'
            : 'Disconnected'

  const tooltip = [
    `State: ${snap.state}`,
    snap.ssid && `SSID: ${snap.ssid}`,
    snap.ip4Address && `IP: ${snap.ip4Address}`,
    snap.signalStrength !== null && `Signal: ${snap.signalStrength}%`,
  ]
    .filter(Boolean)
    .join('\n')

  return (
    <div className={styles.module} data-connected={String(connected)} title={tooltip}>
      <span className={styles.icon} aria-hidden>
        {icon}
      </span>
      {snap.connectionType === 'wifi' && connected && <SignalBars strength={snap.signalStrength} />}
      <span className={styles.label}>{label}</span>
    </div>
  )
}

export const barModule = {
  id: 'network',
  region: 'right' as const,
  order: 20,
  component: NetworkModule,
}
