import type { ComponentType } from 'react'
import type { WidgetId } from './types'
import { CalendarWidget } from './calendar'
import { MediaControlsWidget } from './media-controls'
import { SystemMonitorWidget } from './system-monitor'

export const WIDGET_REGISTRY: Partial<Record<WidgetId, ComponentType>> = {
  calendar: CalendarWidget,
  'system-monitor': SystemMonitorWidget,
  'media-controls': MediaControlsWidget,
}
