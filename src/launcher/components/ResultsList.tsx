import { useEffect, useRef } from 'react'
import type { SearchResult } from '../types'
import { ResultItem } from './ResultItem'
import styles from './ResultsList.module.css'

interface ResultsListProps {
  results: SearchResult[]
  selectedIndex: number
  onSelect: (index: number) => void
}

export function ResultsList({ results, selectedIndex, onSelect }: ResultsListProps) {
  const selectedRef = useRef<HTMLLIElement>(null)

  // Keep selected item in view.
  useEffect(() => {
    selectedRef.current?.scrollIntoView({ block: 'nearest' })
  }, [selectedIndex])

  if (results.length === 0) {
    return null
  }

  return (
    <ul className={styles.list} role="listbox" aria-label="Search results">
      {results.map((result, i) => (
        <ResultItem
          key={result.entry.id}
          result={result}
          isSelected={i === selectedIndex}
          onClick={() => onSelect(i)}
          ref={i === selectedIndex ? selectedRef : undefined}
        />
      ))}
    </ul>
  )
}
