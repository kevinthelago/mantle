import { useEffect, useRef } from 'react'
import type { Mode } from '../types'
import styles from './SearchInput.module.css'

interface SearchInputProps {
  value: string
  mode: Mode
  onChange: (value: string) => void
  placeholder?: string
}

/**
 * Auto-focused search/command input.
 * Typing ">" switches to run mode (arbitrary shell command).
 */
export function SearchInput({ value, mode, onChange, placeholder }: SearchInputProps) {
  const ref = useRef<HTMLInputElement>(null)

  useEffect(() => {
    ref.current?.focus()
  }, [])

  return (
    <div className={styles.wrapper} data-mode={mode}>
      <span className={styles.icon} aria-hidden="true">
        {mode === 'run' ? '>' : ''}
      </span>
      <input
        ref={ref}
        className={styles.input}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder ?? (mode === 'run' ? 'Run command…' : 'Search apps…')}
        autoComplete="off"
        autoCorrect="off"
        autoCapitalize="off"
        spellCheck={false}
        aria-label="Launcher search"
        aria-autocomplete="list"
      />
    </div>
  )
}
