import { type HTMLAttributes, type ReactNode, useId } from 'react';
import styles from './Tooltip.module.css';

export type TooltipPlacement = 'top' | 'bottom' | 'left' | 'right';

export interface TooltipProps {
  content: ReactNode;
  placement?: TooltipPlacement;
  children: ReactNode;
  className?: string;
}

/**
 * CSS-only tooltip (no JS positioning). Wraps trigger in a relative container
 * and shows content on hover/focus-within.
 */
export function Tooltip({ content, placement = 'top', children, className }: TooltipProps) {
  const tipId = useId();
  const cls = [styles.wrapper, className].filter(Boolean).join(' ');
  const contentCls = [styles.content, styles[placement]].join(' ');

  return (
    <span className={cls}>
      {/* aria-describedby links trigger to tooltip text */}
      <span aria-describedby={tipId}>{children}</span>
      <span
        id={tipId}
        role="tooltip"
        className={contentCls}
      >
        {content}
      </span>
    </span>
  );
}

export interface TooltipWrapperProps extends HTMLAttributes<HTMLDivElement> {
  /** Tooltip text shown on hover/focus */
  label: string;
  placement?: TooltipPlacement;
  children: ReactNode;
}
