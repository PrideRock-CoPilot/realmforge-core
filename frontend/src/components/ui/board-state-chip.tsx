// ADR-0002: shadcn/ui is an internal implementation detail.
// Consumers import <BoardStateChip> — never the shadcn primitive directly.

export type BoardState =
  | 'intake'
  | 'mapping'
  | 'review'
  | 'approved'
  | 'execution'
  | 'verification'
  | 'release_ready'
  | 'released'
  | 'blocked'
  | 'archived'

interface BoardStateChipProps {
  state: BoardState
  size?: 'sm' | 'md'
}

// Each state has icon + text label — color is never the only differentiator.
const STATE_META: Record<BoardState, { label: string; icon: string; cssClass: string }> = {
  intake:        { label: 'Intake',          icon: '○', cssClass: 'chip--intake' },
  mapping:       { label: 'Mapping',         icon: '⬡', cssClass: 'chip--mapping' },
  review:        { label: 'Review',          icon: '◈', cssClass: 'chip--review' },
  approved:      { label: 'Approved',        icon: '✓', cssClass: 'chip--approved' },
  execution:     { label: 'Execution',       icon: '▶', cssClass: 'chip--execution' },
  verification:  { label: 'Verification',    icon: '⊛', cssClass: 'chip--verification' },
  release_ready: { label: 'Release Ready',   icon: '◎', cssClass: 'chip--release-ready' },
  released:      { label: 'Released',        icon: '★', cssClass: 'chip--released' },
  blocked:       { label: 'Blocked',         icon: '⊘', cssClass: 'chip--blocked' },
  archived:      { label: 'Archived',        icon: '◻', cssClass: 'chip--archived' },
}

export function BoardStateChip({ state, size = 'md' }: BoardStateChipProps) {
  const { label, icon, cssClass } = STATE_META[state]

  return (
    <span
      role="status"
      className={`board-state-chip board-state-chip--${size} ${cssClass}`}
      // blocked state is announced immediately in the nearest live region;
      // callers owning a live region should set aria-live="polite" on it
      aria-label={`Board state: ${label}`}
    >
      {/* icon is aria-hidden — label carries the full meaning */}
      <span aria-hidden="true" className="chip__icon">
        {icon}
      </span>
      <span className="chip__label">{label}</span>
    </span>
  )
}
