import { memo, type CSSProperties } from 'react'
import { BaseEdge, EdgeLabelRenderer, getStraightPath, type EdgeProps } from '@xyflow/react'

interface VisualEdgeData {
  label?: string
  type?: 'dependency' | 'handoff' | 'data-flow'
  [key: string]: unknown
}

const EDGE_STYLE: Record<string, CSSProperties> = {
  dependency: { stroke: '#6366f1', strokeDasharray: '5 3' },
  handoff:    { stroke: '#10b981' },
  'data-flow':{ stroke: '#f59e0b', strokeDasharray: '2 2' },
}

export const VisualEdge = memo(({
  id, sourceX, sourceY, targetX, targetY, data, label,
}: EdgeProps<VisualEdgeData>) => {
  const [edgePath, labelX, labelY] = getStraightPath({ sourceX, sourceY, targetX, targetY })
  const edgeType = data?.type ?? 'handoff'
  const displayLabel = data?.label ?? label

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        style={EDGE_STYLE[edgeType]}
        // aria-label on the SVG path provides AT context for the edge
        aria-label={displayLabel ? `${edgeType} edge: ${displayLabel}` : `${edgeType} edge`}
      />
      {displayLabel && (
        <EdgeLabelRenderer>
          <span
            className="visual-edge__label"
            style={{ transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)` }}
            // aria-hidden — label is described by the SVG path's aria-label above
            aria-hidden="true"
          >
            {displayLabel}
          </span>
        </EdgeLabelRenderer>
      )}
    </>
  )
})

VisualEdge.displayName = 'VisualEdge'
