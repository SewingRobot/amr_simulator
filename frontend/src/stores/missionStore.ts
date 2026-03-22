import { create } from 'zustand'
import type { Mission, CreateMissionRequest } from '../types/mission'
import { api } from '../services/api'

interface MissionStore {
  missions: Mission[]
  selectedMissionId: string | null
  isLoading: boolean
  fetchMissions: (status?: string) => Promise<void>
  createMission: (req: CreateMissionRequest) => Promise<Mission>
  assignMission: (missionId: string, robotId: string) => Promise<void>
  cancelMission: (missionId: string) => Promise<void>
  selectMission: (id: string | null) => void
  updateMissionFromWs: (mission: Partial<Mission> & { id: string }) => void
}

export const useMissionStore = create<MissionStore>((set, get) => ({
  missions: [],
  selectedMissionId: null,
  isLoading: false,

  fetchMissions: async (status?: string) => {
    set({ isLoading: true })
    try {
      const query = status ? `?status=${status}` : ''
      const missions = await api.get<Mission[]>(`/missions${query}`)
      set({ missions, isLoading: false })
    } catch {
      set({ isLoading: false })
    }
  },

  createMission: async (req: CreateMissionRequest) => {
    const mission = await api.post<Mission>('/missions', req)
    set((state) => ({ missions: [mission, ...state.missions] }))
    return mission
  },

  assignMission: async (missionId: string, robotId: string) => {
    await api.post(`/missions/${missionId}/assign`, { robot_id: robotId })
    set((state) => ({
      missions: state.missions.map((m) =>
        m.id === missionId ? { ...m, status: 'assigned' as const, robot_id: robotId } : m,
      ),
    }))
  },

  cancelMission: async (missionId: string) => {
    await api.post(`/missions/${missionId}/cancel`, {})
    set((state) => ({
      missions: state.missions.map((m) =>
        m.id === missionId ? { ...m, status: 'cancelled' as const } : m,
      ),
    }))
  },

  selectMission: (id: string | null) => set({ selectedMissionId: id }),

  updateMissionFromWs: (update: Partial<Mission> & { id: string }) => {
    set((state) => {
      const exists = state.missions.some((m) => m.id === update.id)
      if (exists) {
        return {
          missions: state.missions.map((m) =>
            m.id === update.id ? { ...m, ...update } : m,
          ),
        }
      }
      return state
    })
  },
}))
