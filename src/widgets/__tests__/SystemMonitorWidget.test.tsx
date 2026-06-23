import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { SystemMonitorWidget } from '../system-monitor/SystemMonitorWidget'
import type { SystemMetrics } from '../types'

const METRICS: SystemMetrics = {
  cpu_percent: 42.5,
  memory_used_mb: 4096,
  memory_total_mb: 8192,
  memory_percent: 50.0,
  net_rx_bytes: 0,
  net_tx_bytes: 0,
}

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('SystemMonitorWidget', () => {
  let invokeMock: ReturnType<typeof vi.fn>

  beforeEach(async () => {
    const mod = await import('@tauri-apps/api/core')
    invokeMock = mod.invoke as ReturnType<typeof vi.fn>
    invokeMock.mockResolvedValue(METRICS)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('shows dash placeholders before the first sample resolves', () => {
    invokeMock.mockReturnValue(new Promise<SystemMetrics>(() => {}))
    render(<SystemMonitorWidget />)
    const dashes = screen.getAllByText('—')
    expect(dashes.length).toBeGreaterThanOrEqual(2)
  })

  it('shows CPU usage after the first sample', async () => {
    render(<SystemMonitorWidget />)
    await waitFor(() => screen.getByText('42.5%'))
  })

  it('shows memory usage percentage', async () => {
    render(<SystemMonitorWidget />)
    await waitFor(() => screen.getByText('50.0%'))
  })

  it('shows used and total memory in GiB', async () => {
    render(<SystemMonitorWidget />)
    // 4096 / 1024 = 4.0 GiB used, 8192 / 1024 = 8.0 GiB total
    await waitFor(() => screen.getByText(/4\.0 \/ 8\.0 GiB/))
  })

  it('calls get_system_metrics on mount', async () => {
    render(<SystemMonitorWidget />)
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith('get_system_metrics'))
  })
})
