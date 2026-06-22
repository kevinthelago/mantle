import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useEffect, useState } from 'react'
import type { PlayerState } from '../types'
import styles from './MediaControlsWidget.module.css'

type Action = 'mpris_play_pause' | 'mpris_next' | 'mpris_prev'

async function dispatch(action: Action) {
  try {
    await invoke(action)
  } catch (e) {
    console.error('[media-controls]', action, e)
  }
}

export function MediaControlsWidget() {
  const [player, setPlayer] = useState<PlayerState | null | undefined>(undefined)

  useEffect(() => {
    invoke<PlayerState | null>('get_player_state')
      .then(setPlayer)
      .catch(() => setPlayer(null))

    const unlisten = listen<PlayerState | null>('mpris:state-changed', (e) => {
      setPlayer(e.payload)
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  // Still loading
  if (player === undefined) return null

  if (player === null) {
    return (
      <div className={styles.widget}>
        <span className={styles.noPlayer}>No media playing</span>
      </div>
    )
  }

  const { metadata, playback_status, can_next, can_prev, can_pause } = player
  const isPlaying = playback_status === 'Playing'

  return (
    <div className={styles.widget}>
      {metadata.art_url && (
        <img className={styles.art} src={metadata.art_url} alt="" aria-hidden="true" />
      )}
      <div className={styles.info}>
        <span className={styles.title} title={metadata.title}>
          {metadata.title ?? 'Unknown'}
        </span>
        <span className={styles.artist} title={metadata.artist}>
          {metadata.artist ?? 'Unknown artist'}
        </span>
        <span className={styles.player}>{player.player_name}</span>
      </div>
      <div className={styles.controls}>
        <button
          className={styles.btn}
          onClick={() => dispatch('mpris_prev')}
          disabled={!can_prev}
          aria-label="Previous track"
        >
          ⏮
        </button>
        <button
          className={`${styles.btn} ${styles.playPause}`}
          onClick={() => dispatch('mpris_play_pause')}
          disabled={!can_pause && playback_status === 'Stopped'}
          aria-label={isPlaying ? 'Pause' : 'Play'}
        >
          {isPlaying ? '⏸' : '▶'}
        </button>
        <button
          className={styles.btn}
          onClick={() => dispatch('mpris_next')}
          disabled={!can_next}
          aria-label="Next track"
        >
          ⏭
        </button>
      </div>
    </div>
  )
}
