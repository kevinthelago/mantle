import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { MediaControlsWidget } from '../media-controls/MediaControlsWidget'
import type { PlayerState } from '../types'

const PLAYING_STATE: PlayerState = {
  player_name: 'spotify',
  playback_status: 'Playing',
  position_us: 30_000_000,
  volume: 0.8,
  metadata: {
    title: 'Test Song',
    artist: 'Test Artist',
    album: 'Test Album',
    art_url: undefined,
    length_us: 240_000_000,
  },
  can_next: true,
  can_prev: false,
  can_pause: true,
}

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }))

describe('MediaControlsWidget', () => {
  let invokeMock: ReturnType<typeof vi.fn>

  beforeEach(async () => {
    const mod = await import('@tauri-apps/api/core')
    invokeMock = mod.invoke as ReturnType<typeof vi.fn>
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders nothing while the initial state is loading', () => {
    invokeMock.mockReturnValue(new Promise<PlayerState | null>(() => {}))
    const { container } = render(<MediaControlsWidget />)
    expect(container.firstChild).toBeNull()
  })

  it('shows a no-player message when there is no active player', async () => {
    invokeMock.mockResolvedValue(null)
    render(<MediaControlsWidget />)
    await waitFor(() => screen.getByText('No media playing'))
  })

  it('shows track title, artist, and player name when playing', async () => {
    invokeMock.mockResolvedValue(PLAYING_STATE)
    render(<MediaControlsWidget />)
    await waitFor(() => screen.getByText('Test Song'))
    screen.getByText('Test Artist')
    screen.getByText('spotify')
  })

  it('disables the previous button when can_prev is false', async () => {
    invokeMock.mockResolvedValue(PLAYING_STATE)
    render(<MediaControlsWidget />)
    await waitFor(() => screen.getByLabelText('Previous track'))
    expect((screen.getByLabelText('Previous track') as HTMLButtonElement).disabled).toBe(true)
  })

  it('enables the next button when can_next is true', async () => {
    invokeMock.mockResolvedValue(PLAYING_STATE)
    render(<MediaControlsWidget />)
    await waitFor(() => screen.getByLabelText('Next track'))
    expect((screen.getByLabelText('Next track') as HTMLButtonElement).disabled).toBe(false)
  })

  it('shows the pause icon when playing', async () => {
    invokeMock.mockResolvedValue(PLAYING_STATE)
    render(<MediaControlsWidget />)
    await waitFor(() => screen.getByLabelText('Pause'))
  })
})
