import { useState } from 'react'
import { useMissionStore } from '../../stores/missionStore'
import { useRobotStore } from '../../stores/robotStore'
import type { MissionStatus } from '../../types/mission'

const STATUS_STEPS: MissionStatus[] = ['created', 'assigned', 'executing', 'completed']

const STATUS_COLORS: Record<MissionStatus, string> = {
  created: 'bg-gray-500',
  assigned: 'bg-blue-500',
  executing: 'bg-yellow-500',
  completed: 'bg-green-500',
  failed: 'bg-red-500',
  cancelled: 'bg-gray-500',
}

function getStepIndex(status: MissionStatus): number {
  if (status === 'failed' || status === 'cancelled') return -1
  return STATUS_STEPS.indexOf(status)
}

export function MissionDetail() {
  const selectedMissionId = useMissionStore((s) => s.selectedMissionId)
  const missions = useMissionStore((s) => s.missions)
  const assignMission = useMissionStore((s) => s.assignMission)
  const cancelMission = useMissionStore((s) => s.cancelMission)
  const selectMission = useMissionStore((s) => s.selectMission)
  const robots = useRobotStore((s) => s.robots)

  const [selectedRobotId, setSelectedRobotId] = useState('')
  const [isAssigning, setIsAssigning] = useState(false)

  const mission = missions.find((m) => m.id === selectedMissionId)

  if (!mission) {
    return (
      <div className="p-6 text-gray-500 text-sm">
        Select a mission to view details.
      </div>
    )
  }

  const currentStepIndex = getStepIndex(mission.status)
  const isTerminal = mission.status === 'failed' || mission.status === 'cancelled' || mission.status === 'completed'
  const canAssign = mission.status === 'created'
  const canCancel = mission.status === 'created' || mission.status === 'assigned' || mission.status === 'executing'

  const handleAssign = async () => {
    if (!selectedRobotId) return
    setIsAssigning(true)
    try {
      await assignMission(mission.id, selectedRobotId)
      setSelectedRobotId('')
    } finally {
      setIsAssigning(false)
    }
  }

  const handleCancel = async () => {
    await cancelMission(mission.id)
  }

  const robotList = Array.from(robots.values())

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Mission Detail</h2>
        <button
          onClick={() => selectMission(null)}
          className="text-sm text-gray-400 hover:text-white"
        >
          Close
        </button>
      </div>

      {/* Status badge */}
      <div className="flex items-center gap-2">
        <span className={`inline-block w-2.5 h-2.5 rounded-full ${STATUS_COLORS[mission.status]}`} />
        <span className="text-sm font-medium capitalize">{mission.status}</span>
      </div>

      {/* Status timeline */}
      <div className="space-y-1">
        <p className="text-xs text-gray-500 uppercase tracking-wider">Progress</p>
        <div className="flex items-center gap-1">
          {STATUS_STEPS.map((step, i) => {
            const isActive = !isTerminal && i <= currentStepIndex
            const isFailed = mission.status === 'failed'
            const isCancelled = mission.status === 'cancelled'
            let color = 'bg-gray-700'
            if (isActive) color = 'bg-blue-500'
            if (isFailed && i <= 2) color = i === 2 ? 'bg-red-500' : 'bg-gray-600'
            if (isCancelled) color = 'bg-gray-600'
            if (mission.status === 'completed' && i <= 3) color = 'bg-green-500'

            return (
              <div key={step} className="flex-1 flex flex-col items-center gap-1">
                <div className={`h-2 w-full rounded ${color}`} />
                <span className="text-[10px] text-gray-500 capitalize">{step}</span>
              </div>
            )
          })}
        </div>
      </div>

      {/* Info */}
      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-gray-400">ID</span>
          <span className="font-mono text-xs">{mission.id}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Priority</span>
          <span>{mission.priority}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Robot</span>
          <span>{mission.robot_id ?? 'Unassigned'}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Path</span>
          <span className="font-mono text-xs">
            {mission.start_node_id ?? '?'} &rarr; {mission.end_node_id ?? '?'}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Created</span>
          <span className="text-xs">{new Date(mission.created_at).toLocaleString()}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Updated</span>
          <span className="text-xs">{new Date(mission.updated_at).toLocaleString()}</span>
        </div>
      </div>

      {/* Actions */}
      <div className="space-y-3 pt-2 border-t border-gray-700">
        {canAssign && (
          <div className="space-y-2">
            <p className="text-sm text-gray-400">Assign to Robot</p>
            <div className="flex gap-2">
              <select
                value={selectedRobotId}
                onChange={(e) => setSelectedRobotId(e.target.value)}
                className="flex-1 bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500"
              >
                <option value="">Select a robot...</option>
                {robotList.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.name} ({r.status})
                  </option>
                ))}
              </select>
              <button
                onClick={handleAssign}
                disabled={!selectedRobotId || isAssigning}
                className="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 rounded transition-colors disabled:opacity-50"
              >
                Assign
              </button>
            </div>
          </div>
        )}
        {canCancel && (
          <button
            onClick={handleCancel}
            className="w-full px-3 py-2 text-sm bg-red-600/20 text-red-400 hover:bg-red-600/30 rounded transition-colors"
          >
            Cancel Mission
          </button>
        )}
      </div>
    </div>
  )
}
