import { create } from 'zustand'
import type { Pose } from '../types/robot'

export interface TelemetryPayload {
  robot_id: string
  timestamp_ms: number
  pose: Pose
  velocity: {
    linear: { x: number; y: number; z: number }
    angular: { x: number; y: number; z: number }
  }
  battery_percent: number
  status: string
  errors: string[]
}

export interface RobotState {
  id: string
  name: string
  status: string
  pose: Pose
  battery: number
  errors: string[]
  lastUpdateMs: number
}

interface RobotStore {
  robots: Map<string, RobotState>
  selectedRobotId: string | null
  selectRobot: (id: string | null) => void
  updateRobotPose: (id: string, pose: Pose) => void
  setRobots: (robots: RobotState[]) => void
  updateFromTelemetry: (payload: TelemetryPayload) => void
}

export const useRobotStore = create<RobotStore>((set) => ({
  robots: new Map(),
  selectedRobotId: null,
  selectRobot: (id) => set({ selectedRobotId: id }),
  updateRobotPose: (id, pose) =>
    set((state) => {
      const robots = new Map(state.robots)
      const robot = robots.get(id)
      if (robot) {
        robots.set(id, { ...robot, pose })
      }
      return { robots }
    }),
  setRobots: (robotList) =>
    set({
      robots: new Map(robotList.map((r) => [r.id, r])),
    }),
  updateFromTelemetry: (payload) =>
    set((state) => {
      const robots = new Map(state.robots)
      const existing = robots.get(payload.robot_id)
      robots.set(payload.robot_id, {
        id: payload.robot_id,
        name: existing?.name ?? payload.robot_id,
        status: payload.status,
        pose: payload.pose,
        battery: payload.battery_percent,
        errors: payload.errors,
        lastUpdateMs: payload.timestamp_ms,
      })
      return { robots }
    }),
}))
