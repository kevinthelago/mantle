import React from 'react'

import { useTray } from './useTray'
import { TrayItemView } from './TrayItem'
import styles from './Tray.module.css'

export function TrayModule(): React.ReactElement | null {
  const { items, activate, sendMenuEvent, refreshMenu } = useTray()

  if (items.length === 0) return null

  return (
    <div className={styles.tray} role="toolbar" aria-label="System tray">
      {items.map((item) => (
        <TrayItemView
          key={item.key}
          item={item}
          onActivate={activate}
          onMenuEvent={sendMenuEvent}
          onRefreshMenu={refreshMenu}
        />
      ))}
    </div>
  )
}

export const barModule = {
  id: 'tray',
  region: 'right' as const,
  order: 1,
  component: TrayModule,
}
