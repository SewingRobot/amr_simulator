import { create } from 'zustand'

interface AuthStore {
  token: string | null
  userId: string | null
  setAuth: (token: string, userId: string) => void
  clearAuth: () => void
  isAuthenticated: () => boolean
}

export const useAuthStore = create<AuthStore>((set, get) => ({
  token: null,
  userId: null,
  setAuth: (token, userId) => set({ token, userId }),
  clearAuth: () => set({ token: null, userId: null }),
  isAuthenticated: () => get().token !== null,
}))
