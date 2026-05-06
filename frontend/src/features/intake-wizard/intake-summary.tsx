import { useId } from 'react'
import {
  Loader2,
  Package,
  Users,
  Wrench,
  FileText,
  AlertCircle,
  ArrowLeft,
  Send,
} from 'lucide-react'
import type { IntakeTree, AnswerMap, IdentifiedModule, IdentifiedPersona, IdentifiedSkill } from './types'
import type { MappedFeatures } from './tree-engine'
import { getAppTypeName } from './app-type-selector'
import { generatePlanName } from './tree-engine'

// ─────────────────────────────────────────────
// Intake Wizard — Summary
// Shows a review of all answers, identified
// modules, personas, and skills before the
// user submits to create a plan.
// ─────────────────────────────────────────────

interface IntakeSummaryProps {
  tree: IntakeTree
  answers: AnswerMap
  features: MappedFeatures
  appTypeId: string
  isSubmitting: boolean
  submitError: string | null
  onSubmit: (planName: string, goal: string, scope: string, owner: string) => void
  onBack: () => void
}

export function IntakeSummary({
  tree,
  answers,
  features,
  appTypeId,
  isSubmitting,
  submitError,
  onSubmit,
  onBack,
}: IntakeSummaryProps) {
  const titleId = useId()
  const suggestedPlanName = generatePlanName(tree, answers)
  const appTypeName = getAppTypeName(appTypeId)
  const handleSubmit = () => {
    // Build goal from answers
    const answerLines = tree.questions
      .filter((q) => {
        const val = answers[q.question_id]
        return val !== undefined && val !== null && val !== ''
      })
      .map((q) => {
        const val = answers[q.question_id]
        const option = q.options?.find((o) => o.value === String(val))
        const label = option?.label ?? String(val)
        return `${q.text}: ${label}`
      })
      .join('\n')

    const goal = `${appTypeName} — ${features.modules.length} modules, ${features.skills.length} skills`
    const scope = `Application type: ${appTypeName}\n\nAnswers:\n${answerLines}`

    onSubmit(suggestedPlanName, goal, scope, 'intake-wizard')
  }

  return (
    <div className="intake-summary" aria-labelledby={titleId}>
      {/* Header */}
      <div className="intake-summary__header">
        <button
          type="button"
          className="btn btn--ghost"
          onClick={onBack}
          disabled={isSubmitting}
          aria-label="Go back to questions"
        >
          <ArrowLeft size={14} aria-hidden="true" />
          Back
        </button>
        <h2 id={titleId} className="intake-summary__title">
          <FileText size={18} aria-hidden="true" />
          Intake Summary
        </h2>
      </div>

      <div className="intake-summary__body">
        {/* Plan Name */}
        <div className="intake-summary__section">
          <h3 className="intake-summary__section-title">Plan Name</h3>
          <p className="intake-summary__plan-name">{suggestedPlanName}</p>
        </div>

        {/* Identified Modules */}
        <div className="intake-summary__section">
          <h3 className="intake-summary__section-title">
            <Package size={14} aria-hidden="true" />
            Identified Modules ({features.modules.length})
          </h3>
          <div className="intake-summary__tags">
            {features.modules.length === 0 && (
              <span className="intake-summary__empty">No modules identified</span>
            )}
            {features.modules.map((m: IdentifiedModule) => (
              <span
                key={m.module}
                className={`intake-summary__tag${m.isMeta ? ' intake-summary__tag--meta' : ''}`}
                title={m.isMeta ? 'Cross-cutting concern' : `From: ${m.source}`}
              >
                {m.module}
                {m.isMeta && <span className="intake-summary__tag-badge">meta</span>}
              </span>
            ))}
          </div>
        </div>

        {/* Personas */}
        <div className="intake-summary__section">
          <h3 className="intake-summary__section-title">
            <Users size={14} aria-hidden="true" />
            Personas Required ({features.personas.length})
          </h3>
          <div className="intake-summary__tags">
            {features.personas.length === 0 && (
              <span className="intake-summary__empty">No personas identified</span>
            )}
            {features.personas.map((p: IdentifiedPersona) => (
              <span
                key={p.persona}
                className={`intake-summary__tag${p.isMeta ? ' intake-summary__tag--meta' : ''}`}
              >
                {p.persona}
              </span>
            ))}
          </div>
        </div>

        {/* Skills */}
        <div className="intake-summary__section">
          <h3 className="intake-summary__section-title">
            <Wrench size={14} aria-hidden="true" />
            Skills Required ({features.skills.length})
          </h3>
          <div className="intake-summary__tags">
            {features.skills.length === 0 && (
              <span className="intake-summary__empty">No skills identified</span>
            )}
            {features.skills.map((s: IdentifiedSkill) => (
              <span
                key={s.skill}
                className={`intake-summary__tag${s.isMeta ? ' intake-summary__tag--meta' : ''}`}
              >
                {s.skill}
              </span>
            ))}
          </div>
        </div>

        {/* Answers Review */}
        <div className="intake-summary__section">
          <h3 className="intake-summary__section-title">Answers Review</h3>
          <dl className="intake-summary__answers">
            {tree.questions
              .filter((q) => {
                const val = answers[q.question_id]
                return val !== undefined && val !== null && val !== ''
              })
              .map((q) => {
                const val = answers[q.question_id]
                let display = String(val)
                const option = q.options?.find((o) => o.value === String(val))
                if (option) display = option.label
                return (
                  <div key={q.question_id} className="intake-summary__answer-row">
                    <dt className="intake-summary__answer-q">{q.text}</dt>
                    <dd className="intake-summary__answer-a">{display}</dd>
                  </div>
                )
              })}
          </dl>
        </div>
      </div>

      {/* Submit error */}
      {submitError && (
        <div className="banner banner--error" role="alert">
          <AlertCircle size={14} aria-hidden="true" />
          <span>{submitError}</span>
        </div>
      )}

      {/* Submit action */}
      <div className="intake-summary__actions">
        <button
          type="button"
          className="btn btn--primary"
          onClick={handleSubmit}
          disabled={isSubmitting}
        >
          {isSubmitting ? (
            <>
              <Loader2 size={14} className="spin" />
              Creating Plan…
            </>
          ) : (
            <>
              <Send size={14} aria-hidden="true" />
              Create Plan & Enter Pipeline
            </>
          )}
        </button>
      </div>
    </div>
  )
}
