export interface AppEntry {
  id: string
  name: string
  description?: string
  icon?: string
  exec: string
  terminal: boolean
  categories: string[]
  keywords: string[]
  desktopFile: string
}

export interface SearchResult {
  entry: AppEntry
  /** Normalised [0, 1] relevance score. */
  score: number
  launchCount: number
}

export type Mode = 'apps' | 'run'
