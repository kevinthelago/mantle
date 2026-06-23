import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { LauncherApp } from '../LauncherApp'

vi.mock('../api', () => ({
  api: {
    search: vi.fn().mockResolvedValue([
      {
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
        launchCount: 3,
      },
    ]),
    launch: vi.fn().mockResolvedValue(undefined),
    runCommand: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  },
}))

describe('LauncherApp', () => {
  it('renders the search input', () => {
    render(<LauncherApp />)
    expect(screen.getByRole('textbox')).toBeTruthy()
  })

  it('has dialog role with accessible label', () => {
    render(<LauncherApp />)
    expect(screen.getByRole('dialog', { name: 'App launcher' })).toBeTruthy()
  })

  it('shows seeded results on mount', async () => {
    render(<LauncherApp />)
    await waitFor(() => expect(screen.getByText('Firefox')).toBeTruthy())
  })

  it('switches search placeholder to run mode on > input', async () => {
    render(<LauncherApp />)
    await userEvent.type(screen.getByRole('textbox'), '>')
    expect(screen.getByPlaceholderText('Run command…')).toBeTruthy()
  })

  it('launches app on Enter', async () => {
    const { api } = await import('../api')
    render(<LauncherApp />)
    await waitFor(() => expect(screen.getByText('Firefox')).toBeTruthy())
    await userEvent.keyboard('{Enter}')
    expect(api.launch).toHaveBeenCalledWith('firefox')
    expect(api.close).toHaveBeenCalled()
  })

  it('closes on Escape', async () => {
    const { api } = await import('../api')
    render(<LauncherApp />)
    await userEvent.keyboard('{Escape}')
    expect(api.close).toHaveBeenCalled()
  })
})
