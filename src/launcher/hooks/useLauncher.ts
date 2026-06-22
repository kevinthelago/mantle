import { useCallback, useEffect, useRef, useState } from 'react'
import { api } from '../api'
import type { Mode, SearchResult } from '../types'

const RUN_PREFIX = '>'

interface LauncherState {
  query: string
  mode: Mode
  results: SearchResult[]
  selectedIndex: number
  isLoading: boolean
  error: string | null
}

interface LauncherActions {
  setQuery: (q: string) => void
  selectNext: () => void
  selectPrev: () => void
  confirm: () => Promise<void>
  dismiss: () => Promise<void>
  reset: () => void
}

export function useLauncher(): LauncherState & LauncherActions {
  const [query, setQueryRaw] = useState('')
  const [mode, setMode] = useState<Mode>('apps')
  const [results, setResults] = useState<SearchResult[]>([])
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const searchRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const setQuery = useCallback((raw: string) => {
    setQueryRaw(raw)

    const isRun = raw.startsWith(RUN_PREFIX)
    setMode(isRun ? 'run' : 'apps')

    if (searchRef.current) clearTimeout(searchRef.current)

    const effectiveQuery = isRun ? raw.slice(1).trimStart() : raw

    if (isRun) {
      // In run mode no search results — just show the raw command.
      setResults([])
      setSelectedIndex(0)
      return
    }

    setIsLoading(true)
    searchRef.current = setTimeout(async () => {
      try {
        const res = await api.search(effectiveQuery)
        setResults(res)
        setSelectedIndex(0)
        setError(null)
      } catch (e) {
        setError(String(e))
      } finally {
        setIsLoading(false)
      }
    }, 50)
  }, [])

  // Seed results on mount (empty query → frequency-ranked apps).
  useEffect(() => {
    api
      .search('')
      .then(setResults)
      .catch(() => {})
  }, [])

  const selectNext = useCallback(() => {
    setSelectedIndex((i) => Math.min(i + 1, results.length - 1))
  }, [results.length])

  const selectPrev = useCallback(() => {
    setSelectedIndex((i) => Math.max(i - 1, 0))
  }, [])

  const confirm = useCallback(async () => {
    if (mode === 'run') {
      const cmd = query.slice(1).trimStart()
      if (!cmd) return
      try {
        await api.runCommand(cmd)
        await api.close()
      } catch (e) {
        setError(String(e))
      }
      return
    }

    const selected = results[selectedIndex]
    if (!selected) return
    try {
      await api.launch(selected.entry.id)
      await api.close()
    } catch (e) {
      setError(String(e))
    }
  }, [mode, query, results, selectedIndex])

  const dismiss = useCallback(async () => {
    await api.close()
  }, [])

  const reset = useCallback(() => {
    setQueryRaw('')
    setMode('apps')
    setResults([])
    setSelectedIndex(0)
    setError(null)
  }, [])

  return {
    query,
    mode,
    results,
    selectedIndex,
    isLoading,
    error,
    setQuery,
    selectNext,
    selectPrev,
    confirm,
    dismiss,
    reset,
  }
}
