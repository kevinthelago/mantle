import { useEffect, useState } from 'react'
import { useCommand } from './lib/bridge'
import { commands } from './lib/bridge/bindings'

function Clock() {
  const [time, setTime] = useState(() => new Date().toLocaleTimeString())

  useEffect(() => {
    const id = setInterval(() => setTime(new Date().toLocaleTimeString()), 1000)
    return () => clearInterval(id)
  }, [])

  return (
    <span style={{ fontFamily: 'monospace', fontSize: 13 }}>{time}</span>
  )
}

function VersionBadge() {
  const { data: version, error } = useCommand(() => commands.getVersion())

  if (error || !version) return null
  return <span style={{ fontSize: 11, opacity: 0.6 }}>v{version}</span>
}

export default function App() {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        height: '100%',
        padding: '0 12px',
        color: '#e0e0e0',
        userSelect: 'none',
      }}
    >
      <VersionBadge />
      <Clock />
    </div>
  )
}
