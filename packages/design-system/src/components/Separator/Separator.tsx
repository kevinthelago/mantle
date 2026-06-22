import { type HTMLAttributes } from 'react'
import styles from './Separator.module.css'

export interface SeparatorProps extends HTMLAttributes<HTMLHRElement> {
  orientation?: 'horizontal' | 'vertical'
  strength?: 'subtle' | 'default' | 'strong'
}

/** Visual divider — horizontal or vertical. */
export function Separator({
  orientation = 'horizontal',
  strength = 'default',
  className,
  ...rest
}: SeparatorProps) {
  const cls = [
    styles.separator,
    styles[orientation],
    strength !== 'default' && styles[strength],
    className,
  ]
    .filter(Boolean)
    .join(' ')

  return <hr role="separator" aria-orientation={orientation} className={cls} {...rest} />
}
