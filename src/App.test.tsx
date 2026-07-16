import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

// Controls what getCurrentWebviewWindow().label returns per test.  Hoisted so
// the vi.mock factory (also hoisted) can close over it safely.
const win = vi.hoisted(() => ({ label: 'bar' }))
vi.mock('@tauri-apps/api/webviewWindow', () => ({
  getCurrentWebviewWindow: () => ({ label: win.label }),
}))

import App, { surfaceIdForLabel } from './App'
import { registerSurface, unregisterSurface } from './lib/surfaces'

const SURFACE_IDS = ['bar', 'launcher', 'widgets', 'notifications']

beforeEach(() => {
  win.label = 'bar'
  for (const id of SURFACE_IDS) {
    registerSurface({ id, component: () => <div data-testid={`surface-${id}`}>{id}</div> })
  }
})

afterEach(() => {
  for (const id of SURFACE_IDS) unregisterSurface(id)
})

describe('surfaceIdForLabel', () => {
  it('maps the plain bar label to the bar surface', () => {
    expect(surfaceIdForLabel('bar')).toBe('bar')
  })

  it('maps per-monitor bar labels to the bar surface', () => {
    expect(surfaceIdForLabel('bar-DP-1')).toBe('bar')
    expect(surfaceIdForLabel('bar-eDP-1')).toBe('bar')
  })

  it('passes non-bar labels through unchanged', () => {
    expect(surfaceIdForLabel('launcher')).toBe('launcher')
    expect(surfaceIdForLabel('widgets')).toBe('widgets')
    expect(surfaceIdForLabel('notifications')).toBe('notifications')
  })
})

describe('App surface router', () => {
  it.each([
    ['bar', 'surface-bar'],
    ['bar-DP-1', 'surface-bar'],
    ['launcher', 'surface-launcher'],
    ['widgets', 'surface-widgets'],
    ['notifications', 'surface-notifications'],
  ])('mounts the surface registered for window label "%s"', (label, testId) => {
    win.label = label
    render(<App />)
    expect(screen.getByTestId(testId)).toBeTruthy()
  })

  it('renders only the surface for the active window, not the others', () => {
    win.label = 'launcher'
    render(<App />)
    expect(screen.getByTestId('surface-launcher')).toBeTruthy()
    expect(screen.queryByTestId('surface-bar')).toBeNull()
    expect(screen.queryByTestId('surface-notifications')).toBeNull()
  })

  it('renders a diagnostic fallback when no surface matches the label', () => {
    win.label = 'mystery-window'
    render(<App />)
    expect(screen.getByText(/no surface registered/i)).toBeTruthy()
    expect(screen.getByText(/mystery-window/)).toBeTruthy()
  })

  it('passes defaultProps to the mounted surface', () => {
    unregisterSurface('launcher')
    registerSurface({
      id: 'launcher',
      component: ({ text }: { text?: string }) => <div>{text}</div>,
      defaultProps: { text: 'from-default-props' },
    })
    win.label = 'launcher'
    render(<App />)
    expect(screen.getByText('from-default-props')).toBeTruthy()
  })
})
