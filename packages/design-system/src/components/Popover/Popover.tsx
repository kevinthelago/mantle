import {
  type ButtonHTMLAttributes,
  type HTMLAttributes,
  type ReactNode,
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
} from 'react'
import styles from './Popover.module.css'

export interface PopoverProps {
  /** The element that opens the popover. */
  trigger: ReactNode
  children: ReactNode
  className?: string
}

/** Accessible popover / dropdown container. Closes on outside click and Escape. */
export function Popover({ trigger, children, className }: PopoverProps) {
  const [open, setOpen] = useState(false)
  const wrapperRef = useRef<HTMLDivElement>(null)
  const contentRef = useRef<HTMLDivElement>(null)
  const panelId = useId()

  const close = useCallback(() => setOpen(false), [])

  useEffect(() => {
    if (!open) return
    const handler = (e: MouseEvent) => {
      if (!wrapperRef.current?.contains(e.target as Node)) close()
    }
    const keyHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close()
    }
    document.addEventListener('mousedown', handler)
    document.addEventListener('keydown', keyHandler)
    return () => {
      document.removeEventListener('mousedown', handler)
      document.removeEventListener('keydown', keyHandler)
    }
  }, [open, close])

  useEffect(() => {
    if (open) contentRef.current?.focus()
  }, [open])

  const contentCls = [styles.content, open && styles.open, className].filter(Boolean).join(' ')

  return (
    <div ref={wrapperRef} className={styles.wrapper}>
      <span
        onClick={() => setOpen((v) => !v)}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={panelId}
      >
        {trigger}
      </span>
      <div
        id={panelId}
        ref={contentRef}
        className={contentCls}
        role="menu"
        aria-hidden={!open}
        tabIndex={-1}
      >
        {children}
      </div>
    </div>
  )
}

export interface MenuItemProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  danger?: boolean
  icon?: ReactNode
}

/** Item inside a Popover menu. */
export function MenuItem({ danger = false, icon, children, className, ...rest }: MenuItemProps) {
  const cls = [styles.menuItem, danger && styles.danger, className].filter(Boolean).join(' ')
  return (
    <button role="menuitem" className={cls} type="button" {...rest}>
      {icon && <span aria-hidden="true">{icon}</span>}
      {children}
    </button>
  )
}

/** Divider between menu sections. */
export function MenuSeparator(props: HTMLAttributes<HTMLDivElement>) {
  return <div role="separator" className={styles.separator} {...props} />
}
