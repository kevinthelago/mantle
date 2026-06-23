import { describe, it, expect } from 'vitest'
import { brightnessIcon } from './useBrightness'

describe('brightnessIcon', () => {
  it('returns high icon at 67%+', () => {
    expect(brightnessIcon(0.67)).toBe('display-brightness-high')
    expect(brightnessIcon(1.0)).toBe('display-brightness-high')
  })

  it('returns medium icon at 34–66%', () => {
    expect(brightnessIcon(0.34)).toBe('display-brightness-medium')
    expect(brightnessIcon(0.5)).toBe('display-brightness-medium')
    expect(brightnessIcon(0.66)).toBe('display-brightness-medium')
  })

  it('returns low icon below 34%', () => {
    expect(brightnessIcon(0.0)).toBe('display-brightness-low')
    expect(brightnessIcon(0.33)).toBe('display-brightness-low')
  })
})
