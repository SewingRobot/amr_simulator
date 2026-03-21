import { create } from 'zustand'

type CameraMode = 'orbit' | 'topdown' | 'follow'
type ViewMode = '2d' | '3d'

interface ViewerStore {
  cameraMode: CameraMode
  setCameraMode: (mode: CameraMode) => void
  viewMode: ViewMode
  toggleViewMode: () => void
  setViewMode: (mode: ViewMode) => void
  showGrid: boolean
  toggleGrid: () => void
  showAxes: boolean
  toggleAxes: () => void
}

export type { ViewMode }

export const useViewerStore = create<ViewerStore>((set) => ({
  cameraMode: 'orbit',
  setCameraMode: (mode) => set({ cameraMode: mode }),
  viewMode: '3d',
  toggleViewMode: () =>
    set((state) => ({ viewMode: state.viewMode === '3d' ? '2d' : '3d' })),
  setViewMode: (mode) => set({ viewMode: mode }),
  showGrid: true,
  toggleGrid: () => set((state) => ({ showGrid: !state.showGrid })),
  showAxes: true,
  toggleAxes: () => set((state) => ({ showAxes: !state.showAxes })),
}))
