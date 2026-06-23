// @vitest-environment node
import { describe, expect, it } from 'vitest'
import { parseActions } from './types'

describe('parseActions', () => {
  it('returns empty array for empty input', () => {
    expect(parseActions([])).toEqual([])
  })

  it('parses a single key-label pair', () => {
    expect(parseActions(['default', 'Open'])).toEqual([{ key: 'default', label: 'Open' }])
  })

  it('parses multiple key-label pairs', () => {
    expect(parseActions(['default', 'Open', 'close', 'Dismiss'])).toEqual([
      { key: 'default', label: 'Open' },
      { key: 'close', label: 'Dismiss' },
    ])
  })

  it('ignores a trailing key without a label', () => {
    expect(parseActions(['default', 'Open', 'orphan'])).toEqual([{ key: 'default', label: 'Open' }])
  })

  it('handles four actions', () => {
    const raw = ['a', 'Action A', 'b', 'Action B', 'c', 'Action C', 'd', 'Action D']
    const result = parseActions(raw)
    expect(result).toHaveLength(4)
    expect(result[3]).toEqual({ key: 'd', label: 'Action D' })
  })

  it('preserves whitespace in labels', () => {
    expect(parseActions(['k', '  spaces  '])).toEqual([{ key: 'k', label: '  spaces  ' }])
  })
})
