import { useMemo, useState } from 'react'
import { GraphCanvas, type VisualNode, type VisualEdgeDef } from '@/components/ui/graph-canvas'
import { Maximize2, Minimize2, Search } from 'lucide-react'

// ─────────────────────────────────────────────
// Visual Map — Full-screen product graph
// Modules as VN-* nodes, relationships as VE-* edges
// ─────────────────────────────────────────────

const MOCK_NODES: VisualNode[] = [
  { id: 'vn-module-login', type: 'module', data: { label: 'Login Module', state: 'active' }, position: { x: 400, y: 0 } },
  { id: 'vn-module-authority', type: 'module', data: { label: 'Authority Core', state: 'active' }, position: { x: 0, y: 200 } },
  { id: 'vn-module-boards', type: 'module', data: { label: 'Boards', state: 'active' }, position: { x: 800, y: 200 } },
  { id: 'vn-module-catalog', type: 'module', data: { label: 'Catalogs', state: 'active' }, position: { x: 200, y: 400 } },
  { id: 'vn-module-grants', type: 'module', data: { label: 'Skill Grants', state: 'active' }, position: { x: 400, y: 400 } },
  { id: 'vn-module-gateway', type: 'module', data: { label: 'Agent Gateway', state: 'active' }, position: { x: 600, y: 400 } },
  { id: 'vn-module-knowledge', type: 'module', data: { label: 'Knowledge', state: 'active' }, position: { x: 400, y: 600 } },
  { id: 'vn-module-runtime', type: 'module', data: { label: 'Runtime Bundle', state: 'active' }, position: { x: 200, y: 800 } },
  { id: 'vn-module-live-watch', type: 'module', data: { label: 'Live Watch', state: 'active' }, position: { x: 600, y: 800 } },
  // Decision nodes
  { id: 'vn-dec-001', type: 'decision', data: { label: 'DEC-COUNCIL-001', state: 'closed' }, position: { x: -200, y: 300 } },
  { id: 'vn-dec-002', type: 'decision', data: { label: 'DEC-COUNCIL-002', state: 'closed' }, position: { x: -200, y: 500 } },
  // Work path nodes
  { id: 'vn-wp-login', type: 'workpath', data: { label: 'WP-LOGIN-001', state: 'in-progress' }, position: { x: 1000, y: 0 } },
]

const MOCK_EDGES: VisualEdgeDef[] = [
  // Authority Core edges
  { id: 've-auth-login', source: 'vn-module-authority', target: 'vn-module-login', type: 'dependency', label: 'authenticates' },
  { id: 've-auth-boards', source: 'vn-module-authority', target: 'vn-module-boards', type: 'dependency', label: 'governs' },
  { id: 've-auth-catalog', source: 'vn-module-authority', target: 'vn-module-catalog', type: 'dependency', label: 'governs' },
  { id: 've-auth-grants', source: 'vn-module-authority', target: 'vn-module-grants', type: 'dependency', label: 'enforces' },
  // Catalog edges
  { id: 've-cat-login', source: 'vn-module-catalog', target: 'vn-module-login', type: 'data-flow', label: 'defines' },
  { id: 've-cat-knowledge', source: 'vn-module-catalog', target: 'vn-module-knowledge', type: 'data-flow', label: 'defines' },
  // Gateway edges
  { id: 've-gate-grants', source: 'vn-module-grants', target: 'vn-module-gateway', type: 'dependency', label: 'authorizes' },
  { id: 've-gate-knowledge', source: 'vn-module-gateway', target: 'vn-module-knowledge', type: 'handoff', label: 'dispatches' },
  // Runtime edges
  { id: 've-rt-knowledge', source: 'vn-module-knowledge', target: 'vn-module-runtime', type: 'handoff', label: 'packages' },
  { id: 've-rt-livewatch', source: 'vn-module-runtime', target: 'vn-module-live-watch', type: 'handoff', label: 'deploys' },
  // Decision edges
  { id: 've-dec-001', source: 'vn-dec-001', target: 'vn-module-boards', type: 'dependency', label: 'resolves' },
  { id: 've-dec-002', source: 'vn-dec-002', target: 'vn-module-authority', type: 'dependency', label: 'resolves' },
  // Work path
  { id: 've-wp-login', source: 'vn-wp-login', target: 'vn-module-login', type: 'handoff', label: 'traverses' },
]

