export type MissionStatus = 'created' | 'assigned' | 'executing' | 'completed' | 'failed' | 'cancelled'

export interface Mission {
  id: string
  robot_id: string | null
  status: MissionStatus
  priority: number
  start_node_id: string | null
  end_node_id: string | null
  path: string[]
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface CreateMissionRequest {
  start_node_id: string
  end_node_id: string
  priority?: number
}
