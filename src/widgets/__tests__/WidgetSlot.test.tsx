import { render, screen } from '@testing-library/react'
import { describe, it, vi } from 'vitest'
import { WidgetSlot } from '../WidgetSlot'

function Bomb(): never {
  throw new Error('widget exploded')
}

describe('WidgetSlot', () => {
  it('passes through children when no error is thrown', () => {
    render(
      <WidgetSlot widgetId="test">
        <span>hello</span>
      </WidgetSlot>,
    )
    screen.getByText('hello')
  })

  it('catches a child error and shows the widget id and message', () => {
    // Suppress React's error boundary output in the test console
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    render(
      <WidgetSlot widgetId="test-widget">
        <Bomb />
      </WidgetSlot>,
    )
    screen.getByText(/test-widget/)
    screen.getByText(/widget exploded/)
    spy.mockRestore()
  })
})
