import type { Pose, Vector3 } from './robot'

export interface TelemetryMessage {
  robotId: string
  timestamp: number
  type: TelemetryType
  data: TelemetryData
}

export type TelemetryType = 'pose' | 'battery' | 'status' | 'lidar' | 'odom'

export type TelemetryData =
  | PoseTelemetry
  | BatteryTelemetry
  | StatusTelemetry
  | LidarTelemetry
  | OdomTelemetry

export interface PoseTelemetry {
  type: 'pose'
  pose: Pose
}

export interface BatteryTelemetry {
  type: 'battery'
  percentage: number
  voltage: number
  charging: boolean
}

export interface StatusTelemetry {
  type: 'status'
  status: string
  message?: string
}

export interface LidarTelemetry {
  type: 'lidar'
  points: Vector3[]
  frameId: string
}

export interface OdomTelemetry {
  type: 'odom'
  pose: Pose
  linearVelocity: Vector3
  angularVelocity: Vector3
}
