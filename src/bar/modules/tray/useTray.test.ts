import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useTray } from './useTray'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}))

const SAMPLE_ITEM = {
  key: 'org.example.App/StatusNotifierItem',
  id: 'app',
  title: 'Example App',
  status: 'active' as const,
  category: 'ApplicationStatus',
  icon: null,
  tooltip: 'Example tooltip',
  menuPath: null,
  menu: null,
}

describe('useTray', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('initializes with empty items', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValue({ items: [] })

    const { result } = renderHook(() => useTray())
    await waitFor(() => expect(result.current.items).toHaveLength(0))
  })

  it('populates items from get_tray_items', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValue({ items: [SAMPLE_ITEM] })

    const { result } = renderHook(() => useTray())
    await waitFor(() => expect(result.current.items).toHaveLength(1))
    expect(result.current.items[0].title).toBe('Example App')
  })

  it('exposes activate, sendMenuEvent, and refreshMenu as functions', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValue({ items: [] })

    const { result } = renderHook(() => useTray())
    expect(typeof result.current.activate).toBe('function')
    expect(typeof result.current.sendMenuEvent).toBe('function')
    expect(typeof result.current.refreshMenu).toBe('function')
  })

  it('activate calls tray_item_activate command', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValue({ items: [] })

    const { result } = renderHook(() => useTray())
    await act(async () => {
      await result.current.activate('some-key', 100, 200)
    })
    expect(invoke).toHaveBeenCalledWith('tray_item_activate', {
      key: 'some-key',
      x: 100,
      y: 200,
    })
  })

  it('sendMenuEvent calls tray_menu_event command', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValue({ items: [] })

    const { result } = renderHook(() => useTray())
    await act(async () => {
      await result.current.sendMenuEvent('some-key', 42, 'clicked')
    })
    expect(invoke).toHaveBeenCalledWith('tray_menu_event', {
      key: 'some-key',
      itemId: 42,
      event: 'clicked',
    })
  })
})
