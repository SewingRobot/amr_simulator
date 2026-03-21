import { useEffect, useRef, useCallback, useState } from 'react'
import { wsManager } from '../services/websocket'

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
  const [isConnected, setIsConnected] = useState(false)
  const onMessageRef = useRef(onMessage)
  onMessageRef.current = onMessage

  const connect = useCallback(() => {
    wsManager.connect(url, token)
  }, [url, token])

  const disconnect = useCallback(() => {
    wsManager.disconnect()
    setIsConnected(false)
  }, [])

  useEffect(() => {
    if (!enabled) return

    connect()

    // Poll connection status
    const interval = setInterval(() => {
      setIsConnected(wsManager.isConnected)
    }, 1000)

    return () => {
      clearInterval(interval)
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
    isConnected,
    connect,
    disconnect,
  }
}
