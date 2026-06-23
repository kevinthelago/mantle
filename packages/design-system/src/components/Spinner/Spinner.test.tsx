import { describe, it, expect } from 'vitest'
import { renderToString } from 'react-dom/server'
import { createElement } from 'react'
import { Spinner } from './Spinner.js'

describe('Spinner', () => {
  it('renders an SVG with role=status', () => {
    const html = renderToString(createElement(Spinner, null))
    expect(html).toContain('<svg')
    expect(html).toContain('role="status"')
  })

  it('has accessible label', () => {
    const html = renderToString(createElement(Spinner, { label: 'Loading…' }))
    expect(html).toContain('aria-label="Loading…"')
  })

  it('uses custom label when provided', () => {
    const html = renderToString(createElement(Spinner, { label: 'Fetching data' }))
    expect(html).toContain('aria-label="Fetching data"')
  })

  it('renders all size variants', () => {
    for (const size of ['sm', 'md', 'lg', 'xl'] as const) {
      const html = renderToString(createElement(Spinner, { size }))
      expect(html).toContain(size)
    }
  })

  it('has track circle and arc path', () => {
    const html = renderToString(createElement(Spinner, null))
    expect(html).toContain('<circle')
    expect(html).toContain('<path')
  })
})
