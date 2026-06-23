import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ResultItem } from '../components/ResultItem'
import type { SearchResult } from '../types'

const baseResult: SearchResult = {
  entry: {
    id: 'firefox',
    name: 'Firefox',
    description: 'Web browser',
    icon: undefined,
    exec: 'firefox %u',
    terminal: false,
    categories: ['Network'],
    keywords: ['browser'],
    desktopFile: '/usr/share/applications/firefox.desktop',
  },
  score: 0.9,
  launchCount: 5,
}

describe('ResultItem', () => {
  it('renders app name', () => {
    render(<ResultItem result={baseResult} isSelected={false} onClick={vi.fn()} />)
    expect(screen.getByText('Firefox')).toBeTruthy()
  })

  it('renders description when present', () => {
    render(<ResultItem result={baseResult} isSelected={false} onClick={vi.fn()} />)
    expect(screen.getByText('Web browser')).toBeTruthy()
  })

  it('omits description when absent', () => {
    const r: SearchResult = {
      ...baseResult,
      entry: { ...baseResult.entry, description: undefined },
    }
    const { container } = render(<ResultItem result={r} isSelected={false} onClick={vi.fn()} />)
    expect(container.querySelectorAll('span').length).toBeGreaterThan(0)
    expect(screen.queryByText('Web browser')).toBeNull()
  })

  it('shows letter fallback icon when no icon path', () => {
    render(<ResultItem result={baseResult} isSelected={false} onClick={vi.fn()} />)
    const fallback = screen.getByText('F')
    expect(fallback).toBeTruthy()
  })

  it('shows <img> when icon is an absolute path', () => {
    const r: SearchResult = {
      ...baseResult,
      entry: { ...baseResult.entry, icon: '/usr/share/icons/firefox.png' },
    }
    const { container } = render(<ResultItem result={r} isSelected={false} onClick={vi.fn()} />)
    expect(container.querySelector('img')).toBeTruthy()
  })

  it('uses letter fallback for named (non-path) icons', () => {
    const r: SearchResult = {
      ...baseResult,
      entry: { ...baseResult.entry, icon: 'firefox' },
    }
    render(<ResultItem result={r} isSelected={false} onClick={vi.fn()} />)
    expect(screen.getByText('F')).toBeTruthy()
  })

  it('marks item as selected via aria-selected', () => {
    const { container } = render(
      <ResultItem result={baseResult} isSelected={true} onClick={vi.fn()} />,
    )
    const li = container.querySelector('li')
    expect(li?.getAttribute('aria-selected')).toBe('true')
    expect(li?.getAttribute('data-selected')).toBe('true')
  })

  it('calls onClick when clicked', async () => {
    const onClick = vi.fn()
    render(<ResultItem result={baseResult} isSelected={false} onClick={onClick} />)
    await userEvent.click(screen.getByRole('option'))
    expect(onClick).toHaveBeenCalledOnce()
  })
})
