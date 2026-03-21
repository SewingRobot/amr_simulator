import { create } from 'zustand'
import type { Pose } from '../types/robot'

export interface RobotState {
  id: string
  name: string
  status: string
  pose: Pose
  battery: number
}

interface RobotStore {
  robots: Map<string, RobotState>
  selectedRobotId: string | null
  selectRobot: (id: string | null) => void
  updateRobotPose: (id: string, pose: Pose) => void
  setRobots: (robots: RobotState[]) => void
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
}))
