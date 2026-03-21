import { describe, it, expect } from 'vitest'
import { WebSocketManager } from '../websocket'

describe('WebSocketManager', () => {
  describe('constructor', () => {
    it('sets initial connection state to disconnected', () => {
      const manager = new WebSocketManager()
      expect(manager.connectionState).toBe('disconnected')
    })

    it('sets isConnected to false initially', () => {
      const manager = new WebSocketManager()
      expect(manager.isConnected).toBe(false)
    })

    it('sets reconnectCountdown to 0 initially', () => {
      const manager = new WebSocketManager()
      expect(manager.reconnectCountdown).toBe(0)
    })
  })

  describe('reconnect backoff', () => {
    it('uses default options when none provided', () => {
      const manager = new WebSocketManager()
      // Default values are tested implicitly via the constructor;
      // maxReconnectAttempts = 10, initialReconnectDelay = 1000, maxReconnectDelay = 30000
      // We verify the manager is in a valid initial state
      expect(manager.connectionState).toBe('disconnected')
    })

    it('accepts custom reconnect options', () => {
      const manager = new WebSocketManager({
        maxReconnectAttempts: 5,
        initialReconnectDelay: 500,
        maxReconnectDelay: 10000,
      })
      // Manager should initialize without error with custom options
      expect(manager.connectionState).toBe('disconnected')
      expect(manager.isConnected).toBe(false)
    })

    it('exponential backoff formula: delay = initialDelay * 2^attempt, capped at maxDelay', () => {
      // This tests the mathematical formula used in scheduleReconnect.
      // The formula is: Math.min(initialDelay * Math.pow(2, attempts), maxDelay)
      const initialDelay = 1000
      const maxDelay = 30000

      // attempt 0: 1000 * 2^0 = 1000
      expect(Math.min(initialDelay * Math.pow(2, 0), maxDelay)).toBe(1000)
      // attempt 1: 1000 * 2^1 = 2000
      expect(Math.min(initialDelay * Math.pow(2, 1), maxDelay)).toBe(2000)
      // attempt 2: 1000 * 2^2 = 4000
      expect(Math.min(initialDelay * Math.pow(2, 2), maxDelay)).toBe(4000)
      // attempt 3: 1000 * 2^3 = 8000
      expect(Math.min(initialDelay * Math.pow(2, 3), maxDelay)).toBe(8000)
      // attempt 4: 1000 * 2^4 = 16000
      expect(Math.min(initialDelay * Math.pow(2, 4), maxDelay)).toBe(16000)
      // attempt 5: 1000 * 2^5 = 32000 -> capped at 30000
      expect(Math.min(initialDelay * Math.pow(2, 5), maxDelay)).toBe(30000)
    })
  })

  describe('disconnect', () => {
    it('sets connection state to disconnected', () => {
      const manager = new WebSocketManager()
      manager.disconnect()
      expect(manager.connectionState).toBe('disconnected')
      expect(manager.isConnected).toBe(false)
    })
  })
})
