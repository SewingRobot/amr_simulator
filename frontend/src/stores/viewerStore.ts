import { create } from 'zustand'

type CameraMode = 'orbit' | 'topdown' | 'follow'

interface ViewerStore {
  cameraMode: CameraMode
  setCameraMode: (mode: CameraMode) => void
  showGrid: boolean
  toggleGrid: () => void
  showAxes: boolean
  toggleAxes: () => void
}

export const useViewerStore = create<ViewerStore>((set) => ({
  cameraMode: 'orbit',
  setCameraMode: (mode) => set({ cameraMode: mode }),
  showGrid: true,
  toggleGrid: () => set((state) => ({ showGrid: !state.showGrid })),
  showAxes: true,
  toggleAxes: () => set((state) => ({ showAxes: !state.showAxes })),
}))
