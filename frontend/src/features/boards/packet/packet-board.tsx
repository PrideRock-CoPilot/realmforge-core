import { BoardCard } from '@/components/ui/board-card'
import type { BoardState } from '@/components/ui/board-state-chip'
import { useBoardPlans, mapStatusToBoardState } from '@/api/hooks'
import { Package } from 'lucide-react'

// ─────────────────────────────────────────────
// Packet Board VIEW-BOARDS-PACKET
// Work packets derived from approved board plans
// ─────────────────────────────────────────────

interface PacketItem {
  id: string
  title: string
  state: BoardState
  lastUpdated: Date
  assignee: string
  workPathId: string
  fileCount: number
}

const ASSIGNEES = ['dmitri@realmforge', 'marcus@realmforge', 'fatima@realmforge', 'meg@realmforge']

export function PacketBoard() {
  const { data, isLoading, isError } = useBoardPlans('approved')

  const items: PacketItem[] = (data?.plans ?? []).map((plan, index) => ({
    id: `pkt-${plan.id}`,
    title: plan.title,
    state: mapStatusToBoardState(plan.status) === 'approved' ? 'execution' : 'blocked' as BoardState,
    lastUpdated: new Date(plan.updated_at),
    assignee: ASSIGNEES[index % ASSIGNEES.length],
    workPathId: plan.work_path_refs[0] ?? `wp-${plan.id}`,
    fileCount: Math.floor(Math.random() * 5) + 1,
  }))

  if (isLoading) {
    return (
      <div>
        <div className="page-header">
          <h1 className="page-header__title">Packet Board</h1>
          <p className="page-header__description">Loading approved plans for packet generation…</p>
        </div>
        <div aria-busy="true">
          <p style={{ color: 'var(--color-text-muted)' }}>Loading…</p>
        </div>
      </div>
    )
  }

  if (isError) {
    return (
      <div>
        <div className="page-header">
          <h1 className="page-header__title">Packet Board</h1>
          <p className="page-header__description" style={{ color: 'var(--color-error)' }}>
            Failed to load board plans
          </p>
        </div>
        <p style={{ color: 'var(--color-text-muted)' }}>
          No approved plans found. Approve a board plan to generate work packets.
        </p>
      </div>
    )
  }

  return (
    <div>
      <div className="page-header">
        <h1 className="page-header__title">Packet Board</h1>
        <p className="page-header__description">
          Work packets derived from approved board plans — assignees, skill grants, and evidence
        </p>
      </div>

      <div className="section-group">
        <h2 className="section-group__heading">
          <Package size={14} aria-hidden="true" style={{ display: 'inline', marginRight: '0.25rem' }} />
          Active Packets ({items.length})
        </h2>
        <div className="board-grid">
          {items.length === 0 && (
            <p style={{ color: 'var(--color-text-muted)', gridColumn: '1 / -1' }}>
              No approved plans yet. Approve a board plan to auto-generate packets.
            </p>
          )}
          {items.map((pkt) => (
            <BoardCard
              key={pkt.id}
              boardId={pkt.id}
              title={pkt.title}
              state={pkt.state}
              lastUpdated={pkt.lastUpdated}
              primaryAction={
                pkt.state === 'execution'
                  ? { label: 'Approve Packet', onClick: () => console.log('Approve', pkt.id) }
                  : undefined
              }
            >
              <dl style={{ margin: 0, fontSize: '0.8125rem', display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0.25rem 0.5rem' }}>
                <dt style={{ color: 'var(--color-text-muted)' }}>Assignee</dt>
                <dd style={{ margin: 0 }}>{pkt.assignee}</dd>
                <dt style={{ color: 'var(--color-text-muted)' }}>Work Path</dt>
                <dd style={{ margin: 0 }}>{pkt.workPathId}</dd>
                <dt style={{ color: 'var(--color-text-muted)' }}>Files</dt>
                <dd style={{ margin: 0 }}>{pkt.fileCount}</dd>
              </dl>
            </BoardCard>
          ))}
        </div>
      </div>
    </div>
  )
}
