import { useMemo, useState } from 'react'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { formatDate } from '@/lib/formatters'
import { ScrollText, Filter } from 'lucide-react'

// ─────────────────────────────────────────────
// Audit Browser — Audit event viewer
// ─────────────────────────────────────────────

interface AuditEvent {
  id: string
  eventType: string
  actorId: string
  resourceId: string
  detail: string
  timestamp: Date
  traceId: string
}

const MOCK_EVENTS: AuditEvent[] = [
  { id: 'aud-001', eventType: 'session.created', actorId: 'user-001', resourceId: 'session-abc', detail: 'Session created for Login module', timestamp: new Date(Date.now() - 1000 * 30), traceId: 'trace-001' },
  { id: 'aud-002', eventType: 'command.board.approve', actorId: 'user-001', resourceId: 'intake-001', detail: 'Intake board item approved', timestamp: new Date(Date.now() - 1000 * 60 * 5), traceId: 'trace-002' },
  { id: 'aud-003', eventType: 'command.packet.request', actorId: 'user-001', resourceId: 'wp-001', detail: 'Work packet requested for Login handler', timestamp: new Date(Date.now() - 1000 * 60 * 15), traceId: 'trace-003' },
  { id: 'aud-004', eventType: 'snapshot.created', actorId: 'system', resourceId: 'snap-001', detail: 'Snapshot anchor created before deployment', timestamp: new Date(Date.now() - 1000 * 60 * 30), traceId: 'trace-004' },
  { id: 'aud-005', eventType: 'policy.evaluated', actorId: 'system', resourceId: 'policy-001', detail: 'Rate limit policy evaluated for login attempt', timestamp: new Date(Date.now() - 1000 * 60 * 60), traceId: 'trace-005' },
  { id: 'aud-006', eventType: 'release.approved', actorId: 'user-002', resourceId: 'rel-001', detail: 'Release v1.0.0 approved for deployment', timestamp: new Date(Date.now() - 1000 * 60 * 120), traceId: 'trace-006' },
  { id: 'aud-007', eventType: 'rollback.executed', actorId: 'system', resourceId: 'snap-abc-123', detail: 'Rollback executed to pre-login state', timestamp: new Date(Date.now() - 1000 * 60 * 240), traceId: 'trace-007' },
]

const EVENT_TYPES = Array.from(new Set(MOCK_EVENTS.map((e) => e.eventType)))

const columns: ColumnDef<AuditEvent>[] = [
  {
    key: 'eventType',
    header: 'Event Type',
    cell: (row) => (
      <code style={{ fontSize: '0.75rem', color: 'var(--color-accent-secondary)' }}>
        {row.eventType}
      </code>
    ),
    sortable: true,
  },
  { key: 'actorId', header: 'Actor', cell: (row) => row.actorId },
  { key: 'resourceId', header: 'Resource', cell: (row) => row.resourceId },
  { key: 'detail', header: 'Detail', cell: (row) => row.detail },
  {
    key: 'timestamp',
    header: 'Timestamp',
    cell: (row) => (
      <span title={formatDate(row.timestamp)} style={{ fontSize: '0.8125rem' }}>
        {formatDate(row.timestamp)}
      </span>
    ),
    sortable: true,
  },
  {
    key: 'traceId',
    header: 'Trace',
    cell: (row) => (
      <code style={{ fontSize: '0.6875rem', color: 'var(--color-text-muted)' }}>
        {row.traceId}
      </code>
    ),
  },
]

export function AuditBrowser() {
  const [filterType, setFilterType] = useState<string>('all')
  const events = useMemo(() => MOCK_EVENTS, [])

  const filteredEvents = useMemo(() => {
    if (filterType === 'all') return events
    return events.filter((e) => e.eventType === filterType)
  }, [events, filterType])

  return (
    <div>
      <div className="page-header">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
          <div>
            <h1 className="page-header__title">Audit Browser</h1>
            <p className="page-header__description">
              Complete audit trail — every governance action is recorded
            </p>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <Filter size={14} aria-hidden="true" style={{ color: 'var(--color-text-muted)' }} />
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value)}
              style={{
                padding: '0.25rem 0.5rem',
                background: 'var(--color-bg-tertiary)',
                color: 'var(--color-text-primary)',
                border: '1px solid var(--color-border-default)',
                borderRadius: 'var(--radius-md)',
                fontSize: '0.8125rem',
                fontFamily: 'inherit',
              }}
              aria-label="Filter by event type"
            >
              <option value="all">All Events</option>
              {EVENT_TYPES.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </div>
        </div>
      </div>

      <div className="section-group">
        <h2 className="section-group__heading">
          <ScrollText size={14} aria-hidden="true" style={{ display: 'inline', marginRight: '0.25rem' }} />
          Event Log ({filteredEvents.length} events)
        </h2>
        <DataTable
          columns={columns}
          data={filteredEvents}
          totalCount={filteredEvents.length}
          pagination={{ pageIndex: 0, pageSize: 10, onPageChange: () => {} }}
          caption="Audit trail event log"
        />
      </div>
    </div>
  )
}
