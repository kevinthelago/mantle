import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useRef, useState } from 'react'
import type { SystemMetrics } from '../types'
import { Sparkline } from './Sparkline'
import styles from './SystemMonitorWidget.module.css'

const HISTORY_CAP = 60
const POLL_INTERVAL_MS = 2_000

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B/s`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB/s`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB/s`
}

export function SystemMonitorWidget() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  const [cpuHist, setCpuHist] = useState<number[]>([])
  const [memHist, setMemHist] = useState<number[]>([])
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const prevNetRef = useRef<{ rx: number; tx: number } | null>(null)
  const [netRate, setNetRate] = useState<{ rx: number; tx: number }>({ rx: 0, tx: 0 })

  const sample = useCallback(async () => {
    try {
      const m = await invoke<SystemMetrics>('get_system_metrics')
      setMetrics(m)
      setCpuHist((h) => [...h.slice(-(HISTORY_CAP - 1)), m.cpu_percent])
      setMemHist((h) => [...h.slice(-(HISTORY_CAP - 1)), m.memory_percent])

      // Compute per-interval byte rate from cumulative totals
      if (prevNetRef.current) {
        const rxDelta = Math.max(0, m.net_rx_bytes - prevNetRef.current.rx)
        const txDelta = Math.max(0, m.net_tx_bytes - prevNetRef.current.tx)
        setNetRate({
          rx: rxDelta / (POLL_INTERVAL_MS / 1000),
          tx: txDelta / (POLL_INTERVAL_MS / 1000),
        })
      }
      prevNetRef.current = { rx: m.net_rx_bytes, tx: m.net_tx_bytes }
    } catch {
      // Backend not available in dev/storybook — silently ignore
    }
  }, [])

  useEffect(() => {
    // Visibility-gated: stop polling when the document is hidden
    function onVisibilityChange() {
      if (document.hidden) {
        if (intervalRef.current !== null) {
          clearInterval(intervalRef.current)
          intervalRef.current = null
        }
      } else {
        if (intervalRef.current === null) {
          sample()
          intervalRef.current = setInterval(sample, POLL_INTERVAL_MS)
        }
      }
    }

    document.addEventListener('visibilitychange', onVisibilityChange)
    sample()
    intervalRef.current = setInterval(sample, POLL_INTERVAL_MS)

    return () => {
      document.removeEventListener('visibilitychange', onVisibilityChange)
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current)
      }
    }
  }, [sample])

  const cpuPct = metrics?.cpu_percent ?? 0
  const memPct = metrics?.memory_percent ?? 0

  return (
    <div className={styles.widget}>
      <Row
        label="CPU"
        value={`${cpuPct.toFixed(1)}%`}
        history={cpuHist}
        color="rgba(100,200,255,0.9)"
        ready={metrics !== null}
      />
      <Row
        label="MEM"
        value={`${memPct.toFixed(1)}%`}
        history={memHist}
        color="rgba(180,120,255,0.9)"
        ready={metrics !== null}
      />
      {metrics && (
        <div className={styles.detail}>
          <span>
            {(metrics.memory_used_mb / 1024).toFixed(1)} /{' '}
            {(metrics.memory_total_mb / 1024).toFixed(1)} GiB
          </span>
          <span className={styles.net}>
            ↓{formatBytes(netRate.rx)} ↑{formatBytes(netRate.tx)}
          </span>
        </div>
      )}
    </div>
  )
}

interface RowProps {
  label: string
  value: string
  history: number[]
  color: string
  ready: boolean
}

function Row({ label, value, history, color, ready }: RowProps) {
  return (
    <div className={styles.row}>
      <span className={styles.label}>{label}</span>
      <Sparkline values={history} max={100} className={styles.spark} color={color} />
      <span className={styles.value} style={{ color: ready ? color : undefined }}>
        {ready ? value : '—'}
      </span>
    </div>
  )
}
