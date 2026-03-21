import { describe, it, expect } from 'vitest'
import { enuToThreeJS, lerp, clamp } from '../math'

describe('enuToThreeJS', () => {
  it('swaps ENU Y to Three.js Z (negated) and ENU Z to Three.js Y', () => {
    // ENU(X, Y, Z) -> Three(X, Z, -Y)
    const [tx, ty, tz] = enuToThreeJS(1, 2, 3)
    expect(tx).toBe(1)  // X stays the same
    expect(ty).toBe(3)  // Three.Y = ENU.Z
    expect(tz).toBe(-2) // Three.Z = -ENU.Y
  })

  it('handles zero values', () => {
    const [tx, ty, tz] = enuToThreeJS(0, 0, 0)
    expect(tx).toBe(0)
    expect(ty).toBe(0)
    expect(tz).toBe(-0) // -0 is equal to 0
  })

  it('handles negative values', () => {
    const [tx, ty, tz] = enuToThreeJS(-1, -2, -3)
    expect(tx).toBe(-1)
    expect(ty).toBe(-3)
    expect(tz).toBe(2)
  })
})

describe('lerp', () => {
  it('returns a at t=0', () => {
    expect(lerp(10, 20, 0)).toBe(10)
  })

  it('returns midpoint at t=0.5', () => {
    expect(lerp(10, 20, 0.5)).toBe(15)
  })

  it('returns b at t=1', () => {
    expect(lerp(10, 20, 1)).toBe(20)
  })

  it('extrapolates beyond 0-1 range', () => {
    expect(lerp(0, 10, 2)).toBe(20)
    expect(lerp(0, 10, -1)).toBe(-10)
  })
})

describe('clamp', () => {
  it('returns value when within range', () => {
    expect(clamp(5, 0, 10)).toBe(5)
  })

  it('clamps to min when below', () => {
    expect(clamp(-5, 0, 10)).toBe(0)
  })

  it('clamps to max when above', () => {
    expect(clamp(15, 0, 10)).toBe(10)
  })

  it('handles equal min and max', () => {
    expect(clamp(5, 3, 3)).toBe(3)
  })

  it('returns boundary values exactly', () => {
    expect(clamp(0, 0, 10)).toBe(0)
    expect(clamp(10, 0, 10)).toBe(10)
  })
})
