import type { BoardState } from '@/components/ui/board-state-chip'

// ─────────────────────────────────────────────
// Board state machine — client-side VIEW ONLY
// ─────────────────────────────────────────────
// Real state machine lives in authority-domain on the Rust side.
// This maps allowed transitions for UI presentation (enabling/disabling buttons).
// The UI NEVER computes state transitions — it only reflects backend state.

export type BoardAction =
  | 'approve_intake'
  | 'request_map'
  | 'approve_path'
  | 'request_packet'
  | 'approve_packet'
  | 'reject_packet'
  | 'accept_evidence'
  | 'request_fix'
  | 'approve_release'
  | 'preview_rollback'
  | 'flag_variance'
  | 'request_estimate'

interface StateTransition {
  from: BoardState
  action: BoardAction
  to: BoardState
}

// UI-only transition map for button enablement
const TRANSITIONS: StateTransition[] = [
  { from: 'intake',        action: 'approve_intake',  to: 'mapping' },
  { from: 'mapping',       action: 'request_map',     to: 'review' },
  { from: 'review',        action: 'approve_path',    to: 'approved' },
  { from: 'approved',      action: 'request_packet',  to: 'execution' },
  { from: 'execution',     action: 'approve_packet',  to: 'verification' },
  { from: 'execution',     action: 'reject_packet',   to: 'review' },
  { from: 'verification',  action: 'accept_evidence', to: 'release_ready' },
  { from: 'verification',  action: 'request_fix',     to: 'execution' },
  { from: 'release_ready', action: 'approve_release', to: 'released' },
  { from: 'blocked',       action: 'request_fix',     to: 'execution' },
]

export function getAllowedActions(state: BoardState): BoardAction[] {
  return TRANSITIONS
    .filter((t) => t.from === state)
    .map((t) => t.action)
}

export function getNextState(state: BoardState, action: BoardAction): BoardState | null {
  const transition = TRANSITIONS.find((t) => t.from === state && t.action === action)
  return transition?.to ?? null
}

export function isTerminalState(state: BoardState): boolean {
  return state === 'released' || state === 'archived'
}

export function getActionLabel(action: BoardAction): string {
  const labels: Record<BoardAction, string> = {
    approve_intake:   'Approve Intake',
    request_map:      'Request Map',
    approve_path:     'Approve Path',
    request_packet:   'Request Packet',
    approve_packet:   'Approve Packet',
    reject_packet:    'Reject Packet',
    accept_evidence:  'Accept Evidence',
    request_fix:      'Request Fix',
    approve_release:  'Approve Release',
    preview_rollback: 'Preview Rollback',
    flag_variance:    'Flag Variance',
    request_estimate: 'Request Estimate',
  }
  return labels[action]
}

export function getStateLabel(state: BoardState): string {
  const labels: Record<BoardState, string> = {
    intake:        'Intake',
    mapping:       'Mapping',
    review:        'Review',
    approved:      'Approved',
    execution:     'Execution',
    verification:  'Verification',
    release_ready: 'Release Ready',
    released:      'Released',
    blocked:       'Blocked',
    archived:      'Archived',
  }
  return labels[state]
}
