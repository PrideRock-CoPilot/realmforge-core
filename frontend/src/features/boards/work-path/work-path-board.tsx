import { useState } from 'react'
import { useWorkPath } from '@/api/hooks'
import { GraphCanvas } from '@/components/ui/graph-canvas'
import { BoardStateChip } from '@/components/ui/board-state-chip'
import { Route } from 'lucide-react'

// ─────────────────────────────────────────────
// Work Path Board VIEW-BOARDS-WORKPATH
// Work path graphs from backend /v1/work-paths/:id
// ─────────────────────────────────────────────

// Known work path IDs that can be fetched from the backend
const KNOWN_WORK_PATH_IDS = ['wp_01', 'wp_02']

export function WorkPathBoard() {
  const [activePathId, setActivePathId] = useState<string | null>(
    KNOWN_WORK_PATH_IDS.length > 0 ? KNOWN_WORK_PATH_IDS[0] : null,
  )

  const { data, isLoading, isError, error } = useWorkPath(activePathId)

  return (
    <div>
      <div className="page-header">
        <h1 className="page-header__title">Work Path Board</h1>
        <p className="page-header__description">
          Work path graphs from backend — nodes, edges, risks, and file links
        </p>
      </div>

      <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem', flexWrap: 'wrap' }}>
        {KNOWN_WORK_PATH_IDS.map((id) => (
          <button
            key={id}
            type="button"
            onClick={() => setActivePathId(id)}
            style={{
              padding: '0.375rem 0.75rem',
              background: activePathId === id ? 'var(--color-bg-tertiary)' : 'var(--color-bg-secondary)',
              border: '1px solid var(--color-border-default)',
              borderRadius: 'var(--radius-md)',
              color: activePathId === id ? 'var(--color-accent-secondary)' : 'var(--color-text-secondary)',
              cursor: 'pointer',
              fontSize: '0.8125rem',
            }}
          >
            <Route size={12} aria-hidden="true" style={{ marginRight: '0.25rem' }} />
            {id}
          </button>
        ))}
        {KNOWN_WORK_PATH_IDS.length === 0 && (
          <p style={{ color: 'var(--color-text-muted)', fontSize: '0.8125rem' }}>
            No work paths configured. Create one via the API.
          </p>
        )}
      </div>

      {isLoading && (
        <div style={{ height: '400px', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-text-muted)' }} aria-busy="true">
          Loading work path graph…
        </div>
      )}

      {isError && (
        <div style={{ color: 'var(--color-error)', padding: '1rem' }} role="alert">
          {error instanceof Error ? error.message : 'Failed to load work path.'}
        </div>
      )}

      {data && !isLoading && !isError && (
        <div>
          <div style={{ marginBottom: '0.5rem', fontSize: '0.875rem', color: 'var(--color-text-secondary)' }}>
            <strong>{data.name}</strong> — {data.description}
            <span style={{ marginLeft: '0.5rem' }}>
              <BoardStateChip state={data.node_count > 0 ? 'execution' : 'intake'} size="sm" />
            </span>
            <span style={{ marginLeft: '0.5rem', fontSize: '0.75rem', color: 'var(--color-text-muted)' }}>
              {data.node_count} nodes
            </span>
          </div>

          <div style={{ height: '500px' }}>
            <GraphCanvas
              nodes={[]}
              edges={[]}
              onNodeClick={(id) => console.log('Node clicked:', id)}
              onEdgeClick={(id) => console.log('Edge clicked:', id)}
              readOnly
            />
          </div>
        </div>
      )}

      {!data && !isLoading && !isError && KNOWN_WORK_PATH_IDS.length > 0 && (
        <div style={{ height: '400px', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-text-muted)' }}>
          Select a work path above to view its graph.
        </div>
      )}
    </div>
  )
}
