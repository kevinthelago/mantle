import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
// Side-effect import: registers the 'widgets' surface with the surface registry.
import './widgets'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
