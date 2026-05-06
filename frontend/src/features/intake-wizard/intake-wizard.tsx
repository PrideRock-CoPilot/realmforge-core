import { useState, useCallback, useEffect } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import {
  ClipboardList,
  HelpCircle,
  CheckCircle2,
  Loader2,
  ChevronRight,
} from 'lucide-react'
import type { IntakeTree, AnswerMap, WizardStep } from './types'
import type { MappedFeatures } from './tree-engine'
import { findTreeForAppType, mapFeatures } from './tree-engine'
import { AppTypeSelector } from './app-type-selector'
import { QuestionWalkthrough } from './question-walkthrough'
import { IntakeSummary } from './intake-summary'
import { customFetch } from '@/api/client'
import { queryKeys } from '@/api/hooks'

// ─────────────────────────────────────────────
// Intake Wizard — Main Orchestrator
// 3-step flow: Select Type → Answer Questions → Summary & Submit
// ─────────────────────────────────────────────

const STEP_LABELS: Record<WizardStep, { label: string; icon: React.ReactNode }> = {
  'select-type': { label: 'Select Type', icon: <ClipboardList size={14} aria-hidden="true" /> },
  questions: { label: 'Questions', icon: <HelpCircle size={14} aria-hidden="true" /> },
  summary: { label: 'Summary', icon: <CheckCircle2 size={14} aria-hidden="true" /> },
}

interface IntakeWizardProps {
  trees: IntakeTree[]
  loading: boolean
  onComplete?: () => void
  onClose?: () => void
}

export function IntakeWizard({ trees, loading, onComplete }: IntakeWizardProps) {
  const queryClient = useQueryClient()

  // Wizard state
  const [step, setStep] = useState<WizardStep>('select-type')
  const [appTypeId, setAppTypeId] = useState<string | null>(null)
  const [answers, setAnswers] = useState<AnswerMap>({})
  const [tree, setTree] = useState<IntakeTree | null>(null)

  // Submission state
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [features, setFeatures] = useState<MappedFeatures>({ modules: [], personas: [], skills: [] })

  // When app type is selected, find the matching tree and initialize
  const handleTypeSelect = useCallback(
    (typeId: string) => {
      setAppTypeId(typeId)
      const matched = findTreeForAppType(trees, typeId)
      if (matched) {
        setTree(matched)
        setAnswers({})
        setFeatures({ modules: [], personas: [], skills: [] })
        setSubmitError(null)
        setStep('questions')
      }
    },
    [trees],
  )

  // Track answers and re-derive features
  const handleAnswer = useCallback(
    (questionId: string, value: string | boolean | number) => {
      setAnswers((prev) => {
        const next = { ...prev, [questionId]: value }
        return next
      })
    },
    [],
  )

  // Re-derive features whenever answers change
  useEffect(() => {
    if (tree && Object.keys(answers).length > 0) {
      setFeatures(mapFeatures(tree, answers))
    }
  }, [tree, answers])

  const handleComplete = useCallback(() => {
    setStep('summary')
  }, [])

  const handleGoBack = useCallback(() => {
    if (step === 'questions') {
      setStep('select-type')
      setAppTypeId(null)
      setTree(null)
      setAnswers({})
    } else if (step === 'summary') {
      setStep('questions')
    }
  }, [step])

  const handleSubmit = useCallback(
    async (planName: string, goal: string, scope: string, owner: string) => {
      if (!tree || !appTypeId) return

      setIsSubmitting(true)
      setSubmitError(null)

      try {
        await customFetch('/v1/intake/plans', {
          method: 'POST',
          body: JSON.stringify({
            name: planName,
            goal,
            scope,
            owner,
          }),
        })

        // Invalidate plan list
        queryClient.invalidateQueries({ queryKey: queryKeys.intakePlans })

        onComplete?.()
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to create plan'
        setSubmitError(msg)
      } finally {
        setIsSubmitting(false)
      }
    },
    [tree, appTypeId, queryClient, onComplete],
  )

  // ── Render ──

  return (
    <div className="intake-wizard" role="region" aria-label="New Intake Wizard">
      {/* Step indicator */}
      <div className="intake-wizard__steps" role="navigation" aria-label="Wizard steps">
        {(['select-type', 'questions', 'summary'] as WizardStep[]).map((s, idx) => {
          const isActive = step === s
          const isPast = ['select-type', 'questions', 'summary'].indexOf(step) > idx
          return (
            <div
              key={s}
              className={`intake-wizard__step${isActive ? ' intake-wizard__step--active' : ''}${isPast ? ' intake-wizard__step--past' : ''}`}
              aria-current={isActive ? 'step' : undefined}
            >
              <span className="intake-wizard__step-num" aria-hidden="true">
                {isPast ? <CheckCircle2 size={12} /> : idx + 1}
              </span>
              <span className="intake-wizard__step-label">
                {STEP_LABELS[s].label}
              </span>
              {idx < 2 && (
                <ChevronRight size={12} className="intake-wizard__step-arrow" aria-hidden="true" />
              )}
            </div>
          )
        })}
      </div>

      {/* Content area */}
      <div className="intake-wizard__content">
        {loading && (
          <div className="intake-wizard__loading" role="status">
            <Loader2 size={24} className="spin" />
            <p>Loading decision trees…</p>
          </div>
        )}

        {!loading && step === 'select-type' && (
          <div className="intake-wizard__step-content">
            <h2 className="intake-wizard__heading">What are you building?</h2>
            <p className="intake-wizard__subheading">
              Select the application type that best matches your project. This determines
              the questions we'll ask to generate your intake plan.
            </p>
            <AppTypeSelector onSelect={handleTypeSelect} selectedType={appTypeId} />
          </div>
        )}

        {!loading && step === 'questions' && tree && (
          <div className="intake-wizard__step-content">
            <QuestionWalkthrough
              tree={tree}
              answers={answers}
              onAnswer={handleAnswer}
              onComplete={handleComplete}
              onBack={handleGoBack}
            />
          </div>
        )}

        {!loading && step === 'summary' && tree && appTypeId && (
          <div className="intake-wizard__step-content">
            <IntakeSummary
              tree={tree}
              answers={answers}
              features={features}
              appTypeId={appTypeId}
              isSubmitting={isSubmitting}
              submitError={submitError}
              onSubmit={handleSubmit}
              onBack={handleGoBack}
            />
          </div>
        )}
      </div>
    </div>
  )
}
