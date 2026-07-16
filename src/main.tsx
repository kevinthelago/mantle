import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
// Side-effect imports: each surface module self-registers with the surface
// registry (src/lib/surfaces.ts) at load time.  App.tsx then routes to the
// surface whose id matches the current Tauri window label.  Every window loads
// the same bundle, so all surfaces must be registered regardless of which
// window this instance renders.
import './bar/shell'
import './launcher'
import './widgets'
import './notifications'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
