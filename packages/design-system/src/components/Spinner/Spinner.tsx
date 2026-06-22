import { type SVGAttributes } from 'react'
import styles from './Spinner.module.css'

export type SpinnerSize = 'sm' | 'md' | 'lg' | 'xl'

export interface SpinnerProps extends SVGAttributes<SVGSVGElement> {
  size?: SpinnerSize
  /** Use current CSS color instead of the brand token. */
  inherit?: boolean
  label?: string
}

/** Animated loading spinner. */
export function Spinner({
  size = 'md',
  inherit = false,
  label = 'Loading…',
  className,
  ...rest
}: SpinnerProps) {
  const cls = [styles.spinner, styles[size], inherit && styles.inherit, className]
    .filter(Boolean)
    .join(' ')

  return (
    <svg
      className={cls}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      role="status"
      aria-label={label}
      {...rest}
    >
      <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="2" strokeOpacity="0.25" />
      <path
        d="M22 12a10 10 0 0 0-10-10"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
      />
    </svg>
  )
}
