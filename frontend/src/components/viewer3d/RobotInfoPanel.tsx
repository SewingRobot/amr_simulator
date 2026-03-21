import { useRobotStore } from '../../stores/robotStore'

export function RobotInfoPanel() {
  const selectedRobotId = useRobotStore((state) => state.selectedRobotId)
  const robots = useRobotStore((state) => state.robots)
  const selectRobot = useRobotStore((state) => state.selectRobot)

  if (!selectedRobotId) return null

  const robot = robots.get(selectedRobotId)
  if (!robot) return null

  return (
    <div className="absolute top-4 right-4 bg-gray-900/90 backdrop-blur border border-gray-700 rounded-lg p-4 w-72 text-white shadow-xl">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold">{robot.name}</h3>
        <button
          onClick={() => selectRobot(null)}
          className="text-gray-400 hover:text-white text-xs"
        >
          Close
        </button>
      </div>

      <div className="space-y-2 text-xs">
        <div className="flex justify-between">
          <span className="text-gray-400">Status</span>
          <span
            className={`font-medium ${
              robot.status === 'active'
                ? 'text-green-400'
                : robot.status === 'error'
                  ? 'text-red-400'
                  : 'text-yellow-400'
            }`}
          >
            {robot.status}
          </span>
        </div>

        <div className="flex justify-between">
          <span className="text-gray-400">Position</span>
          <span className="font-mono">
            ({robot.pose.position.x.toFixed(2)},{' '}
            {robot.pose.position.y.toFixed(2)},{' '}
            {robot.pose.position.z.toFixed(2)})
          </span>
        </div>

        <div className="flex justify-between">
          <span className="text-gray-400">Battery</span>
          <span
            className={`font-medium ${
              robot.battery > 50
                ? 'text-green-400'
                : robot.battery > 20
                  ? 'text-yellow-400'
                  : 'text-red-400'
            }`}
          >
            {robot.battery}%
          </span>
        </div>

        <div className="flex justify-between">
          <span className="text-gray-400">ID</span>
          <span className="font-mono text-gray-500">{robot.id}</span>
        </div>
      </div>
    </div>
  )
}
