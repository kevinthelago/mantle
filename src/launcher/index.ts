import { registerSurface } from '../lib/surfaces'
import { LauncherApp } from './LauncherApp'

// Register so App.tsx renders LauncherApp when the "launcher" window is active.
// main.tsx must import this module as a side-effect for registration to take effect.
registerSurface({ id: 'launcher', component: LauncherApp })

export { LauncherApp } from './LauncherApp'
export type { AppEntry, Mode, SearchResult } from './types'
