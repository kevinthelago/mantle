import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { ResultsList } from '../components/ResultsList'
import type { SearchResult } from '../types'

// jsdom does not implement scrollIntoView
beforeAll(() => {
  window.HTMLElement.prototype.scrollIntoView = vi.fn()
})

function makeResult(name: string): SearchResult {
  return {
    entry: {
      id: name.toLowerCase(),
      name,
      description: `${name} description`,
      icon: undefined,
      exec: name.toLowerCase(),
      terminal: false,
      categories: [],
      keywords: [],
      desktopFile: `/usr/share/applications/${name.toLowerCase()}.desktop`,
    },
    score: 0.8,
    launchCount: 0,
  }
}

describe('ResultsList', () => {
  it('renders nothing when results are empty', () => {
    const { container } = render(<ResultsList results={[]} selectedIndex={0} onSelect={vi.fn()} />)
    expect(container.firstChild).toBeNull()
  })

  it('renders a list item for each result', () => {
    const results = [makeResult('Firefox'), makeResult('Code'), makeResult('Terminal')]
    render(<ResultsList results={results} selectedIndex={0} onSelect={vi.fn()} />)
    expect(screen.getAllByRole('option')).toHaveLength(3)
  })

  it('has listbox role for accessibility', () => {
    const results = [makeResult('Firefox')]
    render(<ResultsList results={results} selectedIndex={0} onSelect={vi.fn()} />)
    expect(screen.getByRole('listbox')).toBeTruthy()
  })

  it('marks the selected index as selected', () => {
    const results = [makeResult('Firefox'), makeResult('Code')]
    render(<ResultsList results={results} selectedIndex={1} onSelect={vi.fn()} />)
    const options = screen.getAllByRole('option')
    expect(options[0].getAttribute('aria-selected')).toBe('false')
    expect(options[1].getAttribute('aria-selected')).toBe('true')
  })

  it('calls onSelect with the correct index when an item is clicked', async () => {
    const onSelect = vi.fn()
    const results = [makeResult('Firefox'), makeResult('Code')]
    render(<ResultsList results={results} selectedIndex={0} onSelect={onSelect} />)
    await userEvent.click(screen.getAllByRole('option')[1])
    expect(onSelect).toHaveBeenCalledWith(1)
  })
})
