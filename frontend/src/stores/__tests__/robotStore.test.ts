import { describe, it, expect, beforeEach } from 'vitest'
import { useRobotStore } from '../robotStore'
import type { RobotState, TelemetryPayload } from '../robotStore'
import type { Pose } from '../../types/robot'

const makePose = (x = 0, y = 0, z = 0): Pose => ({
  position: { x, y, z },
  orientation: { x: 0, y: 0, z: 0, w: 1 },
})

const makeRobot = (id: string, name?: string): RobotState => ({
  id,
  name: name ?? id,
  status: 'idle',
  pose: makePose(),
  battery: 100,
  errors: [],
  lastUpdateMs: Date.now(),
})

describe('robotStore', () => {
  beforeEach(() => {
    // Reset the store before each test
    useRobotStore.setState({
      robots: new Map(),
      selectedRobotId: null,
    })
  })

  describe('setRobots', () => {
    it('adds robots to the map', () => {
      const robots = [makeRobot('r1', 'Robot 1'), makeRobot('r2', 'Robot 2')]
      useRobotStore.getState().setRobots(robots)

      const state = useRobotStore.getState()
      expect(state.robots.size).toBe(2)
      expect(state.robots.get('r1')?.name).toBe('Robot 1')
      expect(state.robots.get('r2')?.name).toBe('Robot 2')
    })
  })

  describe('selectRobot', () => {
    it('sets selectedRobotId', () => {
      useRobotStore.getState().selectRobot('r1')
      expect(useRobotStore.getState().selectedRobotId).toBe('r1')
    })

    it('clears selectedRobotId with null', () => {
      useRobotStore.getState().selectRobot('r1')
      useRobotStore.getState().selectRobot(null)
      expect(useRobotStore.getState().selectedRobotId).toBeNull()
    })
  })

  describe('updateFromTelemetry', () => {
    it('updates existing robot pose and battery', () => {
      const robots = [makeRobot('r1', 'Robot 1')]
      useRobotStore.getState().setRobots(robots)

      const payload: TelemetryPayload = {
        robot_id: 'r1',
        timestamp_ms: 1000,
        pose: makePose(5, 10, 0),
        velocity: {
          linear: { x: 1, y: 0, z: 0 },
          angular: { x: 0, y: 0, z: 0 },
        },
        battery_percent: 80,
        status: 'active',
        errors: [],
      }

      useRobotStore.getState().updateFromTelemetry(payload)

      const updated = useRobotStore.getState().robots.get('r1')
      expect(updated).toBeDefined()
      expect(updated!.pose.position.x).toBe(5)
      expect(updated!.pose.position.y).toBe(10)
      expect(updated!.battery).toBe(80)
      expect(updated!.status).toBe('active')
      // Name should be preserved from existing robot
      expect(updated!.name).toBe('Robot 1')
    })

    it('creates new robot if not exists', () => {
      const payload: TelemetryPayload = {
        robot_id: 'new-robot',
        timestamp_ms: 2000,
        pose: makePose(1, 2, 3),
        velocity: {
          linear: { x: 0, y: 0, z: 0 },
          angular: { x: 0, y: 0, z: 0 },
        },
        battery_percent: 95,
        status: 'idle',
        errors: [],
      }

      useRobotStore.getState().updateFromTelemetry(payload)

      const robot = useRobotStore.getState().robots.get('new-robot')
      expect(robot).toBeDefined()
      expect(robot!.id).toBe('new-robot')
      expect(robot!.name).toBe('new-robot') // name defaults to id when no existing robot
      expect(robot!.battery).toBe(95)
      expect(robot!.pose.position.x).toBe(1)
    })
  })
})
