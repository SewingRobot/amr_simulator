import { useState } from 'react'
import { useMissionStore } from '../../stores/missionStore'

interface MissionCreateFormProps {
  onClose: () => void
}

export function MissionCreateForm({ onClose }: MissionCreateFormProps) {
  const [startNodeId, setStartNodeId] = useState('')
  const [endNodeId, setEndNodeId] = useState('')
  const [priority, setPriority] = useState(1)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const createMission = useMissionStore((s) => s.createMission)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!startNodeId.trim() || !endNodeId.trim()) {
      setError('Start and end nodes are required')
      return
    }
    setIsSubmitting(true)
    setError(null)
    try {
      await createMission({
        start_node_id: startNodeId.trim(),
        end_node_id: endNodeId.trim(),
        priority,
      })
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create mission')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md border border-gray-700">
        <h2 className="text-lg font-semibold mb-4">Create Mission</h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Start Node ID</label>
            <input
              type="text"
              value={startNodeId}
              onChange={(e) => setStartNodeId(e.target.value)}
              className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-sm text-white focus:outline-none focus:border-blue-500"
              placeholder="e.g. node_a"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">End Node ID</label>
            <input
              type="text"
              value={endNodeId}
              onChange={(e) => setEndNodeId(e.target.value)}
              className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-sm text-white focus:outline-none focus:border-blue-500"
              placeholder="e.g. node_b"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">Priority</label>
            <input
              type="number"
              value={priority}
              onChange={(e) => setPriority(Number(e.target.value))}
              min={1}
              max={10}
              className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-sm text-white focus:outline-none focus:border-blue-500"
            />
          </div>
          {error && <p className="text-sm text-red-400">{error}</p>}
          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-500 rounded transition-colors disabled:opacity-50"
            >
              {isSubmitting ? 'Creating...' : 'Create Mission'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
