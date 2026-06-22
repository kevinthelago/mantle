import React from 'react'

import { batteryIcon, formatTimeRemaining, useBattery } from './useBattery'
import styles from './Battery.module.css'

export function BatteryModule(): React.ReactElement | null {
  const snap = useBattery()

  if (!snap.available || !snap.present) return null

  const isLow = snap.percentage < 20 && snap.state === 'discharging'
  const icon = batteryIcon(snap.state, snap.percentage)
  const timeStr =
    snap.state === 'charging'
      ? formatTimeRemaining(snap.timeToFull)
      : formatTimeRemaining(snap.timeToEmpty)

  return (
    <div
      className={styles.module}
      data-state={snap.state}
      data-low={String(isLow)}
      title={`Battery: ${Math.round(snap.percentage)}% — ${snap.state}`}
    >
      <span className={styles.icon} aria-hidden>
        {/* icon-name resolved by the design system icon component at merge */}
        {icon}
      </span>
      <span className={styles.percentage}>{Math.round(snap.percentage)}%</span>
      {timeStr && <span className={styles.time}>{timeStr}</span>}
    </div>
  )
}

/** Bar-shell module registration contract. */
export const barModule = {
  id: 'battery',
  region: 'right' as const,
  order: 30,
  component: BatteryModule,
}
