export type WidgetId = 'calendar' | 'system-monitor' | 'media-controls'

export type AnchorPosition = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' | 'center'

export interface WidgetConfig {
  id: WidgetId
  anchor: AnchorPosition
  visible: boolean
}

export interface WidgetLayerConfig {
  widgets: WidgetConfig[]
}

// --- Metrics ---

export interface SystemMetrics {
  cpu_percent: number
  memory_used_mb: number
  memory_total_mb: number
  memory_percent: number
  net_rx_bytes: number
  net_tx_bytes: number
}

// --- MPRIS ---

export type PlaybackStatus = 'Playing' | 'Paused' | 'Stopped'

export interface PlayerMetadata {
  title?: string
  artist?: string
  album?: string
  art_url?: string
  length_us?: number
}

export interface PlayerState {
  player_name: string
  playback_status: PlaybackStatus
  position_us: number
  volume: number
  metadata: PlayerMetadata
  can_next: boolean
  can_prev: boolean
  can_pause: boolean
}
