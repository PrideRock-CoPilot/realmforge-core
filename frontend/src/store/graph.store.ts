import { create } from 'zustand'

interface GraphStore {
  selectedNodeId: string | null
  zoomLevel: number
  selectNode: (id: string | null) => void
  setZoom: (level: number) => void
}

export const useGraphStore = create<GraphStore>((set) => ({
  selectedNodeId: null,
  zoomLevel: 1,
  selectNode: (id) => set({ selectedNodeId: id }),
  setZoom: (level) => set({ zoomLevel: level }),
}))
