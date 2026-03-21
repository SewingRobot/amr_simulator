import { useRobotStore } from '../../stores/robotStore'
import { useWebSocketTelemetry } from '../../hooks/useWebSocketTelemetry'
import { RobotModel } from './RobotModel'

const ROBOT_COLORS = ['#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899']

export function RobotManager() {
  // Connect to WebSocket and feed telemetry into robotStore
  useWebSocketTelemetry()

  const robots = useRobotStore((state) => state.robots)
  const selectedRobotId = useRobotStore((state) => state.selectedRobotId)
  const selectRobot = useRobotStore((state) => state.selectRobot)

  const robotEntries = Array.from(robots.entries())

  return (
    <group>
      {robotEntries.map(([id, robot], index) => (
        <RobotModel
          key={id}
          pose={robot.pose}
          selected={selectedRobotId === id}
          color={ROBOT_COLORS[index % ROBOT_COLORS.length]}
          name={robot.name}
          onClick={() => selectRobot(id)}
        />
      ))}
    </group>
  )
}
