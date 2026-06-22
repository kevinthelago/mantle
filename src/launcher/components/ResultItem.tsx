import { forwardRef } from 'react'
import type { SearchResult } from '../types'
import styles from './ResultItem.module.css'

interface ResultItemProps {
  result: SearchResult
  isSelected: boolean
  onClick: () => void
}

export const ResultItem = forwardRef<HTMLLIElement, ResultItemProps>(function ResultItem(
  { result, isSelected, onClick },
  ref,
) {
  const { entry } = result

  return (
    <li
      ref={ref}
      className={styles.item}
      aria-selected={isSelected}
      data-selected={isSelected}
      onClick={onClick}
      role="option"
    >
      <AppIcon name={entry.name} iconName={entry.icon} />
      <span className={styles.text}>
        <span className={styles.name}>{entry.name}</span>
        {entry.description && <span className={styles.desc}>{entry.description}</span>}
      </span>
    </li>
  )
})

function AppIcon({ name, iconName }: { name: string; iconName?: string }) {
  if (iconName) {
    // If the icon is an absolute path, use it directly; otherwise treat as
    // a named XDG icon (design-system icon resolver handles the lookup).
    const src = iconName.startsWith('/') ? iconName : undefined
    if (src) {
      return <img className={styles.icon} src={src} alt="" aria-hidden />
    }
  }

  // Fallback: first letter of the app name.
  return (
    <span className={styles.iconFallback} aria-hidden>
      {name.charAt(0).toUpperCase()}
    </span>
  )
}
