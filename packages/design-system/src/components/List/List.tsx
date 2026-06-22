import { forwardRef, type HTMLAttributes, type LiHTMLAttributes, type ReactNode } from 'react'
import styles from './List.module.css'

export interface ListProps extends HTMLAttributes<HTMLUListElement> {
  /** Set to true when list items are interactive (enables role="listbox" semantics). */
  interactive?: boolean
}

/** Styled list container. */
export const List = forwardRef<HTMLUListElement, ListProps>(function List(
  { interactive = false, className, ...rest },
  ref,
) {
  return (
    <ul
      ref={ref}
      role={interactive ? 'listbox' : 'list'}
      className={[styles.list, className].filter(Boolean).join(' ')}
      {...rest}
    />
  )
})

export interface ListItemProps extends LiHTMLAttributes<HTMLLIElement> {
  /** Leading element (icon, avatar, etc.) */
  startSlot?: ReactNode
  /** Trailing element (badge, action, etc.) */
  endSlot?: ReactNode
  /** Secondary descriptive text below the title. */
  description?: ReactNode
  interactive?: boolean
  selected?: boolean
  disabled?: boolean
}

/** Row inside a List. */
export const ListItem = forwardRef<HTMLLIElement, ListItemProps>(function ListItem(
  {
    startSlot,
    endSlot,
    description,
    interactive = false,
    selected = false,
    disabled = false,
    className,
    children,
    ...rest
  },
  ref,
) {
  const cls = [
    styles.listItem,
    interactive && styles.interactive,
    selected && styles.selected,
    disabled && styles.disabled,
    className,
  ]
    .filter(Boolean)
    .join(' ')

  return (
    <li
      ref={ref}
      className={cls}
      role={interactive ? 'option' : undefined}
      aria-selected={interactive ? selected : undefined}
      aria-disabled={disabled || undefined}
      tabIndex={interactive && !disabled ? 0 : undefined}
      {...rest}
    >
      {startSlot && (
        <span className={styles.startSlot} aria-hidden="true">
          {startSlot}
        </span>
      )}
      <span className={styles.content}>
        {description ? (
          <>
            <span className={styles.title}>{children}</span>
            <span className={styles.description}>{description}</span>
          </>
        ) : (
          children
        )}
      </span>
      {endSlot && <span className={styles.endSlot}>{endSlot}</span>}
    </li>
  )
})
