import { useAuditEvents } from '@/api/hooks'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { BoardStateChip, type BoardState } from '@/components/ui/board-state-chip'
import { RealTimeFeed } from '@/components/ui/real-time-feed'
import { formatDate, relativeTime } from '@/lib/formatters'
import { ShieldCheck } from 'lucide-react'

// ─────────────────────────────────────────────
// Evidence Board VIEW-BOARDS-EVIDENCE
// Audit events from /v1/audit/events + live SSE stream
// ─────────────────────────────────────────────

interface EvidenceRow {
  id: string
  title: string
  state: BoardState
  type: 'test' | 'diff' | 'trace' | 'audit'
  source: string
  timestamp: Date
}

function toEvidenceRow(event: {
  id: string
  event_type: string
  actor_id: string
  entity_type: string
  timestamp: string
}): EvidenceRow {
  const type = event.entity_type === 'test' ? 'test'
    : event.entity_type === 'diff' ? 'diff'
    : event.entity_type === 'trace' ? 'trace'
    : 'audit'

  return {
    id: event.id,
    title: `${event.event_type} — ${event.actor_id}`,
    state: 'verification',
    type,
    source: event.entity_type,
    timestamp: new Date(event.timestamp),
  }
}

const columns: ColumnDef<EvidenceRow>[] = [
  { key: 'title', header: 'Evidence', cell: (row) => row.title },
  {
    key: 'type',
    header: 'Type',
    cell: (row) => (
      <span style={{ textTransform: 'capitalize', color: 'var(--color-text-secondary)' }}>
        {row.type}
      </span>
    ),
  },
  {
    key: 'state',
    header: 'Status',
    cell: (row) => <BoardStateChip state={row.state} size="sm" />,
  },
  { key: 'source', header: 'Source', cell: (row) => row.source },
  {
    key: 'timestamp',
    header: 'When',
    cell: (row) => <span title={formatDate(row.timestamp)}>{relativeTime(row.timestamp)}</span>,
  },
]

export function EvidenceBoard() {
  const { data, isLoading, isError, error } = useAuditEvents({ limit: 50 })

  const items: EvidenceRow[] = data?.events?.map(toEvidenceRow) ?? []

  return (
    <div>
      <div className="page-header">
        <h1 className="page-header__title">Evidence Board</h1>
        <p className="page-header__description">
          Audit events — tests, diffs, traces, and system mutations
        </p>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
        <div className="section-group">
          <h2 className="section-group__heading">
            <ShieldCheck size={14} aria-hidden="true" style={{ display: 'inline', marginRight: '0.25rem' }} />
            Evidence Log
            {isLoading && <span style={{ marginLeft: '0.5rem', fontSize: '0.75rem', color: 'var(--color-text-muted)' }}>loading…</span>}
          </h2>

          {isError && (
            <div role="alert" style={{ color: 'var(--color-error)', padding: '0.5rem', marginBottom: '0.5rem' }}>
              {error instanceof Error ? error.message : 'Failed to load evidence.'}
            </div>
          )}

          <DataTable
            columns={columns}
            data={items}
            totalCount={data?.total ?? items.length}
            pagination={{ pageIndex: 0, pageSize: 10, onPageChange: () => {} }}
            caption="Evidence records from audit log"
            isLoading={isLoading}
          />
        </div>

        <div>
          <RealTimeFeed
            endpoint="/v1/audit/stream"
            renderItem={(event: unknown, _index: number) => (
              <div style={{ padding: '0.5rem', borderBottom: '1px solid var(--color-border-default)', fontSize: '0.8125rem' }}>
                {String(event)}
              </div>
            )}
            maxItems={50}
            aria-label="Live audit event feed"
          />
        </div>
      </div>
    </div>
  )
}
