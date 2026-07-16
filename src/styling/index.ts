export { loadStyling, type LoadResult } from './bridge'
export { injectCss, removeManagedStyle } from './injector'

import { loadStyling } from './bridge'
import { injectCss } from './injector'

/**
 * Convenience: fetch, compile, inject, and return the CSS in one call.
 *
 * The `from_cache` field on the returned result indicates whether the backend
 * served the last-good fallback; callers may choose to surface a warning.
 */
export async function loadAndInject(tool: string, version: string, input: string): Promise<string> {
  const result = await loadStyling(tool, version, input)
  injectCss(result.css)
  return result.css
}
