import { act, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { WidgetLayer } from '../WidgetLayer'
import type { WidgetLayerConfig } from '../types'

vi.mock('../registry', () => ({
  WIDGET_REGISTRY: {
    calendar: () => <div data-testid="calendar-widget" />,
    'system-monitor': () => <div data-testid="system-monitor-widget" />,
    'media-controls': () => <div data-testid="media-controls-widget" />,
  },
}))

const CONFIG: WidgetLayerConfig = {
  widgets: [
    { id: 'calendar', anchor: 'top-right', visible: true },
    { id: 'system-monitor', anchor: 'top-right', visible: true },
    { id: 'media-controls', anchor: 'bottom-right', visible: false },
  ],
}

let toggleCallback: (() => void) | undefined

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, cb: () => void) => {
    if (event === 'widget-layer:toggle') toggleCallback = cb
    return Promise.resolve(() => {})
  }),
}))

describe('WidgetLayer', () => {
  let invokeMock: ReturnType<typeof vi.fn>

  beforeEach(async () => {
    toggleCallback = undefined
    const mod = await import('@tauri-apps/api/core')
    invokeMock = mod.invoke as ReturnType<typeof vi.fn>
    invokeMock.mockResolvedValue(CONFIG)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders visible widgets from the loaded config', async () => {
    render(<WidgetLayer />)
    await screen.findByTestId('calendar-widget')
    screen.getByTestId('system-monitor-widget')
  })

  it('omits widgets with visible: false', async () => {
    render(<WidgetLayer />)
    await screen.findByTestId('calendar-widget')
    expect(screen.queryByTestId('media-controls-widget')).toBeNull()
  })

  it('hides the entire layer when the toggle event fires', async () => {
    const { container } = render(<WidgetLayer />)
    await screen.findByTestId('calendar-widget')
    expect(toggleCallback).toBeDefined()
    act(() => {
      toggleCallback!()
    })
    expect(container.firstChild).toBeNull()
  })

  it('falls back to the built-in default config when the backend call fails', async () => {
    invokeMock.mockRejectedValue(new Error('backend unavailable'))
    render(<WidgetLayer />)
    // All three default widgets are visible
    await screen.findByTestId('calendar-widget')
    screen.getByTestId('system-monitor-widget')
    screen.getByTestId('media-controls-widget')
  })
})
