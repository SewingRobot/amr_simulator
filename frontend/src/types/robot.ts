export interface Vector3 {
  x: number
  y: number
  z: number
}

export interface Quaternion {
  x: number
  y: number
  z: number
  w: number
}

export interface Pose {
  position: Vector3
  orientation: Quaternion
}

export interface Robot {
  id: string
  name: string
  model: string
  status: RobotStatus
  pose: Pose
  battery: number
  velocity: Vector3
  lastSeen: number
}

export type RobotStatus =
  | 'idle'
  | 'active'
  | 'charging'
  | 'error'
  | 'offline'

export interface RobotCommand {
  robotId: string
  type: 'navigate' | 'stop' | 'dock' | 'undock'
  payload?: Record<string, unknown>
}
