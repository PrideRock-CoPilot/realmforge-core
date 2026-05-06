import { useCallback } from 'react'
import { ChevronLeft, ChevronRight, HelpCircle, CheckCircle2 } from 'lucide-react'
import type { IntakeTree, IntakeQuestion, AnswerMap } from './types'
import { getCurrentQuestionIndex, getVisibleQuestions, getProgress } from './tree-engine'

// ─────────────────────────────────────────────
// Intake Wizard — Question Walkthrough
// Shows one question at a time with support for
// SingleChoice, Boolean, and Number types.
// Tracks visible questions dynamically as
// conditions change with each answer.
// ─────────────────────────────────────────────

interface QuestionWalkthroughProps {
  tree: IntakeTree
  answers: AnswerMap
  onAnswer: (questionId: string, value: string | boolean | number) => void
  onComplete: () => void
  onBack: () => void
}

export function QuestionWalkthrough({
  tree,
  answers,
  onAnswer,
  onComplete,
  onBack,
}: QuestionWalkthroughProps) {
  const visibleQuestions = getVisibleQuestions(tree, answers)
  const currentIdx = getCurrentQuestionIndex(tree, answers)
  const currentQuestion = visibleQuestions[currentIdx]
  const progress = getProgress(tree, answers)

  // Determine if we can "finish early" (all required questions answered)
  const allRequiredAnswered = visibleQuestions
    .filter((q) => q.required)
    .every((q) => {
      const val = answers[q.question_id]
      return val !== undefined && val !== null && val !== ''
    })
  // Can't finish unless at least the current question is answered
  const canFinish = allRequiredAnswered

  const handleAnswer = useCallback(
    (value: string | boolean | number) => {
      if (!currentQuestion) return
      onAnswer(currentQuestion.question_id, value)
    },
    [currentQuestion, onAnswer],
  )

  const handleNext = useCallback(() => {
    if (!currentQuestion) return
    const val = answers[currentQuestion.question_id]
    if (val === undefined || val === null || val === '') return

    // Check if this was the last question
    const nextIdx = currentIdx + 1
    if (nextIdx >= visibleQuestions.length) {
      onComplete()
    }
    // If there's a next question, the effect of the answer
    // may have changed visible questions — we re-derive each render.
  }, [currentQuestion, currentIdx, visibleQuestions.length, answers, onComplete])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && currentQuestion && answers[currentQuestion.question_id] !== undefined) {
        handleNext()
      }
    },
    [currentQuestion, answers, handleNext],
  )

  if (!currentQuestion) {
    return (
      <div className="intake-wizard__empty" role="status">
        <p>All questions answered. Review your summary to complete.</p>
        <button type="button" className="btn btn--primary" onClick={onComplete}>
          <CheckCircle2 size={14} aria-hidden="true" />
          View Summary
        </button>
      </div>
    )
  }

  return (
    <div className="intake-question-view" onKeyDown={handleKeyDown}>
      {/* Navigation header */}
      <div className="intake-question-view__nav">
        <button
          type="button"
          className="btn btn--ghost"
          onClick={onBack}
          aria-label="Go back to type selection"
        >
          <ChevronLeft size={14} aria-hidden="true" />
          Back
        </button>

        <span className="intake-question-view__counter" aria-live="polite">
          {currentIdx + 1} of {visibleQuestions.length}
        </span>

        {canFinish && (
          <button
            type="button"
            className="btn btn--primary btn--sm"
            onClick={onComplete}
            aria-label="Finish answering and view summary"
          >
            <CheckCircle2 size={12} aria-hidden="true" />
            Finish
          </button>
        )}
      </div>

      {/* Progress bar */}
      <div
        className="intake-question-view__progress"
        role="progressbar"
        aria-valuenow={progress.percent}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={`Question progress: ${progress.answered} of ${progress.total} answered`}
      >
        <div
          className="intake-question-view__progress-fill"
          style={{ width: `${progress.percent}%` }}
        />
      </div>

      {/* Current question */}
      <div key={currentQuestion.question_id} className="intake-question-card">
        <p className="intake-question-card__text">{currentQuestion.text}</p>
        {!currentQuestion.required && (
          <span className="intake-question-card__optional">Optional</span>
        )}

        <div className="intake-question-card__input">
          <QuestionInput
            question={currentQuestion}
            currentValue={answers[currentQuestion.question_id] ?? null}
            onAnswer={handleAnswer}
          />
        </div>

        <div className="intake-question-card__actions">
          <button
            type="button"
            className="btn btn--primary"
            onClick={handleNext}
            disabled={
              currentQuestion.required &&
              (answers[currentQuestion.question_id] === undefined ||
                answers[currentQuestion.question_id] === null ||
                answers[currentQuestion.question_id] === '')
            }
          >
            {currentIdx + 1 >= visibleQuestions.length ? 'View Summary' : 'Next'}
            <ChevronRight size={14} aria-hidden="true" />
          </button>
        </div>
      </div>
    </div>
  )
}

