import { describe, it, expect } from 'vitest'
import { renderToString } from 'react-dom/server'
import { createElement } from 'react'
import { Text } from './Text.js'

describe('Text', () => {
  it('renders as span by default', () => {
    const html = renderToString(createElement(Text, null, 'Hello'))
    expect(html).toContain('<span')
    expect(html).toContain('Hello')
  })

  it('renders as specified element via as prop', () => {
    const html = renderToString(createElement(Text, { as: 'p' }, 'Paragraph'))
    expect(html).toContain('<p')
    expect(html).toContain('Paragraph')
  })

  it('renders as h1 heading', () => {
    const html = renderToString(createElement(Text, { as: 'h1' }, 'Title'))
    expect(html).toContain('<h1')
  })

  it('applies size class', () => {
    const html = renderToString(createElement(Text, { size: 'xl' }, 'Large text'))
    expect(html).toContain('xl')
  })

  it('applies color class', () => {
    const html = renderToString(createElement(Text, { color: 'brand' }, 'Brand text'))
    expect(html).toContain('brand')
  })

  it('applies mono font class', () => {
    const html = renderToString(createElement(Text, { font: 'mono' }, 'code'))
    expect(html).toContain('mono')
  })
})
