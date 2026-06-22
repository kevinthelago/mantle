import { Notification, NotificationAction, parseActions } from './types'
import { useNotificationStore } from './store'
import styles from './NotificationPopup.module.css'

// ── single popup card ─────────────────────────────────────────────────────────

interface PopupCardProps {
  notification: Notification
}

function PopupCard({ notification: n }: PopupCardProps) {
  const { dismiss, invokeAction, pauseExpiry, resumeExpiry } = useNotificationStore()
  const actions = parseActions(n.actions)

  const handleMouseEnter = () => {
    pauseExpiry(n.id)
  }

  const handleMouseLeave = () => {
    resumeExpiry(n.id)
  }

  const handleDismiss = () => {
    dismiss(n.id)
  }

  const handleAction = (action: NotificationAction) => {
    invokeAction(n.id, action.key)
  }

  const urgencyClass =
    n.urgency === 'critical' ? styles.critical : n.urgency === 'low' ? styles.low : styles.normal

  const imageUrl = n.image
    ? `data:image/png;base64,${n.image.data_b64}`
    : n.image_path
      ? `file://${n.image_path}`
      : null

  return (
    <div
      className={`${styles.card} ${urgencyClass}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      role="alert"
      aria-live={n.urgency === 'critical' ? 'assertive' : 'polite'}
    >
      <div className={styles.header}>
        {imageUrl && (
          <img className={styles.icon} src={imageUrl} alt={n.app_name} width={32} height={32} />
        )}
        {!imageUrl && n.app_icon && (
          <img
            className={styles.icon}
            src={`/icons/${n.app_icon}.png`}
            alt={n.app_name}
            width={32}
            height={32}
            onError={(e) => {
              const img = e.currentTarget as HTMLImageElement
              img.style.display = 'none'
            }}
          />
        )}
        <span className={styles.appName}>{n.app_name}</span>
        <button
          className={styles.closeBtn}
          onClick={handleDismiss}
          aria-label="Dismiss notification"
        >
          ×
        </button>
      </div>

      <div className={styles.body}>
        <p className={styles.summary}>{n.summary}</p>
        {n.body && (
          <p
            className={styles.bodyText}
            // Body is sanitized server-side (FDO markup subset only).
            dangerouslySetInnerHTML={{ __html: n.body }}
          />
        )}
      </div>

      {actions.length > 0 && (
        <div className={styles.actions}>
          {actions.map((action) => (
            <button
              key={action.key}
              className={styles.actionBtn}
              onClick={() => handleAction(action)}
            >
              {action.label}
            </button>
          ))}
        </div>
      )}

      {n.expire_timeout_ms && (
        <div
          className={styles.progressBar}
          style={{
            animationDuration: `${n.expire_timeout_ms}ms`,
            animationPlayState: 'running',
          }}
        />
      )}
    </div>
  )
}

// ── popup layer (all active popups) ──────────────────────────────────────────

export function NotificationPopupLayer() {
  const active = useNotificationStore((s) => s.active)
  const dndEnabled = useNotificationStore((s) => s.dndEnabled)

  if (dndEnabled) return null

  return (
    <div className={styles.layer} aria-label="Notifications">
      {active.map((n) => (
        <PopupCard key={n.id} notification={n} />
      ))}
    </div>
  )
}
