import { useEffect } from 'react'

interface KeyboardNavOptions {
  onUp: () => void
  onDown: () => void
  onConfirm: () => void
  onDismiss: () => void
  onType?: (key: string) => void
}

/** Attach keyboard navigation to the launcher window. */
export function useKeyboardNav({ onUp, onDown, onConfirm, onDismiss }: KeyboardNavOptions): void {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault()
          onUp()
          break
        case 'ArrowDown':
          e.preventDefault()
          onDown()
          break
        case 'Enter':
          e.preventDefault()
          onConfirm()
          break
        case 'Escape':
          e.preventDefault()
          onDismiss()
          break
        default:
          break
      }
    }

    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [onUp, onDown, onConfirm, onDismiss])
}
