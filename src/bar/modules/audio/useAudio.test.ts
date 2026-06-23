import { describe, it, expect } from 'vitest'
import { volumeIconName } from './useAudio'

describe('volumeIconName', () => {
  it('maps muted to audio-volume-muted', () => {
    expect(volumeIconName('muted')).toBe('audio-volume-muted')
  })

  it('maps low to audio-volume-low', () => {
    expect(volumeIconName('low')).toBe('audio-volume-low')
  })

  it('maps medium to audio-volume-medium', () => {
    expect(volumeIconName('medium')).toBe('audio-volume-medium')
  })

  it('maps high to audio-volume-high', () => {
    expect(volumeIconName('high')).toBe('audio-volume-high')
  })
})
