import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import BarRegion from '../BarRegion'

// Mock the registry so we don't need to lazy-load real modules.
vi.mock('../../registry', () => ({
  resolveModule: vi.fn((name: string) => {
    if (name === 'clock') return () => <div>ClockWidget</div>
    if (name === 'workspaces') return () => <div>WorkspacesWidget</div>
    return null
  }),
}))

describe('BarRegion', () => {
  it('renders all resolved modules', () => {
    render(<BarRegion modules={['clock', 'workspaces']} position="left" />)
    expect(screen.getByText('ClockWidget')).toBeTruthy()
    expect(screen.getByText('WorkspacesWidget')).toBeTruthy()
  })

  it('skips unknown module names', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    render(<BarRegion modules={['unknown-module']} position="right" />)
    // Nothing should be rendered (region is empty apart from the wrapper).
    expect(screen.queryByRole('tab')).toBeNull()
    warn.mockRestore()
  })

  it('warns when a module name is not in the registry', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    render(<BarRegion modules={['no-such-module']} position="center" />)
    expect(warn).toHaveBeenCalledWith(expect.stringContaining('unknown module'))
    warn.mockRestore()
  })

  it('has the correct ARIA region label', () => {
    render(<BarRegion modules={[]} position="left" />)
    expect(screen.getByRole('region', { name: 'left' })).toBeTruthy()
  })

  it('applies the position class to the wrapper', () => {
    const { container } = render(<BarRegion modules={[]} position="center" />)
    const region = container.firstElementChild!
    expect(region.classList.contains('bar-region--center')).toBe(true)
  })
})
