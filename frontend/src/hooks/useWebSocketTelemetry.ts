import { useEffect, useState, useCallback } from 'react'
import {
  wsManager,
  type ConnectionState,
  type WsEnvelope,
} from '../services/websocket'
import { useRobotStore, type TelemetryPayload } from '../stores/robotStore'

const WS_URL = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/api/ws`

export function useWebSocketTelemetry() {
  const [connectionState, setConnectionState] = useState<ConnectionState>(
    wsManager.connectionState,
  )
  const [error, setError] = useState<string | null>(null)
  const [reconnectCountdown, setReconnectCountdown] = useState(0)

  const updateFromTelemetry = useRobotStore((s) => s.updateFromTelemetry)

  const handleTelemetryMessage = useCallback(
    (data: unknown) => {
      const envelope = data as WsEnvelope
      if (envelope.type === 'telemetry' && envelope.payload) {
        const payload = envelope.payload as TelemetryPayload
        updateFromTelemetry(payload)
      }
    },
    [updateFromTelemetry],
  )

  useEffect(() => {
    // Track connection state
    const unsubState = wsManager.onConnectionStateChange((state) => {
      setConnectionState(state)
      setReconnectCountdown(wsManager.reconnectCountdown)
      if (state === 'connected') {
        setError(null)
      } else if (state === 'disconnected' && wsManager.reconnectCountdown === 0) {
        setError('Connection lost')
      }
    })

    // Listen for telemetry messages
    const unsubMsg = wsManager.onMessage('telemetry', handleTelemetryMessage)

    // Connect with empty token (auth handled elsewhere or not required for now)
    wsManager.connect(WS_URL, '')

    return () => {
      unsubState()
      unsubMsg()
      wsManager.disconnect()
    }
  }, [handleTelemetryMessage])

  return {
    isConnected: connectionState === 'connected',
    connectionState,
    reconnectCountdown,
    error,
  }
}
