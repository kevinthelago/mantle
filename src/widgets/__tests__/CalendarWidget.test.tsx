import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { CalendarWidget } from '../calendar/CalendarWidget'

// Tuesday, 23 June 2026 at 14:30 local time
const FIXED = new Date(2026, 5, 23, 14, 30, 0)

describe('CalendarWidget', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(FIXED)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows the current month and year', () => {
    render(<CalendarWidget />)
    screen.getByText('June 2026')
  })

  it('shows the current time', () => {
    render(<CalendarWidget />)
    screen.getByText('14:30')
  })

  it('renders seven weekday headers starting on Monday', () => {
    render(<CalendarWidget />)
    for (const h of ['Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa', 'Su']) {
      screen.getByText(h)
    }
  })

  it('marks today with a distinct CSS class', () => {
    const { container } = render(<CalendarWidget />)
    const today = container.querySelector('[class*="today"]')
    expect(today).not.toBeNull()
    expect(today?.textContent).toBe('23')
  })

  it('renders days from adjacent months with an outside class', () => {
    const { container } = render(<CalendarWidget />)
    const outside = container.querySelectorAll('[class*="outside"]')
    expect(outside.length).toBeGreaterThan(0)
  })
})
