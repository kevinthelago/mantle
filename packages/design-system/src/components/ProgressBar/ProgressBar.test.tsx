import { describe, it, expect } from 'vitest'
import { renderToString } from 'react-dom/server'
import { createElement } from 'react'
import { ProgressBar } from './ProgressBar.js'

describe('ProgressBar', () => {
  it('renders with role=progressbar', () => {
    const html = renderToString(createElement(ProgressBar, { value: 50 }))
    expect(html).toContain('role="progressbar"')
  })

  it('sets aria-valuenow for determinate progress', () => {
    const html = renderToString(createElement(ProgressBar, { value: 75 }))
    expect(html).toContain('aria-valuenow="75"')
    expect(html).toContain('aria-valuemin="0"')
    expect(html).toContain('aria-valuemax="100"')
  })

  it('omits aria-valuenow for indeterminate state', () => {
    const html = renderToString(createElement(ProgressBar, null))
    expect(html).not.toContain('aria-valuenow')
    expect(html).toContain('aria-valuetext="loading"')
  })

  it('renders label when provided', () => {
    const html = renderToString(createElement(ProgressBar, { value: 30, label: 'Uploading' }))
    expect(html).toContain('Uploading')
    expect(html).toContain('aria-label="Uploading"')
  })

  it('renders percentage text when showValue=true', () => {
    const html = renderToString(createElement(ProgressBar, { value: 50, showValue: true }))
    expect(html).toContain('50%')
  })

  it('clamps value at 100', () => {
    const html = renderToString(createElement(ProgressBar, { value: 150, showValue: true }))
    expect(html).toContain('100%')
  })

  it('clamps value at 0', () => {
    const html = renderToString(createElement(ProgressBar, { value: -10, showValue: true }))
    expect(html).toContain('0%')
  })

  it('renders color variant classes', () => {
    for (const color of ['brand', 'success', 'error', 'warning', 'info'] as const) {
      const html = renderToString(createElement(ProgressBar, { color }))
      expect(html).toContain(color)
    }
  })
})
