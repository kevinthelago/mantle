import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useEffect, useState } from 'react'

export type AudioIcon = 'muted' | 'low' | 'medium' | 'high'

export interface AudioSnapshot {
  available: boolean
  volume: number
  muted: boolean
  sinkName: string | null
  icon: AudioIcon
}

const UNAVAILABLE: AudioSnapshot = {
  available: false,
  volume: 0,
  muted: false,
  sinkName: null,
  icon: 'muted',
}

export function useAudio(): {
  snap: AudioSnapshot
  setVolume: (v: number) => Promise<void>
  setMute: (m: boolean) => Promise<void>
  toggleMute: () => Promise<void>
} {
  const [snap, setSnap] = useState<AudioSnapshot>(UNAVAILABLE)

  useEffect(() => {
    let unlisten: UnlistenFn | undefined

    invoke<AudioSnapshot>('get_audio_snapshot')
      .then(setSnap)
      .catch(() => setSnap(UNAVAILABLE))

    listen<AudioSnapshot>('audio_update', (e) => setSnap(e.payload)).then((fn) => {
      unlisten = fn
    })

    return () => {
      unlisten?.()
    }
  }, [])

  async function setVolume(volume: number): Promise<void> {
    await invoke('set_audio_volume', { volume: Math.max(0, Math.min(1, volume)) })
  }

  async function setMute(muted: boolean): Promise<void> {
    await invoke('set_audio_mute', { muted })
  }

  async function toggleMute(): Promise<void> {
    await setMute(!snap.muted)
  }

  return { snap, setVolume, setMute, toggleMute }
}

export function volumeIconName(icon: AudioIcon): string {
  switch (icon) {
    case 'muted':
      return 'audio-volume-muted'
    case 'low':
      return 'audio-volume-low'
    case 'medium':
      return 'audio-volume-medium'
    case 'high':
      return 'audio-volume-high'
  }
}