export function VisualMapPage() {
  const [fullScreen, setFullScreen] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')

  const nodes = useMemo(() => MOCK_NODES, [])
  const edges = useMemo(() => MOCK_EDGES, [])

  const filteredNodes = useMemo(() => {
    if (!searchQuery.trim()) return nodes
    const q = searchQuery.toLowerCase()
    return nodes.filter(
      (n) =>
        n.data.label.toLowerCase().includes(q) ||
        n.id.toLowerCase().includes(q) ||
        n.type.toLowerCase().includes(q),
    )
  }, [nodes, searchQuery])

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div className="page-header" style={{ marginBottom: '0.75rem' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
          <div>
            <h1 className="page-header__title">Visual Map</h1>
            <p className="page-header__description">
              Product graph — modules, decisions, work paths, and relationships
            </p>
          </div>
          <button
            type="button"
            onClick={() => setFullScreen(!fullScreen)}
            style={{
              padding: '0.375rem 0.75rem',
              background: 'var(--color-bg-tertiary)',
              color: 'var(--color-text-primary)',
              border: '1px solid var(--color-border-default)',
              borderRadius: 'var(--radius-md)',
              cursor: 'pointer',
              fontSize: '0.8125rem',
              display: 'flex',
              alignItems: 'center',
              gap: '0.375rem',
            }}
            aria-label={fullScreen ? 'Exit full screen' : 'Enter full screen'}
          >
            {fullScreen ? <Minimize2 size={14} aria-hidden="true" /> : <Maximize2 size={14} aria-hidden="true" />}
            {fullScreen ? 'Exit Full Screen' : 'Full Screen'}
          </button>
        </div>
      </div>

      {/* Search */}
      <div style={{ position: 'relative', marginBottom: '0.75rem' }}>
        <Search
          size={14}
          aria-hidden="true"
          style={{
            position: 'absolute',
            left: '0.75rem',
            top: '50%',
            transform: 'translateY(-50%)',
            color: 'var(--color-text-muted)',
          }}
        />
        <input
          type="text"
          placeholder="Search nodes by name, ID, or type…"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          style={{
            width: '100%',
            padding: '0.5rem 0.75rem 0.5rem 2.25rem',
            background: 'var(--color-bg-secondary)',
            border: '1px solid var(--color-border-default)',
            borderRadius: 'var(--radius-md)',
            color: 'var(--color-text-primary)',
            fontSize: '0.8125rem',
            fontFamily: 'inherit',
          }}
          aria-label="Search graph nodes"
        />
      </div>

      {/* Graph canvas */}
      <div style={{
        flex: 1,
        minHeight: fullScreen ? 'calc(100vh - 160px)' : '500px',
        transition: 'min-height 0.2s',
      }}>
        <GraphCanvas
          nodes={filteredNodes}
          edges={edges}
          onNodeClick={(id) => console.log('Node clicked:', id)}
          onEdgeClick={(id) => console.log('Edge clicked:', id)}
          readOnly
        />
      </div>

      <div style={{
        marginTop: '0.75rem',
        padding: '0.5rem 0.75rem',
        background: 'var(--color-bg-tertiary)',
        borderRadius: 'var(--radius-md)',
        fontSize: '0.75rem',
        color: 'var(--color-text-muted)',
        display: 'flex',
        gap: '1.5rem',
      }}>
        <span>Nodes: {filteredNodes.length}</span>
        <span>Edges: {edges.length}</span>
        <span>
          {filteredNodes.filter((n) => n.type === 'module').length} modules
        </span>
        <span>
          {filteredNodes.filter((n) => n.type === 'decision').length} decisions
        </span>
      </div>
    </div>
  )
}
