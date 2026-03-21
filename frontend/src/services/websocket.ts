type MessageHandler = (data: unknown) => void

interface WebSocketManagerOptions {
  maxReconnectAttempts?: number
  initialReconnectDelay?: number
  maxReconnectDelay?: number
}

export class WebSocketManager {
  private ws: WebSocket | null = null
  private url: string = ''
  private token: string = ''
  private topics: Set<string> = new Set()
  private messageHandlers: Map<string, MessageHandler[]> = new Map()
  private reconnectAttempts: number = 0
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private isConnecting: boolean = false
  private shouldReconnect: boolean = true

  private maxReconnectAttempts: number
  private initialReconnectDelay: number
  private maxReconnectDelay: number

  constructor(options: WebSocketManagerOptions = {}) {
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 10
    this.initialReconnectDelay = options.initialReconnectDelay ?? 1000
    this.maxReconnectDelay = options.maxReconnectDelay ?? 30000
  }

  connect(url: string, token: string): void {
    this.url = url
    this.token = token
    this.shouldReconnect = true
    this.doConnect()
  }

  private doConnect(): void {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return
    }

    this.isConnecting = true

    const wsUrl = `${this.url}?token=${encodeURIComponent(this.token)}`
    this.ws = new WebSocket(wsUrl)

    this.ws.onopen = () => {
      this.isConnecting = false
      this.reconnectAttempts = 0
      console.log('[WS] Connected')

      // Re-subscribe to all topics
      for (const topic of this.topics) {
        this.sendSubscribe(topic)
      }
    }

    this.ws.onmessage = (event: MessageEvent) => {
      try {
        const message = JSON.parse(event.data as string) as {
          topic?: string
          type?: string
          [key: string]: unknown
        }
        const topic = message.topic ?? message.type ?? 'default'

        const handlers = this.messageHandlers.get(topic)
        if (handlers) {
          for (const handler of handlers) {
            handler(message)
          }
        }

        // Also notify wildcard handlers
        const wildcardHandlers = this.messageHandlers.get('*')
        if (wildcardHandlers) {
          for (const handler of wildcardHandlers) {
            handler(message)
          }
        }
      } catch (err) {
        console.error('[WS] Failed to parse message:', err)
      }
    }

    this.ws.onclose = () => {
      this.isConnecting = false
      console.log('[WS] Disconnected')

      if (this.shouldReconnect) {
        this.scheduleReconnect()
      }
    }

    this.ws.onerror = (error) => {
      console.error('[WS] Error:', error)
      this.isConnecting = false
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[WS] Max reconnect attempts reached')
      return
    }

    const delay = Math.min(
      this.initialReconnectDelay * Math.pow(2, this.reconnectAttempts),
      this.maxReconnectDelay,
    )

    console.log(
      `[WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1}/${this.maxReconnectAttempts})`,
    )

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++
      this.doConnect()
    }, delay)
  }

  subscribe(topic: string): void {
    this.topics.add(topic)
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.sendSubscribe(topic)
    }
  }

  unsubscribe(topic: string): void {
    this.topics.delete(topic)
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ action: 'unsubscribe', topic }))
    }
  }

  private sendSubscribe(topic: string): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ action: 'subscribe', topic }))
    }
  }

  onMessage(topic: string, handler: MessageHandler): () => void {
    const handlers = this.messageHandlers.get(topic) ?? []
    handlers.push(handler)
    this.messageHandlers.set(topic, handlers)

    return () => {
      const current = this.messageHandlers.get(topic)
      if (current) {
        this.messageHandlers.set(
          topic,
          current.filter((h) => h !== handler),
        )
      }
    }
  }

  disconnect(): void {
    this.shouldReconnect = false
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }

  get isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN
  }
}

// Singleton instance
export const wsManager = new WebSocketManager()
