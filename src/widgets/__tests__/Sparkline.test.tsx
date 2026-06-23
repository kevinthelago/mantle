import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Sparkline } from '../system-monitor/Sparkline'

describe('Sparkline', () => {
  it('renders an empty SVG when given no values', () => {
    const { container } = render(<Sparkline values={[]} max={100} />)
    expect(container.querySelector('polyline')).toBeNull()
    expect(container.querySelector('path')).toBeNull()
  })

  it('renders an empty SVG for a single value', () => {
    const { container } = render(<Sparkline values={[50]} max={100} />)
    expect(container.querySelector('polyline')).toBeNull()
  })

  it('renders a polyline and fill path for two or more values', () => {
    const { container } = render(<Sparkline values={[0, 50, 100]} max={100} />)
    expect(container.querySelector('polyline')).not.toBeNull()
    expect(container.querySelector('path')).not.toBeNull()
  })

  it('uses the given width and height in the SVG viewBox', () => {
    const { container } = render(<Sparkline values={[10, 20]} max={100} width={120} height={40} />)
    expect(container.querySelector('svg')?.getAttribute('viewBox')).toBe('0 0 120 40')
  })

  it('defaults to 80×24 viewBox', () => {
    const { container } = render(<Sparkline values={[10, 20]} max={100} />)
    expect(container.querySelector('svg')?.getAttribute('viewBox')).toBe('0 0 80 24')
  })

  it('marks the data-bearing SVG as aria-hidden', () => {
    const { container } = render(<Sparkline values={[10, 20]} max={100} />)
    expect(container.querySelector('svg')?.getAttribute('aria-hidden')).toBe('true')
  })
})
