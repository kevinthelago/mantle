import { describe, it, expect } from 'vitest'
import { renderToString } from 'react-dom/server'
import { createElement } from 'react'
import { Badge } from './Badge.js'

describe('Badge', () => {
  it('renders as a span element', () => {
    const html = renderToString(createElement(Badge, null, 'Status'))
    expect(html).toContain('<span')
    expect(html).toContain('Status')
  })

  it('renders all variant classes', () => {
    for (const variant of ['default', 'brand', 'success', 'error', 'warning', 'info'] as const) {
      const html = renderToString(createElement(Badge, { variant }, variant))
      expect(html).toContain(variant)
    }
  })

  it('renders status dot when dot=true', () => {
    const html = renderToString(createElement(Badge, { dot: true }, 'Online'))
    // dot span should be aria-hidden
    expect(html).toContain('aria-hidden="true"')
  })

  it('does not render dot span when dot=false', () => {
    const html = renderToString(createElement(Badge, { dot: false }, 'Tag'))
    expect(html).not.toContain('aria-hidden')
  })
})
