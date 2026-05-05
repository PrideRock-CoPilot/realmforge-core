import { memo } from 'react'
import { Handle, Position, type NodeProps } from '@xyflow/react'

interface VisualNodeData {
  label: string
  state?: string
  metadata?: Record<string, unknown>
  [key: string]: unknown
}

// icon + shape are always present alongside color — never color alone
const NODE_META: Record<string, { icon: string; shape: string }> = {
  module:   { icon: '▣', shape: 'node--module' },
  phase:    { icon: '◈', shape: 'node--phase' },
  decision: { icon: '◆', shape: 'node--decision' },
  workpath: { icon: '→', shape: 'node--workpath' },
}

function BaseVisualNode({ type, data, selected }: NodeProps<VisualNodeData> & { type: string }) {
  const meta = NODE_META[type] ?? { icon: '○', shape: 'node--unknown' }

  return (
    <div
      className={`visual-node ${meta.shape}${selected ? ' visual-node--selected' : ''}`}
      // aria-label on the node div provides the text alternative for AT users
      // The full graph has role="img" + aria-describedby pointing to the text list
      aria-label={`${type} node: ${data.label}${data.state ? `, state: ${data.state}` : ''}`}
    >
      <Handle type="target" position={Position.Top} />
      <span aria-hidden="true" className="visual-node__icon">{meta.icon}</span>
      <span className="visual-node__label">{data.label}</span>
      {data.state && (
        <span className="visual-node__state" aria-hidden="true">
          {data.state}
        </span>
      )}
      <Handle type="source" position={Position.Bottom} />
    </div>
  )
}

export const VisualModuleNode   = memo((props: NodeProps<VisualNodeData>) => <BaseVisualNode {...props} type="module" />)
export const VisualPhaseNode    = memo((props: NodeProps<VisualNodeData>) => <BaseVisualNode {...props} type="phase" />)
export const VisualDecisionNode = memo((props: NodeProps<VisualNodeData>) => <BaseVisualNode {...props} type="decision" />)
export const VisualWorkPathNode = memo((props: NodeProps<VisualNodeData>) => <BaseVisualNode {...props} type="workpath" />)

VisualModuleNode.displayName   = 'VisualModuleNode'
VisualPhaseNode.displayName    = 'VisualPhaseNode'
VisualDecisionNode.displayName = 'VisualDecisionNode'
VisualWorkPathNode.displayName = 'VisualWorkPathNode'
