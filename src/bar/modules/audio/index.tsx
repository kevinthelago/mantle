import React, { useCallback } from 'react'

import { useAudio, volumeIconName } from './useAudio'
import styles from './Audio.module.css'

export function AudioModule(): React.ReactElement | null {
  const { snap, setVolume, toggleMute } = useAudio()

  const handleClick = useCallback(() => {
    toggleMute().catch(console.error)
  }, [toggleMute])

  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      e.preventDefault()
      const delta = -e.deltaY / 1000
      setVolume(snap.volume + delta).catch(console.error)
    },
    [snap.volume, setVolume],
  )

  if (!snap.available) return null

  const icon = volumeIconName(snap.icon)
  const pct = Math.round(snap.volume * 100)

  return (
    <div
      className={styles.module}
      data-muted={String(snap.muted)}
      title={snap.sinkName ? `Volume: ${pct}% — ${snap.sinkName}` : `Volume: ${pct}%`}
      onClick={handleClick}
      onWheel={handleWheel}
      role="button"
      tabIndex={0}
      aria-label={`Volume ${pct}%${snap.muted ? ' (muted)' : ''}`}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') handleClick()
        if (e.key === 'ArrowUp') setVolume(snap.volume + 0.05).catch(console.error)
        if (e.key === 'ArrowDown') setVolume(snap.volume - 0.05).catch(console.error)
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
  id: 'audio',
  region: 'right' as const,
  order: 10,
  component: AudioModule,
}
