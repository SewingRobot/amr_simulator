import { useEffect, useState } from 'react'
import { useMissionStore } from '../../stores/missionStore'
import { MissionCreateForm } from './MissionCreateForm'
import { MissionDetail } from './MissionDetail'
import type { MissionStatus } from '../../types/mission'

type FilterStatus = 'all' | 'active' | 'completed'

const STATUS_BADGE_COLORS: Record<MissionStatus, string> = {
  created: 'bg-gray-500/20 text-gray-400',
  assigned: 'bg-blue-500/20 text-blue-400',
  executing: 'bg-yellow-500/20 text-yellow-400',
  completed: 'bg-green-500/20 text-green-400',
  failed: 'bg-red-500/20 text-red-400',
  cancelled: 'bg-gray-500/20 text-gray-400',
}

function filterMissions(filter: FilterStatus) {
  switch (filter) {
    case 'active':
      return (status: MissionStatus) =>
        status === 'created' || status === 'assigned' || status === 'executing'
    case 'completed':
      return (status: MissionStatus) =>
        status === 'completed' || status === 'failed' || status === 'cancelled'
    default:
      return () => true
  }
}

export function MissionListPage() {
  const missions = useMissionStore((s) => s.missions)
  const isLoading = useMissionStore((s) => s.isLoading)
  const fetchMissions = useMissionStore((s) => s.fetchMissions)
  const selectMission = useMissionStore((s) => s.selectMission)
  const selectedMissionId = useMissionStore((s) => s.selectedMissionId)

  const [filter, setFilter] = useState<FilterStatus>('all')
  const [showCreateForm, setShowCreateForm] = useState(false)

  useEffect(() => {
    fetchMissions()
  }, [fetchMissions])

  const filteredMissions = missions.filter((m) => filterMissions(filter)(m.status))

  return (
    <div className="flex h-full">
      {/* Mission list */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Toolbar */}
        <div className="flex items-center justify-between p-4 border-b border-gray-800">
          <div className="flex items-center gap-3">
            <h2 className="text-sm font-semibold">Missions</h2>
            <select
              value={filter}
              onChange={(e) => setFilter(e.target.value as FilterStatus)}
              className="bg-gray-800 border border-gray-700 rounded px-2 py-1 text-xs text-gray-300 focus:outline-none"
            >
              <option value="all">All</option>
              <option value="active">Active</option>
              <option value="completed">Completed</option>
            </select>
          </div>
          <button
            onClick={() => setShowCreateForm(true)}
            className="px-3 py-1.5 text-xs bg-blue-600 hover:bg-blue-500 rounded transition-colors"
          >
            Create Mission
          </button>
        </div>

        {/* Table */}
        <div className="flex-1 overflow-auto">
          {isLoading ? (
            <div className="p-4 text-sm text-gray-500">Loading missions...</div>
          ) : filteredMissions.length === 0 ? (
            <div className="p-4 text-sm text-gray-500">No missions found.</div>
          ) : (
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-xs text-gray-500 border-b border-gray-800">
                  <th className="px-4 py-2 font-medium">Status</th>
                  <th className="px-4 py-2 font-medium">Priority</th>
                  <th className="px-4 py-2 font-medium">Route</th>
                  <th className="px-4 py-2 font-medium">Robot</th>
                  <th className="px-4 py-2 font-medium">Created</th>
                </tr>
              </thead>
              <tbody>
                {filteredMissions.map((mission) => (
                  <tr
                    key={mission.id}
                    onClick={() => selectMission(mission.id)}
                    className={`border-b border-gray-800/50 cursor-pointer transition-colors ${
                      selectedMissionId === mission.id
                        ? 'bg-blue-600/10'
                        : 'hover:bg-gray-800/50'
                    }`}
                  >
                    <td className="px-4 py-2.5">
                      <span
                        className={`inline-block px-2 py-0.5 rounded text-xs capitalize ${STATUS_BADGE_COLORS[mission.status]}`}
                      >
                        {mission.status}
                      </span>
                    </td>
                    <td className="px-4 py-2.5">{mission.priority}</td>
                    <td className="px-4 py-2.5 font-mono text-xs">
                      {mission.start_node_id ?? '?'} &rarr; {mission.end_node_id ?? '?'}
                    </td>
                    <td className="px-4 py-2.5 text-gray-400">
                      {mission.robot_id ?? '-'}
                    </td>
                    <td className="px-4 py-2.5 text-xs text-gray-500">
                      {new Date(mission.created_at).toLocaleString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>

      {/* Detail panel */}
      {selectedMissionId && (
        <aside className="w-80 flex-shrink-0 border-l border-gray-800 overflow-auto bg-gray-900">
          <MissionDetail />
        </aside>
      )}

      {/* Create form modal */}
      {showCreateForm && (
        <MissionCreateForm
          onClose={() => {
            setShowCreateForm(false)
            fetchMissions()
          }}
        />
      )}
    </div>
  )
}
