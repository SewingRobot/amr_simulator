export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting'

type MessageHandler = (data: unknown) => void
type ConnectionStateHandler = (state: ConnectionState) => void

interface WebSocketManagerOptions {
  maxReconnectAttempts?: number
  initialReconnectDelay?: number
  maxReconnectDelay?: number
}

export interface WsEnvelope {
  type: string
  topic?: string
  payload?: unknown
}

export class WebSocketManager {
  private ws: WebSocket | null = null
  private url: string = ''
  private token: string = ''
  private topics: Set<string> = new Set()
  private messageHandlers: Map<string, MessageHandler[]> = new Map()
  private connectionStateHandlers: Set<ConnectionStateHandler> = new Set()
  private reconnectAttempts: number = 0
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private shouldReconnect: boolean = true
  private _connectionState: ConnectionState = 'disconnected'
  private _reconnectCountdown: number = 0
  private countdownTimer: ReturnType<typeof setInterval> | null = null

  private maxReconnectAttempts: number
  private initialReconnectDelay: number
  private maxReconnectDelay: number

  constructor(options: WebSocketManagerOptions = {}) {
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 10
    this.initialReconnectDelay = options.initialReconnectDelay ?? 1000
    this.maxReconnectDelay = options.maxReconnectDelay ?? 30000
  }

  get connectionState(): ConnectionState {
    return this._connectionState
  }

  get reconnectCountdown(): number {
    return this._reconnectCountdown
  }

  private setConnectionState(state: ConnectionState): void {
    this._connectionState = state
    for (const handler of this.connectionStateHandlers) {
      handler(state)
    }
  }

  onConnectionStateChange(handler: ConnectionStateHandler): () => void {
    this.connectionStateHandlers.add(handler)
    return () => {
      this.connectionStateHandlers.delete(handler)
    }
  }

  connect(url: string, token: string): void {
    this.url = url
    this.token = token
    this.shouldReconnect = true
    this.doConnect()
  }

  private doConnect(): void {
    if (this._connectionState === 'connected') {
      return
    }

    if (this.reconnectAttempts > 0) {
      this.setConnectionState('reconnecting')
    } else {
      this.setConnectionState('connecting')
    }

    const wsUrl = `${this.url}?token=${encodeURIComponent(this.token)}`
    this.ws = new WebSocket(wsUrl)

    this.ws.onopen = () => {
      this.reconnectAttempts = 0
      this._reconnectCountdown = 0
      this.clearCountdownTimer()
      this.setConnectionState('connected')
      console.log('[WS] Connected')

      // Auto-subscribe to telemetry wildcard
      this.subscribe('telemetry:*')

      // Re-subscribe to all tracked topics
      for (const topic of this.topics) {
        this.sendSubscribe(topic)
      }
    }

    this.ws.onmessage = (event: MessageEvent) => {
      try {
        const envelope = JSON.parse(event.data as string) as WsEnvelope
        const msgType = envelope.type ?? 'unknown'
        const topic = envelope.topic ?? msgType

        // Dispatch to type-based handlers (e.g. "telemetry")
        this.dispatchToHandlers(msgType, envelope)

        // Dispatch to topic-based handlers if topic differs from type
        if (topic !== msgType) {
          this.dispatchToHandlers(topic, envelope)
        }

        // Dispatch to wildcard handlers
        this.dispatchToHandlers('*', envelope)
      } catch (err) {
        console.error('[WS] Failed to parse message:', err)
      }
    }

    this.ws.onclose = () => {
      console.log('[WS] Disconnected')
      this.setConnectionState('disconnected')

      if (this.shouldReconnect) {
        this.scheduleReconnect()
      }
    }

    this.ws.onerror = (error) => {
      console.error('[WS] Error:', error)
    }
  }

  private dispatchToHandlers(key: string, envelope: WsEnvelope): void {
    const handlers = this.messageHandlers.get(key)
    if (handlers) {
      for (const handler of handlers) {
        handler(envelope)
      }
    }
  }

  private clearCountdownTimer(): void {
    if (this.countdownTimer) {
      clearInterval(this.countdownTimer)
      this.countdownTimer = null
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[WS] Max reconnect attempts reached')
      this.setConnectionState('disconnected')
      return
    }

    const delay = Math.min(
      this.initialReconnectDelay * Math.pow(2, this.reconnectAttempts),
      this.maxReconnectDelay,
    )

    this._reconnectCountdown = Math.ceil(delay / 1000)
    this.setConnectionState('reconnecting')

    console.log(
      `[WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1}/${this.maxReconnectAttempts})`,
    )

    // Update countdown every second
    this.clearCountdownTimer()
    this.countdownTimer = setInterval(() => {
      this._reconnectCountdown = Math.max(0, this._reconnectCountdown - 1)
      // Notify state handlers so UI updates countdown
      for (const handler of this.connectionStateHandlers) {
        handler(this._connectionState)
      }
    }, 1000)

    this.reconnectTimer = setTimeout(() => {
      this.clearCountdownTimer()
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

  onMessage(key: string, handler: MessageHandler): () => void {
    const handlers = this.messageHandlers.get(key) ?? []
    handlers.push(handler)
    this.messageHandlers.set(key, handlers)

    return () => {
      const current = this.messageHandlers.get(key)
      if (current) {
        this.messageHandlers.set(
          key,
          current.filter((h) => h !== handler),
        )
      }
    }
  }

  disconnect(): void {
    this.shouldReconnect = false
    this.clearCountdownTimer()
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    this.setConnectionState('disconnected')
  }

  get isConnected(): boolean {
    return this._connectionState === 'connected'
  }
}

// Singleton instance
export const wsManager = new WebSocketManager()
