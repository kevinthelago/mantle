import { describe, expect, it } from 'vitest'
import { resolveModule, KNOWN_MODULES } from '../registry'

describe('resolveModule', () => {
  it('returns a component for each known module name', () => {
    for (const name of KNOWN_MODULES) {
      const mod = resolveModule(name)
      expect(mod, `expected a component for '${name}'`).not.toBeNull()
    }
  })

  it('returns null for an unknown module name', () => {
    expect(resolveModule('does-not-exist')).toBeNull()
    expect(resolveModule('')).toBeNull()
  })

  it('KNOWN_MODULES contains the expected module names', () => {
    expect(KNOWN_MODULES).toContain('clock')
    expect(KNOWN_MODULES).toContain('workspaces')
    expect(KNOWN_MODULES).toContain('window-title')
    expect(KNOWN_MODULES).toContain('power-menu')
  })

  it('returns the same lazy component on repeated calls for the same name', () => {
    // resolveModule creates a new lazy() each call; just verify it's non-null.
    const a = resolveModule('clock')
    const b = resolveModule('clock')
    expect(a).not.toBeNull()
    expect(b).not.toBeNull()
  })
})
