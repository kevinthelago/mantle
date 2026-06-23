import { describe, it, expect } from 'vitest'
import { renderToString } from 'react-dom/server'
import { createElement } from 'react'
import { Button } from './Button.js'

describe('Button', () => {
  it('renders with default secondary variant', () => {
    const html = renderToString(createElement(Button, null, 'Click me'))
    expect(html).toContain('<button')
    expect(html).toContain('Click me')
  })

  it('applies primary variant class', () => {
    const html = renderToString(createElement(Button, { variant: 'primary' }, 'Save'))
    expect(html).toContain('primary')
  })

  it('marks disabled buttons with aria-disabled', () => {
    const html = renderToString(createElement(Button, { disabled: true }, 'Disabled'))
    expect(html).toContain('aria-disabled="true"')
    expect(html).toContain('disabled')
  })

  it('marks loading buttons with aria-busy and aria-disabled', () => {
    const html = renderToString(createElement(Button, { loading: true }, 'Saving…'))
    expect(html).toContain('aria-busy="true"')
    expect(html).toContain('aria-disabled="true"')
    expect(html).toContain('disabled')
  })

  it('renders loading spinner SVG when loading', () => {
    const html = renderToString(createElement(Button, { loading: true }, 'Loading'))
    expect(html).toContain('<svg')
    expect(html).toContain('aria-hidden="true"')
  })

  it('renders startIcon before children', () => {
    const icon = createElement('span', { 'data-testid': 'start' }, '→')
    const html = renderToString(createElement(Button, { startIcon: icon }, 'Go'))
    const iconIdx = html.indexOf('data-testid="start"')
    const labelIdx = html.indexOf('>Go<')
    expect(iconIdx).toBeGreaterThan(-1)
    expect(iconIdx).toBeLessThan(labelIdx)
  })

  it('renders endIcon after children', () => {
    const icon = createElement('span', { 'data-testid': 'end' }, '→')
    const html = renderToString(createElement(Button, { endIcon: icon }, 'Go'))
    const labelIdx = html.indexOf('>Go<')
    const iconIdx = html.indexOf('data-testid="end"')
    expect(iconIdx).toBeGreaterThan(labelIdx)
  })

  it('renders all size variants without error', () => {
    for (const size of ['sm', 'md', 'lg'] as const) {
      const html = renderToString(createElement(Button, { size }, size))
      expect(html).toContain(size)
    }
  })

  it('renders destructive variant', () => {
    const html = renderToString(createElement(Button, { variant: 'destructive' }, 'Delete'))
    expect(html).toContain('destructive')
  })
})
