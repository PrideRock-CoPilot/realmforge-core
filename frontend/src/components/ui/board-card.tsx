import { useId } from 'react'
import { BoardStateChip, type BoardState } from './board-state-chip'

interface PrimaryAction {
  label: string
  onClick: () => void
  disabled?: boolean
  disabledReason?: string
}

interface BoardCardProps {
  boardId: string
  title: string
  state: BoardState
  lastUpdated: Date
  primaryAction?: PrimaryAction
  children?: React.ReactNode
}

export function BoardCard({
  boardId,
  title,
  state,
  lastUpdated,
  primaryAction,
  children,
}: BoardCardProps) {
  const titleId = useId()
  const disabledReasonId = useId()

  const formattedDate = new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(lastUpdated)

  return (
    // Root is <article> — each board card is a self-contained piece of content.
    // aria-labelledby points to the title heading inside.
    <article aria-labelledby={titleId} className="board-card" data-board-id={boardId}>
      <header className="board-card__header">
        <h2 id={titleId} className="board-card__title">
          {title}
        </h2>
        <BoardStateChip state={state} size="sm" />
      </header>

      <div className="board-card__body">{children}</div>

      <footer className="board-card__footer">
        <time dateTime={lastUpdated.toISOString()} className="board-card__updated">
          Updated {formattedDate}
        </time>

        {primaryAction && (
          <button
            type="button"
            onClick={primaryAction.disabled ? undefined : primaryAction.onClick}
            // Both aria-disabled and disabled attribute — visual + AT contract
            disabled={primaryAction.disabled}
            aria-disabled={primaryAction.disabled}
            aria-describedby={
              primaryAction.disabled && primaryAction.disabledReason
                ? disabledReasonId
                : undefined
            }
            className="board-card__action"
          >
            {primaryAction.label}
          </button>
        )}

        {primaryAction?.disabled && primaryAction.disabledReason && (
          // Off-screen text for screen readers — visible on focus via CSS .sr-only
          <span id={disabledReasonId} className="sr-only">
            {primaryAction.disabledReason}
          </span>
        )}
      </footer>
    </article>
  )
}
