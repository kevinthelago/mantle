import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { create } from 'zustand'
import { Notification, NotificationEvent } from './types'

interface NotificationState {
  /** Currently visible popup notifications (not yet dismissed/expired). */
  active: Notification[]
  /** Full notification history (persisted). */
  history: Notification[]
  dndEnabled: boolean
  centerOpen: boolean

  // ── actions ───────────────────────────────────────────────────────────────
  init: () => () => void
  openCenter: () => void
  closeCenter: () => void
  toggleCenter: () => void
  dismiss: (id: number) => Promise<void>
  invokeAction: (id: number, actionKey: string) => Promise<void>
  setDnd: (enabled: boolean) => Promise<void>
  pauseExpiry: (id: number) => Promise<void>
  resumeExpiry: (id: number) => Promise<void>
  clearHistory: () => Promise<void>
}

export const useNotificationStore = create<NotificationState>((set, _get) => ({
  active: [],
  history: [],
  dndEnabled: false,
  centerOpen: false,

  init() {
    // Hydrate from backend.
    Promise.all([
      invoke<Notification[]>('get_active_notifications'),
      invoke<Notification[]>('get_notification_history'),
      invoke<boolean>('get_dnd_enabled'),
    ]).then(([active, history, dndEnabled]) => {
      set({ active, history, dndEnabled })
    })

    // Subscribe to live events.
    const unlisten = listen<NotificationEvent>('notification-event', ({ payload }) => {
      switch (payload.type) {
        case 'Added':
          set((s) => ({
            active: [payload.data, ...s.active],
            history: s.active.find((n) => n.id === payload.data.id)
              ? s.history
              : [payload.data, ...s.history].slice(0, 200),
          }))
          break

        case 'Replaced': {
          const { id, notification } = payload.data
          set((s) => ({
            active: s.active.map((n) => (n.id === id ? notification : n)),
            history: s.history.map((n) => (n.id === id ? notification : n)),
          }))
          break
        }

        case 'Closed':
          set((s) => ({
            active: s.active.filter((n) => n.id !== payload.data.id),
          }))
          break

        case 'DndChanged':
          set({ dndEnabled: payload.data.enabled })
          break

        case 'ActionInvoked':
          // Close the notification after action.
          set((s) => ({
            active: s.active.filter((n) => n.id !== payload.data.id),
          }))
          break
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  },

  openCenter: () => set({ centerOpen: true }),
  closeCenter: () => set({ centerOpen: false }),
  toggleCenter: () => set((s) => ({ centerOpen: !s.centerOpen })),

  async dismiss(id) {
    set((s) => ({ active: s.active.filter((n) => n.id !== id) }))
    await invoke('dismiss_notification', { id })
  },

  async invokeAction(id, actionKey) {
    set((s) => ({ active: s.active.filter((n) => n.id !== id) }))
    await invoke('invoke_action', { id, actionKey })
  },

  async setDnd(enabled) {
    set({ dndEnabled: enabled })
    await invoke('set_dnd_enabled', { enabled })
  },

  async pauseExpiry(id) {
    await invoke('pause_notification_expiry', { id })
  },

  async resumeExpiry(id) {
    await invoke('resume_notification_expiry', { id })
  },

  async clearHistory() {
    set({ history: [] })
    await invoke('clear_notification_history')
  },
}))
