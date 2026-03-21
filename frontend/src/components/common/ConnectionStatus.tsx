import { useState, useEffect } from 'react'
import {
  wsManager,
  type ConnectionState,
} from '../../services/websocket'

const STATE_CONFIG: Record<
  ConnectionState,
  { color: string; bgColor: string; label: string }
> = {
  connected: {
    color: 'bg-green-500',
    bgColor: 'bg-green-500/10 border-green-500/30',
    label: 'Connected',
  },
  connecting: {
    color: 'bg-yellow-500',
    bgColor: 'bg-yellow-500/10 border-yellow-500/30',
    label: 'Connecting...',
  },
  reconnecting: {
    color: 'bg-yellow-500',
    bgColor: 'bg-yellow-500/10 border-yellow-500/30',
    label: 'Reconnecting',
  },
  disconnected: {
    color: 'bg-red-500',
    bgColor: 'bg-red-500/10 border-red-500/30',
    label: 'Disconnected',
  },
}

export function ConnectionStatus() {
  const [state, setState] = useState<ConnectionState>(wsManager.connectionState)
  const [countdown, setCountdown] = useState(0)

  useEffect(() => {
    const unsub = wsManager.onConnectionStateChange((newState) => {
      setState(newState)
      setCountdown(wsManager.reconnectCountdown)
    })
    return unsub
  }, [])

  const config = STATE_CONFIG[state]
  const showCountdown = state === 'reconnecting' && countdown > 0

  return (
    <div
      className={`fixed top-3 right-3 z-50 flex items-center gap-2 px-3 py-1.5 rounded-full border text-xs ${config.bgColor}`}
    >
      <span className={`inline-block w-2 h-2 rounded-full ${config.color} ${
        state === 'connecting' || state === 'reconnecting' ? 'animate-pulse' : ''
      }`} />
      <span className="text-gray-200">
        {config.label}
        {showCountdown && ` (${countdown}s)`}
      </span>
    </div>
  )
}
