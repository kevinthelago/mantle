import { type HTMLAttributes } from 'react';
import styles from './ProgressBar.module.css';

export type ProgressBarColor = 'brand' | 'success' | 'error' | 'warning' | 'info';
export type ProgressBarSize = 'sm' | 'md' | 'lg';

export interface ProgressBarProps extends HTMLAttributes<HTMLDivElement> {
  /** Value 0–100. Omit or pass undefined for indeterminate. */
  value?: number;
  max?: number;
  color?: ProgressBarColor;
  size?: ProgressBarSize;
  label?: string;
  showValue?: boolean;
}

/** Determinate or indeterminate progress indicator. */
export function ProgressBar({
  value,
  max = 100,
  color = 'brand',
  size = 'md',
  label,
  showValue = false,
  className,
  ...rest
}: ProgressBarProps) {
  const indeterminate = value === undefined;
  const pct = indeterminate ? 0 : Math.min(100, Math.max(0, (value / max) * 100));

  const wrapperCls = [styles.wrapper, styles[color], styles[size], className]
    .filter(Boolean)
    .join(' ');
  const trackCls = [styles.track, indeterminate && styles.indeterminate]
    .filter(Boolean)
    .join(' ');

  return (
    <div className={wrapperCls} {...rest}>
      {(label || showValue) && (
        <div className={styles.header}>
          {label && <span className={styles.label}>{label}</span>}
          {showValue && !indeterminate && (
            <span className={styles.valueLabel}>{Math.round(pct)}%</span>
          )}
        </div>
      )}
      <div
        className={trackCls}
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={max}
        aria-valuenow={indeterminate ? undefined : value}
        aria-label={label}
        aria-valuetext={indeterminate ? 'loading' : `${Math.round(pct)}%`}
      >
        <div
          className={styles.fill}
          style={indeterminate ? undefined : { width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
