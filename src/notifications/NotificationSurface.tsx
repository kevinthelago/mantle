import { useEffect } from 'react'
import { useNotificationStore } from './store'
import { NotificationPopupLayer } from './NotificationPopup'
import { NotificationCenter } from './NotificationCenter'
import styles from './NotificationSurface.module.css'

/**
 * Root component for the `notifications` Tauri window.
 * Renders the overlay popup layer and the slide-in notification center.
 * Registered with the surface registry so App.tsx routes here when the
 * window label is "notifications".
 */
export function NotificationSurface() {
  const init = useNotificationStore((s) => s.init)

  useEffect(() => {
    return init()
  }, [init])

  return (
    <div className={styles.root}>
      <NotificationPopupLayer />
      <NotificationCenter />
    </div>
  )
}
