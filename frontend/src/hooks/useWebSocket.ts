import { useEffect, useRef, useCallback, useState } from 'react'
import { wsManager, type ConnectionState } from '../services/websocket'

interface UseWebSocketOptions {
  url: string
  token: string
  topics?: string[]
  onMessage?: (data: unknown) => void
  enabled?: boolean
}

export function useWebSocket({
  url,
  token,
  topics = [],
  onMessage,
  enabled = true,
}: UseWebSocketOptions) {
  const [connectionState, setConnectionState] = useState<ConnectionState>(
    wsManager.connectionState,
  )
  const onMessageRef = useRef(onMessage)
  onMessageRef.current = onMessage

  const connect = useCallback(() => {
    wsManager.connect(url, token)
  }, [url, token])

  const disconnect = useCallback(() => {
    wsManager.disconnect()
  }, [])

  useEffect(() => {
    if (!enabled) return

    connect()

    const unsubState = wsManager.onConnectionStateChange((state) => {
      setConnectionState(state)
    })

    return () => {
      unsubState()
      disconnect()
    }
  }, [enabled, connect, disconnect])

  // Subscribe to topics
  useEffect(() => {
    if (!enabled) return

    for (const topic of topics) {
      wsManager.subscribe(topic)
    }

    return () => {
      for (const topic of topics) {
        wsManager.unsubscribe(topic)
      }
    }
  }, [topics, enabled])

  // Register message handler
  useEffect(() => {
    if (!enabled || !onMessageRef.current) return

    const unsubscribe = wsManager.onMessage('*', (data) => {
      onMessageRef.current?.(data)
    })

    return unsubscribe
  }, [enabled])

  return {
    isConnected: connectionState === 'connected',
    connectionState,
    connect,
    disconnect,
  }
}
