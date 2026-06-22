import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';
import styles from './Button.module.css';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'destructive';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  /** Prepend icon */
  startIcon?: ReactNode;
  /** Append icon */
  endIcon?: ReactNode;
}

/** Interactive button with primary/secondary/ghost/destructive variants. */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    variant = 'secondary',
    size = 'md',
    loading = false,
    startIcon,
    endIcon,
    disabled,
    className,
    children,
    ...rest
  },
  ref,
) {
  const cls = [
    styles.button,
    styles[variant],
    styles[size],
    loading && styles.loading,
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <button
      ref={ref}
      className={cls}
      disabled={disabled ?? loading}
      aria-disabled={disabled ?? loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading && (
        <svg
          className={styles.spinner}
          viewBox="0 0 24 24"
          fill="none"
          aria-hidden="true"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M21 12a9 9 0 1 1-6.219-8.56"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </svg>
      )}
      {!loading && startIcon}
      {children}
      {!loading && endIcon}
    </button>
  );
});
