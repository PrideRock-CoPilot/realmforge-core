import { Fragment, useCallback, useId, useRef, type KeyboardEvent, type MouseEvent } from 'react'
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  type NodeTypes,
  type EdgeTypes,
  type Node,
  type Edge,
  useReactFlow,
  ReactFlowProvider,
} from '@xyflow/react'
import { useGraphStore } from '@/store/graph.store'
import {
  VisualModuleNode,
  VisualPhaseNode,
  VisualDecisionNode,
  VisualWorkPathNode,
} from './visual-node'
import { VisualEdge as VisualEdgeComponent } from './visual-edge'

export interface VisualNode {
  id: string
  type: 'module' | 'phase' | 'decision' | 'workpath'
  data: {
    label: string
    state?: string
    metadata?: Record<string, unknown>
  }
  position: { x: number; y: number }
}

export interface VisualEdgeDef {
  id: string
  source: string
  target: string
  label?: string
  type?: 'dependency' | 'handoff' | 'data-flow'
}

interface GraphCanvasProps {
  nodes: VisualNode[]
  edges: VisualEdgeDef[]
  onNodeClick?: (nodeId: string) => void
  onEdgeClick?: (edgeId: string) => void
  readOnly?: boolean
}

// @xyflow/react v12 has stricter generic constraints on NodeTypes/EdgeTypes.
// Our custom node/edge components accept any props; we cast to satisfy the type checker.
const nodeTypes = {
  module:   VisualModuleNode,
  phase:    VisualPhaseNode,
  decision: VisualDecisionNode,
  workpath: VisualWorkPathNode,
} as NodeTypes

const edgeTypes = {
  dependency:  VisualEdgeComponent,
  handoff:     VisualEdgeComponent,
  'data-flow': VisualEdgeComponent,
} as EdgeTypes

function GraphCanvasInner({
  nodes,
  edges,
  onNodeClick,
  onEdgeClick,
  readOnly = false,
}: GraphCanvasProps) {
  const textAltId = useId()
  const { selectNode, setZoom } = useGraphStore()
  const { getZoom } = useReactFlow()

  // Detail panel ref — focus moves here when a node is activated (spec requirement)
  const detailPanelRef = useRef<HTMLDivElement>(null)

  const handleNodeClick = useCallback(
    (_: MouseEvent, node: Node) => {
      selectNode(node.id)
      onNodeClick?.(node.id)
      // Focus moves to detail panel when node is activated
      // requestAnimationFrame ensures the panel has rendered before we focus it
      requestAnimationFrame(() => detailPanelRef.current?.focus())
    },
    [onNodeClick, selectNode],
  )

  const handleEdgeClick = useCallback(
    (_: MouseEvent, edge: Edge) => {
      onEdgeClick?.(edge.id)
    },
    [onEdgeClick],
  )

  const handleMoveEnd = useCallback(() => {
    setZoom(getZoom())
  }, [getZoom, setZoom])

  // Keyboard: Escape from detail panel returns focus to the graph
  const graphRef = useRef<HTMLDivElement>(null)
  const handleDetailKeyDown = useCallback((e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === 'Escape') {
      selectNode(null)
      graphRef.current?.focus()
    }
  }, [selectNode])

  const selectedNodeId = useGraphStore((s) => s.selectedNodeId)
  const selectedNode = nodes.find((n) => n.id === selectedNodeId)

  // Mapped to React Flow node/edge format
  const rfNodes: Node[] = nodes.map((n) => ({
    id: n.id,
    type: n.type,
    data: n.data,
    position: n.position,
    selected: n.id === selectedNodeId,
  }))

  const rfEdges: Edge[] = edges.map((e) => ({
    id: e.id,
    source: e.source,
    target: e.target,
    label: e.label,
    type: e.type ?? 'handoff',
    data: { type: e.type, label: e.label },
  }))

  return (
    <div className="graph-canvas-wrapper">
      {/* role="img" on the canvas region; aria-describedby points to the text alternative */}
      <div
        ref={graphRef}
        role="img"
        aria-label={`Visual graph with ${nodes.length} nodes and ${edges.length} edges`}
        aria-describedby={textAltId}
        className="graph-canvas"
        tabIndex={0}
      >
        <ReactFlow
          nodes={rfNodes}
          edges={rfEdges}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          onNodeClick={handleNodeClick}
          onEdgeClick={handleEdgeClick}
          onMoveEnd={handleMoveEnd}
          nodesDraggable={!readOnly}
          nodesConnectable={false}
          elementsSelectable={!readOnly}
          // Do NOT set panOnScroll=false — virtualization requires default behavior
          fitView
        >
          <Background />
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>

      {/* Text alternative for screen reader users — off-screen <details> */}
      <details id={textAltId} className="sr-only">
        <summary>Graph contents — text list</summary>
        <section>
          <h3>Nodes ({nodes.length})</h3>
          <ul>
            {nodes.map((n) => (
              <li key={n.id}>
                <strong>{n.type}:</strong> {n.data.label}
                {n.data.state ? ` — state: ${n.data.state}` : ''}
              </li>
            ))}
          </ul>
          <h3>Connections ({edges.length})</h3>
          <ul>
            {edges.map((e) => {
              const src = nodes.find((n) => n.id === e.source)?.data.label ?? e.source
              const tgt = nodes.find((n) => n.id === e.target)?.data.label ?? e.target
              return (
                <li key={e.id}>
                  {src} → {tgt}
                  {e.label ? ` (${e.label})` : ''}
                  {e.type ? ` [${e.type}]` : ''}
                </li>
              )
            })}
          </ul>
        </section>
      </details>

      {/* Detail panel — focus lands here on node activation; Escape returns focus to graph */}
      {selectedNode && (
        <div
          ref={detailPanelRef}
          role="region"
          aria-label={`Details for ${selectedNode.data.label}`}
          className="graph-canvas__detail-panel"
          tabIndex={-1}
          onKeyDown={handleDetailKeyDown}
        >
          <h2 className="graph-canvas__detail-title">{selectedNode.data.label}</h2>
          <dl className="graph-canvas__detail-meta">
            <dt>Type</dt>
            <dd>{selectedNode.type}</dd>
            {selectedNode.data.state && (
              <>
                <dt>State</dt>
                <dd>{selectedNode.data.state}</dd>
              </>
            )}
            {selectedNode.data.metadata &&
              Object.entries(selectedNode.data.metadata).map(([k, v]) => (
                <Fragment key={k}>
                  <dt>{k}</dt>
                  <dd>{String(v)}</dd>
                </Fragment>
              ))}
          </dl>
          <button
            type="button"
            className="graph-canvas__detail-close"
            onClick={() => {
              selectNode(null)
              graphRef.current?.focus()
            }}
          >
            Close (Esc)
          </button>
        </div>
      )}
    </div>
  )
}

// ReactFlowProvider must wrap the component that uses useReactFlow()
export function GraphCanvas(props: GraphCanvasProps) {
  return (
    <ReactFlowProvider>
      <GraphCanvasInner {...props} />
    </ReactFlowProvider>
  )
}
