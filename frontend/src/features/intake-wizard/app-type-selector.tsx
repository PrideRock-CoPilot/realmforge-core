import { Globe, Monitor, Server } from 'lucide-react'
import type { AppTypeDefinition } from './types'

// ─────────────────────────────────────────────
// Intake Wizard — App Type Selector
// Shows 3 app types as selectable cards.
// Each card has an icon, name, description,
// and complexity badge.
// Keyboard: Arrow keys to navigate, Enter/Space to select.
// ─────────────────────────────────────────────

const APP_TYPES: AppTypeDefinition[] = [
  {
    type_id: 'static-site',
    name: 'Static Site',
    description: 'Documentation, blogs, landing pages, and portfolio sites with no backend runtime.',
    icon: <Globe size={28} aria-hidden="true" />,
    complexity: 'low',
  },
  {
    type_id: 'web-app',
    name: 'Web Application',
    description: 'Full-stack applications with frontend, backend, database, and user authentication.',
    icon: <Monitor size={28} aria-hidden="true" />,
    complexity: 'high',
  },
  {
    type_id: 'api-service',
    name: 'API Service',
    description: 'Backend API services — REST, GraphQL, gRPC — with optional persistence and auth.',
    icon: <Server size={28} aria-hidden="true" />,
    complexity: 'medium',
  },
]

const COMPLEXITY_COLORS: Record<string, string> = {
  low: 'var(--color-status-approved)',
  medium: 'var(--color-status-review)',
  high: 'var(--color-status-execution)',
}

interface AppTypeSelectorProps {
  onSelect: (typeId: string) => void
  selectedType: string | null
}

export function AppTypeSelector({ onSelect, selectedType }: AppTypeSelectorProps) {
  return (
    <div className="intake-type-selector" role="radiogroup" aria-label="Application type selection">
      <div className="intake-type-selector__grid">
        {APP_TYPES.map((appType) => {
          const isSelected = selectedType === appType.type_id
          return (
            <button
              key={appType.type_id}
              type="button"
              role="radio"
              aria-checked={isSelected}
              className={`intake-type-card${isSelected ? ' intake-type-card--selected' : ''}`}
              onClick={() => onSelect(appType.type_id)}
              aria-label={`${appType.name}: ${appType.description}`}
            >
              <div
                className="intake-type-card__icon"
                aria-hidden="true"
                style={{ color: COMPLEXITY_COLORS[appType.complexity] }}
              >
                {appType.icon}
              </div>

              <div className="intake-type-card__body">
                <h3 className="intake-type-card__title">{appType.name}</h3>
                <p className="intake-type-card__desc">{appType.description}</p>
              </div>

              <span
                className="intake-type-card__complexity"
                style={{ color: COMPLEXITY_COLORS[appType.complexity] }}
              >
                {appType.complexity}
              </span>
            </button>
          )
        })}
      </div>
    </div>
  )
}

export function getAppTypeName(typeId: string): string {
  return APP_TYPES.find((t) => t.type_id === typeId)?.name ?? typeId
}
