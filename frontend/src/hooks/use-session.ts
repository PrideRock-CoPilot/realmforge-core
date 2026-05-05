import { useCallback } from 'react'
import { useSessionStore } from '@/store/session.store'

// ─────────────────────────────────────────────
// Session hook — wraps Zustand session store
// ─────────────────────────────────────────────

export function useSession() {
  const sessionToken = useSessionStore((s) => s.sessionToken)
  const actorId = useSessionStore((s) => s.actorId)
  const expiresAt = useSessionStore((s) => s.expiresAt)
  const setSession = useSessionStore((s) => s.setSession)
  const clearSession = useSessionStore((s) => s.clearSession)
  const isAuthenticated = useSessionStore((s) => s.isAuthenticated())

  const login = useCallback(
    (token: string, actor: string, expires: Date) => {
      setSession(token, actor, expires)
    },
    [setSession],
  )

  const logout = useCallback(() => {
    clearSession()
  }, [clearSession])

  return {
    sessionToken,
    actorId,
    expiresAt,
    isAuthenticated,
    login,
    logout,
  }
}
