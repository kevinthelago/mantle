import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react'
import styles from './IconButton.module.css'

export type IconButtonVariant = 'primary' | 'secondary' | 'ghost' | 'destructive'
export type IconButtonSize = 'sm' | 'md' | 'lg'

export interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Accessible label — required since there is no visible text. */
  'aria-label': string
  variant?: IconButtonVariant
  size?: IconButtonSize
  children: ReactNode
}

/** Icon-only button. `aria-label` is required. */
export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(function IconButton(
  { variant = 'ghost', size = 'md', className, children, ...rest },
  ref,
) {
  const cls = [styles.iconButton, styles[variant], styles[size], className]
    .filter(Boolean)
    .join(' ')

  return (
    <button ref={ref} className={cls} type="button" {...rest}>
      {children}
    </button>
  )
})
