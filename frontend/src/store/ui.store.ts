import { create } from 'zustand'

interface UiStore {
  sidebarOpen: boolean
  activeBoard: string | null
  sidebarSection: string

  toggleSidebar: () => void
  setSidebarOpen: (open: boolean) => void
  setActiveBoard: (boardId: string | null) => void
  setSidebarSection: (section: string) => void
}

export const useUiStore = create<UiStore>((set) => ({
  sidebarOpen: true,
  activeBoard: null,
  sidebarSection: 'boards',

  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
  setActiveBoard: (boardId) => set({ activeBoard: boardId }),
  setSidebarSection: (section) => set({ sidebarSection: section }),
}))
