import { describe, it, expect } from 'vitest'
import { renderToString } from 'react-dom/server'
import { createElement } from 'react'
import { Separator } from './Separator.js'

describe('Separator', () => {
  it('renders with role=separator', () => {
    const html = renderToString(createElement(Separator, null))
    expect(html).toContain('role="separator"')
  })

  it('defaults to horizontal orientation', () => {
    const html = renderToString(createElement(Separator, null))
    expect(html).toContain('aria-orientation="horizontal"')
  })

  it('supports vertical orientation', () => {
    const html = renderToString(createElement(Separator, { orientation: 'vertical' }))
    expect(html).toContain('aria-orientation="vertical"')
  })

  it('applies strength class', () => {
    const html = renderToString(createElement(Separator, { strength: 'strong' }))
    expect(html).toContain('strong')
  })
})
