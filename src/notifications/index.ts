import { registerSurface } from '../lib/surfaces'
import { NotificationSurface } from './NotificationSurface'

// Register the notifications overlay surface so App.tsx routes here when
// the "notifications" Tauri window is active.
registerSurface({ id: 'notifications', component: NotificationSurface })

export { NotificationSurface } from './NotificationSurface'
export { NotificationPopupLayer } from './NotificationPopup'
export { NotificationCenter } from './NotificationCenter'
export {
  NotificationIndicator,
  registerNotificationIndicator,
  useNotificationIndicatorRegistration,
} from './NotificationIndicator'
export { useNotificationStore } from './store'
export type {
  Notification,
  NotificationAction,
  NotificationEvent,
  NotificationImage,
  Urgency,
  ClosedReason,
} from './types'
export { parseActions } from './types'
