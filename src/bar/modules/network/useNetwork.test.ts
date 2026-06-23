import { describe, it, expect } from 'vitest'
import { isConnected, networkIcon, signalBars } from './useNetwork'
import type { NetworkSnapshot } from './useNetwork'

const BASE: NetworkSnapshot = {
  available: true,
  state: 'connectedGlobal',
  connectivity: 'full',
  connectionType: 'wired',
  ssid: null,
  signalStrength: null,
  interface: 'eth0',
  ip4Address: '192.168.1.1',
}

describe('isConnected', () => {
  it('returns true for connected states', () => {
    expect(isConnected('connectedLocal')).toBe(true)
    expect(isConnected('connectedSite')).toBe(true)
    expect(isConnected('connectedGlobal')).toBe(true)
  })

  it('returns false for other states', () => {
    expect(isConnected('disconnected')).toBe(false)
    expect(isConnected('disconnecting')).toBe(false)
    expect(isConnected('connecting')).toBe(false)
    expect(isConnected('asleep')).toBe(false)
    expect(isConnected('unknown')).toBe(false)
  })
})

describe('networkIcon', () => {
  it('returns offline icon when unavailable', () => {
    expect(networkIcon({ ...BASE, available: false })).toBe('network-offline')
  })

  it('returns offline icon when disconnected', () => {
    expect(networkIcon({ ...BASE, state: 'disconnected' })).toBe('network-offline')
  })

  it('returns wired icon for wired connection', () => {
    expect(networkIcon({ ...BASE, connectionType: 'wired' })).toBe('network-wired')
  })

  it('returns vpn icon for vpn connection', () => {
    expect(networkIcon({ ...BASE, connectionType: 'vpn' })).toBe('network-vpn')
  })

  it('returns generic icon for unknown connection type', () => {
    expect(networkIcon({ ...BASE, connectionType: { other: 'gsm' } })).toBe(
      'network-transmit-receive',
    )
  })

  it('returns wifi icons based on signal strength', () => {
    const wifi = (strength: number) => ({
      ...BASE,
      connectionType: 'wifi' as const,
      signalStrength: strength,
    })
    expect(networkIcon(wifi(75))).toBe('network-wireless-signal-excellent')
    expect(networkIcon(wifi(50))).toBe('network-wireless-signal-good')
    expect(networkIcon(wifi(25))).toBe('network-wireless-signal-ok')
    expect(networkIcon(wifi(10))).toBe('network-wireless-signal-weak')
  })
})

describe('signalBars', () => {
  it('returns 0 for null', () => {
    expect(signalBars(null)).toBe(0)
  })

  it('returns 0 for zero strength', () => {
    expect(signalBars(0)).toBe(0)
  })

  it('returns 1 for weak signal (1–24)', () => {
    expect(signalBars(1)).toBe(1)
    expect(signalBars(24)).toBe(1)
  })

  it('returns 2 for ok signal (25–49)', () => {
    expect(signalBars(25)).toBe(2)
    expect(signalBars(49)).toBe(2)
  })

  it('returns 3 for good signal (50–74)', () => {
    expect(signalBars(50)).toBe(3)
    expect(signalBars(74)).toBe(3)
  })

  it('returns 4 for excellent signal (75+)', () => {
    expect(signalBars(75)).toBe(4)
    expect(signalBars(100)).toBe(4)
  })
})
