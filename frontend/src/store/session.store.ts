import { create } from 'zustand'

// ADR-0004: session_token is the Bearer credential.
// Stored in memory (Zustand), never persisted to localStorage — token
// is lost on page refresh and requires re-login. Intentional: short-lived
// sessions match the governance model.
interface SessionState {
  sessionToken: string | null
  actorId: string | null
  expiresAt: Date | null

  setSession: (token: string, actorId: string, expiresAt: Date) => void
  clearSession: () => void
  isAuthenticated: () => boolean
}

export const useSessionStore = create<SessionState>((set, get) => ({
  sessionToken: null,
  actorId: null,
  expiresAt: null,

  setSession: (token, actorId, expiresAt) =>
    set({ sessionToken: token, actorId, expiresAt }),

  clearSession: () =>
    set({ sessionToken: null, actorId: null, expiresAt: null }),

  isAuthenticated: () => {
    const { sessionToken, expiresAt } = get()
    if (!sessionToken || !expiresAt) return false
    return new Date() < expiresAt
  },
}))
