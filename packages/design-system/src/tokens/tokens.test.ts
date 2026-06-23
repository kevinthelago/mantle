import { describe, it, expect } from 'vitest'
import { generateTokensCSS, generateTailwindPreset } from './generate.js'
import { palette, typeScale, spacing, radius, shadow, transition, zIndex, semanticDark, semanticLight } from './tokens.js'

describe('token exports', () => {
  it('exports palette with expected color groups', () => {
    expect(palette.neutral[900]).toBe('#1a1a1a')
    expect(palette.orange[500]).toBe('#e8956d')
    expect(palette.green[400]).toBe('#4ade80')
    expect(palette.red[400]).toBe('#f87171')
  })

  it('exports typeScale with font families', () => {
    expect(typeScale.fontFamily.sans).toContain('Inter')
    expect(typeScale.fontFamily.mono).toContain('JetBrains Mono')
  })

  it('exports spacing with pixel values', () => {
    expect(spacing['4']).toBe('16px')
    expect(spacing['0']).toBe('0px')
  })

  it('exports radius tokens', () => {
    expect(radius.none).toBe('0px')
    expect(radius.full).toBe('9999px')
  })

  it('exports semanticDark with nested color structure', () => {
    expect(semanticDark.color.bg.base).toBe(palette.neutral[900])
    expect(semanticDark.color.text.primary).toBe(palette.neutral[50])
    expect(semanticDark.color.brand.default).toBe(palette.orange[500])
  })

  it('exports semanticLight with inverted neutral mapping', () => {
    expect(semanticLight.color.bg.base).toBe(palette.neutral[0])
    expect(semanticLight.color.text.primary).toBe(palette.neutral[900])
  })
})

describe('generateTokensCSS', () => {
  const css = generateTokensCSS()

  it('contains :root block with dark-default colors', () => {
    expect(css).toContain(':root {')
    expect(css).toContain('--color-bg-base: #1a1a1a')
    expect(css).toContain('--color-text-primary: #f5f5f5')
  })

  it('contains [data-theme="light"] overrides', () => {
    expect(css).toMatch(/\[data-theme=['"]light['"]\]/)
    expect(css).toContain('--color-bg-base: #ffffff')
  })

  it('contains typography custom properties', () => {
    expect(css).toContain('--font-size-base:')
    expect(css).toContain('--font-weight-medium:')
    expect(css).toContain('--line-height-normal:')
  })

  it('contains spacing custom properties', () => {
    expect(css).toContain('--spacing-4:')
  })

  it('contains radius custom properties', () => {
    expect(css).toContain('--radius-full: 9999px')
  })

  it('contains shadow custom properties', () => {
    expect(css).toContain('--shadow-none: none')
  })

  it('contains z-index custom properties', () => {
    expect(css).toContain('--z-tooltip:')
  })

  it('contains transition custom properties', () => {
    expect(css).toContain('--duration-fast:')
    expect(css).toContain('--easing-ease-out:')
  })

  it('only contains color overrides in light block, not typography', () => {
    const lightStart = css.indexOf('[data-theme')
    const lightBlock = css.slice(lightStart)
    expect(lightBlock).not.toContain('--font-size-')
    expect(lightBlock).not.toContain('--spacing-')
    expect(lightBlock).toContain('--color-bg-base: #ffffff')
  })
})

describe('generateTailwindPreset', () => {
  const preset = generateTailwindPreset() as {
    theme: { extend: Record<string, Record<string, unknown>> }
  }

  it('has theme.extend structure', () => {
    expect(preset.theme).toBeDefined()
    expect(preset.theme.extend).toBeDefined()
  })

  it('maps bg colors to CSS vars', () => {
    const { colors } = preset.theme.extend as { colors: Record<string, Record<string, string>> }
    expect(colors['bg']!['base']).toBe('var(--color-bg-base)')
    expect(colors['bg']!['inverted']).toBe('var(--color-bg-inverted)')
  })

  it('maps brand colors including hover/active', () => {
    const { colors } = preset.theme.extend as { colors: Record<string, Record<string, string>> }
    expect(colors['brand']!['hover']).toBe('var(--color-brand-hover)')
  })

  it('maps font sizes to CSS vars', () => {
    const { fontSize } = preset.theme.extend as { fontSize: Record<string, string> }
    expect(fontSize['base']).toBe('var(--font-size-base)')
    expect(fontSize['2xl']).toBe('var(--font-size-2xl)')
  })

  it('maps border radius keys', () => {
    const { borderRadius } = preset.theme.extend as { borderRadius: Record<string, string> }
    expect(borderRadius['full']).toBe('var(--radius-full)')
    expect(borderRadius['md']).toBe('var(--radius-md)')
  })

  it('maps spacing keys with dot-notation', () => {
    const { spacing: sp } = preset.theme.extend as { spacing: Record<string, string> }
    // spacing['4'] in tokens → '4' key in preset (or '0.5' for '0-5')
    expect(sp['4']).toBe('var(--spacing-4)')
    expect(sp['0.5']).toBe('var(--spacing-0-5)')
  })

  it('maps transition durations', () => {
    const { transitionDuration } = preset.theme.extend as {
      transitionDuration: Record<string, string>
    }
    expect(transitionDuration['fast']).toBe('var(--duration-fast)')
    expect(transitionDuration['normal']).toBe('var(--duration-normal)')
  })

  it('maps z-index tokens', () => {
    const { zIndex: zi } = preset.theme.extend as { zIndex: Record<string, string> }
    expect(zi['tooltip']).toBe('var(--z-tooltip)')
    expect(zi['modal']).toBe('var(--z-modal)')
  })
})
