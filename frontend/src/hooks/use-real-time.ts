import { useEffect, useRef, useState } from 'react'
import { useSessionStore } from '@/store/session.store'

// ADR-0003: SSE as real-time transport. Browser EventSource handles reconnect
// automatically via Last-Event-ID. No WebSocket.

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error'

interface UseRealTimeOptions {
  enabled?: boolean
  maxItems?: number
  pauseWhenHidden?: boolean
}

interface UseRealTimeResult<T> {
  items: T[]
  status: ConnectionStatus
  lastEventId: string | null
}

export function useRealTime<T = unknown>(
  endpoint: string,
  options: UseRealTimeOptions = {},
): UseRealTimeResult<T> {
  const { enabled = true, maxItems = 100, pauseWhenHidden = true } = options
  const token = useSessionStore((s) => s.sessionToken)

  const [items, setItems] = useState<T[]>([])
  const [status, setStatus] = useState<ConnectionStatus>('disconnected')
  const [lastEventId, setLastEventId] = useState<string | null>(null)
  const esRef = useRef<EventSource | null>(null)

  useEffect(() => {
    if (!enabled || !token) {
      setStatus('disconnected')
      return
    }

    // Pause when tab is hidden — resume on visibility change
    if (pauseWhenHidden && document.hidden) {
      return
    }

    function connect(resumeId: string | null) {
      const url = resumeId
        ? `${endpoint}${endpoint.includes('?') ? '&' : '?'}_resume=${resumeId}`
        : endpoint

      setStatus('connecting')

      // EventSource does not support custom headers natively.
      // Bearer token is passed via a short-lived query param set server-side,
      // or via a session cookie. For Phase 1 (same-origin), the session is
      // validated from extensions. This is noted as a known limitation.
      // TODO (Fatima): confirm Phase 2 SSE auth mechanism (cookie vs signed URL).
      const es = new EventSource(url)
      esRef.current = es

      es.onopen = () => setStatus('connected')

      es.onmessage = (e: MessageEvent) => {
        setLastEventId(e.lastEventId)
        try {
          const parsed = JSON.parse(e.data) as T
          setItems((prev) => {
            const next = [...prev, parsed]
            return next.length > maxItems ? next.slice(-maxItems) : next
          })
        } catch {
          // Non-JSON SSE message (heartbeat or comment) — ignore
        }
      }

      es.onerror = () => {
        setStatus('error')
        es.close()
        // Browser's EventSource will reconnect automatically with Last-Event-ID
      }
    }

    connect(lastEventId)

    const handleVisibility = () => {
      if (!document.hidden && esRef.current?.readyState === EventSource.CLOSED) {
        connect(lastEventId)
      }
    }

    if (pauseWhenHidden) {
      document.addEventListener('visibilitychange', handleVisibility)
    }

    return () => {
      esRef.current?.close()
      esRef.current = null
      if (pauseWhenHidden) {
        document.removeEventListener('visibilitychange', handleVisibility)
      }
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [endpoint, enabled, token, pauseWhenHidden, maxItems])

  return { items, status, lastEventId }
}
