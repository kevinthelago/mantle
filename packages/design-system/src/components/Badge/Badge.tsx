import { type HTMLAttributes } from 'react'
import styles from './Badge.module.css'

export type BadgeVariant = 'default' | 'brand' | 'success' | 'error' | 'warning' | 'info'

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant
  /** Renders a status dot before the label. */
  dot?: boolean
}

/** Status badge / label chip. */
export function Badge({
  variant = 'default',
  dot = false,
  className,
  children,
  ...rest
}: BadgeProps) {
  const cls = [styles.badge, styles[variant], className].filter(Boolean).join(' ')
  return (
    <span className={cls} {...rest}>
      {dot && <span className={styles.dot} aria-hidden="true" />}
      {children}
    </span>
  )
}
