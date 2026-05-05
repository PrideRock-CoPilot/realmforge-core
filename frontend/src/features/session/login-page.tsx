import { useCallback, useState, useId, type FormEvent } from 'react'
import { useSession } from '@/hooks/use-session'
import { LogIn } from 'lucide-react'

// ─────────────────────────────────────────────
// Login Page — first surface before authentication
// ─────────────────────────────────────────────
// ADR-0004: Bearer token auth — tokens are memory-only
// ADR-0005: Same-origin hosting — API_BASE_URL is empty
//
// Login is username + password only. Multi-tenant routing
// and project selection happen inside the app after login.

export function LoginPage() {
  const { login } = useSession()
  const titleId = useId()
  const [actorId, setActorId] = useState('')
  const [credential, setCredential] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = useCallback(
    async (e: FormEvent) => {
      e.preventDefault()
      setError(null)
      setIsSubmitting(true)

      try {
        // Default tenant — multi-tenant selection happens inside the app
        const response = await fetch('/v1/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            tenant_id: 'system',
            project_id: 'project-uat-001',
            actor_id: actorId.trim(),
            credential: credential,
            scope: 'read',
          }),
        })

        if (!response.ok) {
          const body = await response.json().catch(() => ({ detail: 'Login failed' }))
          throw new Error(body.detail || body.title || 'Login failed')
        }

        const data = await response.json()
        login(
          data.session_token,
          data.actor_id,
          new Date(data.expires_at),
        )
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Login failed')
      } finally {
        setIsSubmitting(false)
      }
    },
    [actorId, credential, login],
  )

  return (
    <div className="login-page">
      <div className="login-card">
        <div className="login-card__header">
          <LogIn size={24} aria-hidden="true" />
          <h1 id={titleId} className="login-card__title">
            RealmForge
          </h1>
          <p className="login-card__subtitle">
            Sign in to your governance workspace
          </p>
        </div>

        <form
          onSubmit={handleSubmit}
          aria-labelledby={titleId}
          className="login-card__form"
        >
          {error && (
            <div role="alert" className="login-card__error">
              {error}
            </div>
          )}

          <div className="login-card__field">
            <label htmlFor="actor-id" className="login-card__label">
              Username
            </label>
            <input
              id="actor-id"
              type="text"
              value={actorId}
              onChange={(e) => setActorId(e.target.value)}
              required
              autoComplete="username"
              className="login-card__input"
              placeholder="e.g. alice"
            />
          </div>

          <div className="login-card__field">
            <label htmlFor="credential" className="login-card__label">
              Password
            </label>
            <input
              id="credential"
              type="password"
              value={credential}
              onChange={(e) => setCredential(e.target.value)}
              required
              autoComplete="current-password"
              className="login-card__input"
              placeholder="Enter your password"
            />
          </div>

          <button
            type="submit"
            disabled={isSubmitting}
            className="login-card__submit"
            aria-busy={isSubmitting}
          >
            {isSubmitting ? 'Signing in\u2026' : 'Sign in'}
          </button>
        </form>
      </div>
    </div>
  )
}
