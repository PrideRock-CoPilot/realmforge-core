import { useState } from 'react'
import { useBundles, queryKeys } from '@/api/hooks'
import { BoardCard } from '@/components/ui/board-card'
import type { BoardState } from '@/components/ui/board-state-chip'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { formatDate } from '@/lib/formatters'
import { Rocket } from 'lucide-react'
import { useQuery } from '@tanstack/react-query'
import { customFetch } from '@/api/client'

// ─────────────────────────────────────────────
// Release Board VIEW-BOARDS-RELEASE
// Bundles from /v1/bundles + rollback snapshots from /v1/snapshots
// ─────────────────────────────────────────────

interface ReleaseItem {
  id: string
  title: string
  state: BoardState
  lastUpdated: Date
  version: string
  approvals: number
  requiredApprovals: number
}

interface RollbackAnchor {
  id: string
  snapshotId: string
  createdAt: Date
  description: string
}

const releaseStateMap: Record<string, BoardState> = {
  draft: 'intake',
  pending: 'review',
  approved: 'release_ready',
  deployed: 'released',
  failed: 'blocked',
}

// Types for snapshot list response
interface SnapshotListItem {
  id: string
  snapshot_id: string
  reason: string
  status: string
  created_at: string
}

interface SnapshotListResponse {
  snapshots: SnapshotListItem[]
  total: number
}

export function ReleaseBoard() {
  const [showRollbackPreview, setShowRollbackPreview] = useState(false)

  const {
    data: bundlesData,
    isLoading: bundlesLoading,
    isError: bundlesError,
    error: bundlesErr,
  } = useBundles()

  // Fetch snapshots for rollback anchors
  const {
    data: snapshotsData,
    isLoading: snapshotsLoading,
  } = useQuery<SnapshotListResponse>({
    queryKey: queryKeys.snapshots,
    queryFn: () => customFetch<SnapshotListResponse>('/v1/snapshots'),
  })

  const releases: ReleaseItem[] = (bundlesData?.bundles ?? []).map((b) => ({
    id: b.id,
    title: `${b.name} v${b.version}`,
    state: releaseStateMap[b.status] ?? 'execution',
    lastUpdated: new Date(b.created_at),
    version: b.version,
    approvals: b.status === 'approved' ? 3 : 0,
    requiredApprovals: 3,
  }))

  const rollbacks: RollbackAnchor[] = (snapshotsData?.snapshots ?? []).map((s) => ({
    id: s.id,
    snapshotId: s.snapshot_id ?? s.id,
    createdAt: new Date(s.created_at),
    description: s.reason,
  }))

  const rollbackColumns: ColumnDef<RollbackAnchor>[] = [
    { key: 'id', header: 'Anchor ID', cell: (row) => row.id },
    { key: 'snapshotId', header: 'Snapshot', cell: (row) => row.snapshotId },
    { key: 'createdAt', header: 'Created', cell: (row) => formatDate(row.createdAt) },
    { key: 'description', header: 'Description', cell: (row) => row.description },
  ]

  return (
    <div>
      <div className="page-header">
        <h1 className="page-header__title">Release Board</h1>
        <p className="page-header__description">
          Bundle status, release approvals, and rollback snapshots
        </p>
      </div>

      <div className="section-group">
        <h2 className="section-group__heading">
          <Rocket size={14} aria-hidden="true" style={{ display: 'inline', marginRight: '0.25rem' }} />
          Bundles ({releases.length})
          {bundlesLoading && <span style={{ marginLeft: '0.5rem', fontSize: '0.75rem', color: 'var(--color-text-muted)' }}>loading…</span>}
        </h2>

        {bundlesError && (
          <div role="alert" style={{ color: 'var(--color-error)', padding: '0.5rem', marginBottom: '0.5rem' }}>
            {bundlesErr instanceof Error ? bundlesErr.message : 'Failed to load bundles.'}
          </div>
        )}

        <div className="board-grid">
          {releases.length === 0 && !bundlesLoading && (
            <p style={{ color: 'var(--color-text-muted)', gridColumn: '1 / -1' }}>
              No bundles found. Deploy a bundle to see it here.
            </p>
          )}
          {releases.map((rel) => (
            <BoardCard
              key={rel.id}
              boardId={rel.id}
              title={rel.title}
              state={rel.state}
              lastUpdated={rel.lastUpdated}
              primaryAction={
                rel.state === 'release_ready'
                  ? { label: 'Approve Release', onClick: () => console.log('Approve', rel.id) }
                  : undefined
              }
            >
              <dl style={{ margin: 0, fontSize: '0.8125rem', display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0.25rem 0.5rem' }}>
                <dt style={{ color: 'var(--color-text-muted)' }}>Version</dt>
                <dd style={{ margin: 0 }}>{rel.version}</dd>
                <dt style={{ color: 'var(--color-text-muted)' }}>Approvals</dt>
                <dd style={{ margin: 0 }}>{rel.approvals}/{rel.requiredApprovals}</dd>
              </dl>
            </BoardCard>
          ))}
        </div>
      </div>

      <div className="section-group">
        <h2 className="section-group__heading">
          Rollback Anchors
          {snapshotsLoading && <span style={{ marginLeft: '0.5rem', fontSize: '0.75rem', color: 'var(--color-text-muted)' }}>loading…</span>}
        </h2>
        <div>
          <button
            type="button"
            onClick={() => setShowRollbackPreview(!showRollbackPreview)}
            style={{
              marginBottom: '0.75rem',
              padding: '0.375rem 0.75rem',
              background: 'var(--color-bg-tertiary)',
              color: 'var(--color-text-primary)',
              border: '1px solid var(--color-border-default)',
              borderRadius: 'var(--radius-md)',
              cursor: 'pointer',
              fontSize: '0.8125rem',
            }}
            aria-expanded={showRollbackPreview}
          >
            {showRollbackPreview ? 'Hide Rollback Preview' : 'Preview Rollback'}
          </button>

          {showRollbackPreview && (
            <div style={{
              padding: '1rem',
              background: 'var(--color-bg-secondary)',
              border: '1px solid var(--color-border-default)',
              borderRadius: 'var(--radius-lg)',
              marginBottom: '1rem',
            }}>
              <h3 style={{ fontSize: '0.875rem', fontWeight: 600, margin: '0 0 0.5rem', color: 'var(--color-warning)' }}>
                ⚠ Rollback Preview
              </h3>
              <p style={{ fontSize: '0.8125rem', color: 'var(--color-text-secondary)', margin: 0 }}>
                Rollback will restore the system to the state captured by the selected snapshot anchor.
                This reverts all mutations made after the anchor timestamp. Audit trail remains intact.
              </p>
            </div>
          )}

          <DataTable
            columns={rollbackColumns}
            data={rollbacks}
            totalCount={rollbacks.length}
            pagination={{ pageIndex: 0, pageSize: 10, onPageChange: () => {} }}
            caption="Rollback anchors from snapshot history"
            isLoading={snapshotsLoading}
          />
        </div>
      </div>
    </div>
  )
}
