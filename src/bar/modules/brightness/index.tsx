import React, { useCallback } from 'react'

import { brightnessIcon, useBrightness } from './useBrightness'
import styles from './Brightness.module.css'

export function BrightnessModule(): React.ReactElement | null {
  const { snap, setBrightness } = useBrightness()

  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      e.preventDefault()
      const delta = -e.deltaY / 1000
      setBrightness(snap.percentage + delta).catch(console.error)
    },
    [snap.percentage, setBrightness],
  )

  if (!snap.available) return null

  const pct = Math.round(snap.percentage * 100)
  const icon = brightnessIcon(snap.percentage)

  return (
    <div
      className={styles.module}
      title={`Brightness: ${pct}% (${snap.device})`}
      onWheel={handleWheel}
      role="slider"
      aria-label={`Brightness ${pct}%`}
      aria-valuenow={pct}
      aria-valuemin={0}
      aria-valuemax={100}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'ArrowUp') setBrightness(snap.percentage + 0.05).catch(console.error)
        if (e.key === 'ArrowDown') setBrightness(snap.percentage - 0.05).catch(console.error)
      }}
    >
      <span className={styles.icon} aria-hidden>
        {icon}
      </span>
      <span className={styles.label}>{pct}%</span>
    </div>
  )
}

export const barModule = {
  id: 'brightness',
  region: 'right' as const,
  order: 15,
  component: BrightnessModule,
}
