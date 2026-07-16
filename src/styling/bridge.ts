import { invoke } from '@tauri-apps/api/core'

export interface LoadResult {
  /** Compiled CSS string. */
  css: string
  /**
   * True when the backend served the last-good fallback because the live
   * compile failed.  Callers can surface a warning when this is true.
   */
  from_cache: boolean
}

/**
 * Ask the Tauri backend to fetch (if not cached), compile, and return CSS.
 *
 * @param tool    npm package name — must be in the backend allowlist.
 * @param version Exact version string (e.g. `"1.69.5"`).
 * @param input   Source text (SCSS, Tailwind CSS directives, etc.).
 */
export function loadStyling(tool: string, version: string, input: string): Promise<LoadResult> {
  return invoke<LoadResult>('load_styling', { tool, version, input })
}
