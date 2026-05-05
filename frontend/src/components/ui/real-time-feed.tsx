import { useState } from 'react'
import { useRealTime, type ConnectionStatus } from '@/hooks/use-real-time'

// Component contract (spec doc §23):
// - role="log" aria-live="polite" — screen reader announces new events in order
// - "pause updates" toggle is keyboard-accessible
// - Visible + announced status when SSE connection drops
// - aria-label is required — describes the live region to assistive technology

interface RealTimeFeedProps {
  endpoint: string
  renderItem: (event: unknown, index: number) => React.ReactNode
  maxItems?: number
  pauseWhenHidden?: boolean
  'aria-label': string
}

export function RealTimeFeed({
  endpoint,
  renderItem,
  maxItems = 100,
  pauseWhenHidden = true,
  'aria-label': ariaLabel,
}: RealTimeFeedProps) {
  const [paused, setPaused] = useState(false)

  const { items, status } = useRealTime(paused ? '' : endpoint, {
    enabled: !paused,
    maxItems,
    pauseWhenHidden,
  })

  return (
    <section aria-label={ariaLabel}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
        <ConnectionStatusBadge status={status} />
        <button
          type="button"
          onClick={() => setPaused((p) => !p)}
          aria-pressed={paused}
          aria-label={paused ? 'Resume live updates' : 'Pause live updates'}
        >
          {paused ? 'Resume updates' : 'Pause updates'}
        </button>
      </div>

      {/* role="log" + aria-live="polite": screen reader announces additions in order */}
      <div
        role="log"
        aria-live="polite"
        aria-label={ariaLabel}
        aria-atomic="false"
      >
        {items.length === 0 ? (
          <p aria-live="polite">
            {status === 'connecting' ? 'Connecting…' : 'No events yet.'}
          </p>
        ) : (
          items.map((item, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: append-only log, index is stable
            <div key={i}>{renderItem(item, i)}</div>
          ))
        )}
      </div>
    </section>
  )
}

function ConnectionStatusBadge({ status }: { status: ConnectionStatus }) {
  const labels: Record<ConnectionStatus, string> = {
    connecting: 'Connecting to live feed…',
    connected: 'Live',
    disconnected: 'Disconnected',
    error: 'Connection lost — reconnecting…',
  }
  const label = labels[status]

  return (
    // aria-live="assertive" for error/disconnected — needs immediate announcement
    <span
      role="status"
      aria-live={status === 'error' || status === 'disconnected' ? 'assertive' : 'polite'}
      aria-label={label}
    >
      {label}
    </span>
  )
}
