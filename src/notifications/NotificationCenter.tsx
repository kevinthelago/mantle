import { Notification, parseActions } from './types'
import { useNotificationStore } from './store'
import styles from './NotificationCenter.module.css'

// ── history item ──────────────────────────────────────────────────────────────

interface HistoryItemProps {
  notification: Notification
}

function HistoryItem({ notification: n }: HistoryItemProps) {
  const { dismiss, invokeAction } = useNotificationStore()
  const actions = parseActions(n.actions)

  const imageUrl = n.image
    ? `data:image/png;base64,${n.image.data_b64}`
    : n.image_path
      ? `file://${n.image_path}`
      : null

  const timestamp = new Date(n.created_at * 1000).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  })

  return (
    <div className={`${styles.item} ${n.urgency === 'critical' ? styles.critical : ''}`}>
      <div className={styles.itemHeader}>
        {imageUrl && <img className={styles.icon} src={imageUrl} alt={n.app_name} />}
        <span className={styles.appName}>{n.app_name}</span>
        <span className={styles.time}>{timestamp}</span>
        <button
          className={styles.closeBtn}
          onClick={() => dismiss(n.id)}
          aria-label="Remove notification"
        >
          ×
        </button>
      </div>
      <p className={styles.summary}>{n.summary}</p>
      {n.body && <p className={styles.body} dangerouslySetInnerHTML={{ __html: n.body }} />}
      {actions.length > 0 && (
        <div className={styles.actions}>
          {actions.map((a) => (
            <button
              key={a.key}
              className={styles.actionBtn}
              onClick={() => invokeAction(n.id, a.key)}
            >
              {a.label}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

// ── DND toggle ────────────────────────────────────────────────────────────────

function DndToggle() {
  const dndEnabled = useNotificationStore((s) => s.dndEnabled)
  const setDnd = useNotificationStore((s) => s.setDnd)

  return (
    <button
      className={`${styles.dndToggle} ${dndEnabled ? styles.dndActive : ''}`}
      onClick={() => setDnd(!dndEnabled)}
      aria-pressed={dndEnabled}
      title={dndEnabled ? 'Do Not Disturb: ON' : 'Do Not Disturb: OFF'}
    >
      <span className={styles.dndIcon}>{dndEnabled ? '🔕' : '🔔'}</span>
      <span>{dndEnabled ? 'DND on' : 'DND off'}</span>
    </button>
  )
}

// ── notification center panel ─────────────────────────────────────────────────

export function NotificationCenter() {
  const centerOpen = useNotificationStore((s) => s.centerOpen)
  const closeCenter = useNotificationStore((s) => s.closeCenter)
  const history = useNotificationStore((s) => s.history)
  const clearHistory = useNotificationStore((s) => s.clearHistory)

  if (!centerOpen) return null

  return (
    <>
      {/* Click-away backdrop */}
      <div className={styles.backdrop} onClick={closeCenter} aria-hidden="true" />

      <aside className={styles.panel} aria-label="Notification Center">
        <div className={styles.toolbar}>
          <h2 className={styles.title}>Notifications</h2>
          <DndToggle />
          {history.length > 0 && (
            <button className={styles.clearBtn} onClick={clearHistory}>
              Clear all
            </button>
          )}
          <button
            className={styles.panelCloseBtn}
            onClick={closeCenter}
            aria-label="Close notification center"
          >
            ×
          </button>
        </div>

        <div className={styles.list}>
          {history.length === 0 ? (
            <p className={styles.empty}>No notifications</p>
          ) : (
            history.map((n) => <HistoryItem key={n.id} notification={n} />)
          )}
        </div>
      </aside>
    </>
  )
}