// ── Question Input Sub-component ──

interface QuestionInputProps {
  question: IntakeQuestion
  currentValue: string | boolean | number | null
  onAnswer: (value: string | boolean | number) => void
}

function QuestionInput({ question, currentValue, onAnswer }: QuestionInputProps) {
  const typeName = question.question_type.type
  const radioName = `q-${question.question_id}`
  const inputId = `input-${question.question_id}`

  switch (typeName) {
    case 'SingleChoice':
      return (
        <div className="intake-options" role="radiogroup" aria-label={question.text}>
          {question.options?.map((opt) => {
            const optId = `${radioName}-${opt.value}`
            const isSelected = currentValue === opt.value
            return (
              <button
                key={opt.value}
                type="button"
                role="radio"
                id={optId}
                aria-checked={isSelected}
                className={`intake-option-btn${isSelected ? ' intake-option-btn--selected' : ''}`}
                onClick={() => onAnswer(opt.value)}
              >
                <span className="intake-option-btn__indicator" aria-hidden="true">
                  {isSelected ? '◉' : '○'}
                </span>
                <span className="intake-option-btn__label">{opt.label}</span>
              </button>
            )
          })}
        </div>
      )

    case 'Boolean':
      return (
        <div className="intake-boolean" role="radiogroup" aria-label={question.text}>
          <button
            type="button"
            role="radio"
            aria-checked={currentValue === true}
            className={`intake-boolean-btn${currentValue === true ? ' intake-boolean-btn--yes' : ''}`}
            onClick={() => onAnswer(true)}
          >
            Yes
          </button>
          <button
            type="button"
            role="radio"
            aria-checked={currentValue === false}
            className={`intake-boolean-btn${currentValue === false ? ' intake-boolean-btn--no' : ''}`}
            onClick={() => onAnswer(false)}
          >
            No
          </button>
        </div>
      )

    case 'Number': {
      const params = question.question_type.params
      return (
        <div>
          <label htmlFor={inputId} className="sr-only">
            {question.text}
          </label>
          <input
            id={inputId}
            type="number"
            className="input"
            min={params?.min}
            max={params?.max}
            value={currentValue !== null ? String(currentValue) : ''}
            onChange={(e) => {
              const v = e.target.value
              if (v === '') {
                onAnswer('' as unknown as number)
                return
              }
              const num = Number(v)
              if (!isNaN(num)) onAnswer(num)
            }}
            placeholder="Enter a number..."
            aria-describedby={question.required ? undefined : `${inputId}-optional`}
            autoFocus
          />
          {!question.required && (
            <span id={`${inputId}-optional`} className="intake-field-hint">
              Optional — leave blank to skip
            </span>
          )}
        </div>
      )
    }

    default:

      return (
        <p className="intake-field-hint" role="alert">
          <HelpCircle size={12} aria-hidden="true" />
          Unsupported question type: {typeName}
        </p>
      )
  }
}
