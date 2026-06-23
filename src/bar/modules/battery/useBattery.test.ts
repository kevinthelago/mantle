import { describe, it, expect } from 'vitest'
import { batteryIcon, formatTimeRemaining } from './useBattery'

describe('formatTimeRemaining', () => {
  it('returns empty string for zero or negative', () => {
    expect(formatTimeRemaining(0)).toBe('')
    expect(formatTimeRemaining(-10)).toBe('')
  })

  it('formats sub-hour durations as minutes', () => {
    expect(formatTimeRemaining(60)).toBe('1m')
    expect(formatTimeRemaining(90)).toBe('1m')
    expect(formatTimeRemaining(3599)).toBe('59m')
  })

  it('formats hour+ durations with hours and minutes', () => {
    expect(formatTimeRemaining(3600)).toBe('1h 0m')
    expect(formatTimeRemaining(5400)).toBe('1h 30m')
    expect(formatTimeRemaining(9000)).toBe('2h 30m')
  })
})

describe('batteryIcon', () => {
  it('returns charging icon when charging or pending charge', () => {
    expect(batteryIcon('charging', 50)).toBe('battery-charging')
    expect(batteryIcon('pendingCharge', 80)).toBe('battery-charging')
  })

  it('returns full icon at 90%+', () => {
    expect(batteryIcon('discharging', 90)).toBe('battery-full')
    expect(batteryIcon('discharging', 100)).toBe('battery-full')
  })

  it('returns good icon at 60–89%', () => {
    expect(batteryIcon('discharging', 60)).toBe('battery-good')
    expect(batteryIcon('discharging', 89)).toBe('battery-good')
  })

  it('returns low icon at 30–59%', () => {
    expect(batteryIcon('discharging', 30)).toBe('battery-low')
    expect(batteryIcon('discharging', 59)).toBe('battery-low')
  })

  it('returns caution icon below 30%', () => {
    expect(batteryIcon('discharging', 0)).toBe('battery-caution')
    expect(batteryIcon('discharging', 29)).toBe('battery-caution')
  })

  it('uses discharging icon logic for unknown/empty states', () => {
    expect(batteryIcon('unknown', 95)).toBe('battery-full')
    expect(batteryIcon('empty', 5)).toBe('battery-caution')
  })
})
