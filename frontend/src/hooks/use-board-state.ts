import { useMemo } from 'react'
import type { BoardState } from '@/components/ui/board-state-chip'
import { getAllowedActions, getActionLabel, isTerminalState } from '@/lib/board-state'

// ─────────────────────────────────────────────
// Board state hook — UI helpers for board state
// ─────────────────────────────────────────────

interface UseBoardStateResult {
  allowedActions: Array<{
    action: string
    label: string
    handler: () => void
  }>
  isTerminal: boolean
  isBlocked: boolean
}

export function useBoardState(
  state: BoardState,
  onAction?: (action: string) => void,
): UseBoardStateResult {
  const actions = useMemo(() => {
    return getAllowedActions(state).map((action) => ({
      action,
      label: getActionLabel(action),
      handler: () => onAction?.(action),
    }))
  }, [state, onAction])

  const isTerminal = useMemo(() => isTerminalState(state), [state])
  const isBlocked = state === 'blocked'

  return {
    allowedActions: actions,
    isTerminal,
    isBlocked,
  }
}
