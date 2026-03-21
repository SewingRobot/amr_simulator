import type { Quaternion as QuatType } from '../types/robot'

/**
 * Convert ENU (East-North-Up) coordinates to Three.js coordinate system.
 * ENU: X=East, Y=North, Z=Up
 * Three.js: X=Right, Y=Up, Z=Out (towards camera)
 * Mapping: ENU(X,Y,Z) -> Three(X,Z,-Y) ... but we use (X,Z,Y) with grid on XZ plane
 *
 * Swap: Three.X = ENU.X, Three.Y = ENU.Z, Three.Z = -ENU.Y
 */
export function enuToThreeJS(
  x: number,
  y: number,
  z: number,
): [number, number, number] {
  return [x, z, -y]
}

/**
 * Convert Three.js coordinates back to ENU.
 */
export function threeJSToEnu(
  x: number,
  y: number,
  z: number,
): [number, number, number] {
  return [x, -z, y]
}

/**
 * Convert ENU quaternion to Three.js quaternion.
 * Applies the same axis swap as position coordinates.
 */
export function enuQuaternionToThreeJS(q: QuatType): QuatType {
  return {
    x: q.x,
    y: q.z,
    z: -q.y,
    w: q.w,
  }
}

/**
 * Linear interpolation between two values.
 */
export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t
}

/**
 * Clamp a value between min and max.
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}
