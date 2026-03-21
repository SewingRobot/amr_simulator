import type { Vector3 } from './robot'

export interface MapMetadata {
  id: string
  name: string
  resolution: number // meters per cell
  width: number
  height: number
  origin: Vector3
}

export interface Wall {
  start: Vector3
  end: Vector3
  height: number
}

export interface Obstacle {
  id: string
  position: Vector3
  size: Vector3
  type: 'static' | 'dynamic'
}

export interface EnvironmentMap {
  metadata: MapMetadata
  walls: Wall[]
  obstacles: Obstacle[]
}
