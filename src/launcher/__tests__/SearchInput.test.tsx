import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { SearchInput } from '../components/SearchInput'

describe('SearchInput', () => {
  it('renders with default search placeholder in apps mode', () => {
    render(<SearchInput value="" mode="apps" onChange={vi.fn()} />)
    expect(screen.getByPlaceholderText('Search apps…')).toBeTruthy()
  })

  it('renders with run-mode placeholder when mode is run', () => {
    render(<SearchInput value="" mode="run" onChange={vi.fn()} />)
    expect(screen.getByPlaceholderText('Run command…')).toBeTruthy()
  })

  it('uses custom placeholder when provided', () => {
    render(<SearchInput value="" mode="apps" onChange={vi.fn()} placeholder="Custom…" />)
    expect(screen.getByPlaceholderText('Custom…')).toBeTruthy()
  })

  it('shows run icon in run mode', () => {
    const { container } = render(<SearchInput value="" mode="run" onChange={vi.fn()} />)
    const icon = container.querySelector('[aria-hidden="true"]')
    expect(icon?.textContent).toBe('>')
  })

  it('calls onChange when user types', async () => {
    const onChange = vi.fn()
    render(<SearchInput value="" mode="apps" onChange={onChange} />)
    await userEvent.type(screen.getByRole('textbox'), 'fire')
    expect(onChange).toHaveBeenCalledTimes(4)
  })

  it('reflects controlled value', () => {
    render(<SearchInput value="firefox" mode="apps" onChange={vi.fn()} />)
    expect((screen.getByRole('textbox') as HTMLInputElement).value).toBe('firefox')
  })

  it('has aria-label for accessibility', () => {
    render(<SearchInput value="" mode="apps" onChange={vi.fn()} />)
    expect(screen.getByLabelText('Launcher search')).toBeTruthy()
  })
})
