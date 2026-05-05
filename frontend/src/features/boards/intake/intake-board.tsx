import { useState } from 'react'
import {
  useIntakePlans,
  useCreateIntakePlan,
  useRefineIntakePlan,
  useSetIntakeArchitecture,
  useDecomposeIntakePlan,
  usePacketizeIntakePlan,
  useAdvanceIntakePlanToReady,
  useIntakePlan,
  useIntakePlanAudit,
} from '@/api/hooks'
import type {
  IntakePlan,
  CoreArea,
  PlanDecision,
  PlanRisk,
  PlanPhase,
  WorkPacket,
  PlanAuditEntry,
} from '@/api/hooks'
import { useSession } from '@/hooks/use-session'
import { formatDate, relativeTime } from '@/lib/formatters'
import { X, Loader2, AlertCircle, CheckCircle2, Plus, ArrowRight, ChevronRight, ChevronDown, FileText, Layers, Siren, GitBranch, PackageCheck, CircleDot, ClipboardList, History } from 'lucide-react'

// ─────────────────────────────────────────────
// Intake Pipeline Page — PIPELINE-VIEW-INTAKE
// 6-stage pipeline: intake → refinement → architecture → decomposition → packetization → ready
// ─────────────────────────────────────────────

const PIPELINE_STAGES = [
  { key: 'intake',        label: 'Intake',         icon: ClipboardList,   color: 'chip--intake' },
  { key: 'refinement',    label: 'Refinement',     icon: FileText,        color: 'chip--mapping' },
  { key: 'architecture',  label: 'Architecture',   icon: Layers,          color: 'chip--review' },
  { key: 'decomposition', label: 'Decomposition',  icon: GitBranch,       color: 'chip--approved' },
  { key: 'packetization', label: 'Packetization',  icon: PackageCheck,    color: 'chip--execution' },
  { key: 'ready',         label: 'Ready',          icon: CircleDot,       color: 'chip--release-ready' },
] as const

type PipelineStage = (typeof PIPELINE_STAGES)[number]['key']

function getStageActions(currentStage: string, isReady: boolean): { label: string; action: string; stage: PipelineStage }[] {
  if (isReady) return []
  switch (currentStage) {
    case 'intake':
      return [{ label: 'Refine Plan', action: 'refine', stage: 'refinement' as PipelineStage }]
    case 'refinement':
      return [{ label: 'Set Architecture', action: 'architecture', stage: 'architecture' as PipelineStage }]
    case 'architecture':
      return [{ label: 'Decompose', action: 'decompose', stage: 'decomposition' as PipelineStage }]
    case 'decomposition':
      return [{ label: 'Generate Packets', action: 'packetize', stage: 'packetization' as PipelineStage }]
    case 'packetization':
      return [{ label: 'Advance to Ready', action: 'ready', stage: 'ready' as PipelineStage }]
    default:
      return []
  }
}

// ── Stage Color Map for inline styling ──

const STAGE_COLORS: Record<PipelineStage, string> = {
  intake:        '#6366f1',
  refinement:    '#8b5cf6',
  architecture:  '#f59e0b',
  decomposition: '#10b981',
  packetization: '#3b82f6',
  ready:         '#14b8a6',
}

// ── Create Plan Dialog ──

interface CreatePlanDialogProps {
  onClose: () => void
  onSubmit: (name: string, goal: string, scope: string, owner: string) => void
  isSubmitting: boolean
}

function CreatePlanDialog({ onClose, onSubmit, isSubmitting }: CreatePlanDialogProps) {
  const [name, setName] = useState('')
  const [goal, setGoal] = useState('')
  const [scope, setScope] = useState('')
  const [owner, setOwner] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim() || !goal.trim() || !owner.trim()) return
    onSubmit(name.trim(), goal.trim(), scope.trim(), owner.trim())
  }

  return (
    <div className="modal-overlay" onClick={onClose} role="dialog" aria-modal="true" aria-labelledby="create-pipeline-plan-title">
      <div className="modal modal--wide" onClick={(e) => e.stopPropagation()}>
        <form onSubmit={handleSubmit}>
          <header className="modal__header">
            <h2 id="create-pipeline-plan-title" className="modal__title">
              <Plus size={16} aria-hidden="true" />
              New Pipeline Plan
            </h2>
            <button type="button" onClick={onClose} className="modal__close" aria-label="Close create dialog">
              <X size={16} />
            </button>
          </header>

          <div className="modal__body" style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            <div>
              <label htmlFor="plan-name" style={{ fontSize: '0.8125rem', fontWeight: 500 }}>
                Plan Name <span style={{ color: 'var(--color-error)' }}>*</span>
              </label>
              <input
                id="plan-name"
                className="input"
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Login Vertical — Phase 9"
                required
                autoFocus
                style={{ marginTop: '0.25rem' }}
              />
            </div>

            <div>
              <label htmlFor="plan-goal" style={{ fontSize: '0.8125rem', fontWeight: 500 }}>
                Goal <span style={{ color: 'var(--color-error)' }}>*</span>
              </label>
              <textarea
                id="plan-goal"
                className="input input--textarea"
                rows={3}
                value={goal}
                onChange={(e) => setGoal(e.target.value)}
                placeholder="What outcome does this plan achieve?"
                required
                style={{ marginTop: '0.25rem' }}
              />
            </div>

            <div>
              <label htmlFor="plan-scope" style={{ fontSize: '0.8125rem', fontWeight: 500 }}>
                Scope
              </label>
              <textarea
                id="plan-scope"
                className="input input--textarea"
                rows={2}
                value={scope}
                onChange={(e) => setScope(e.target.value)}
                placeholder="Boundaries and areas covered (optional)"
                style={{ marginTop: '0.25rem' }}
              />
            </div>

            <div>
              <label htmlFor="plan-owner" style={{ fontSize: '0.8125rem', fontWeight: 500 }}>
                Owner <span style={{ color: 'var(--color-error)' }}>*</span>
              </label>
              <input
                id="plan-owner"
                className="input"
                type="text"
                value={owner}
                onChange={(e) => setOwner(e.target.value)}
                placeholder="e.g. dmitri, iris, yusuf"
                required
                style={{ marginTop: '0.25rem' }}
              />
            </div>
          </div>

          <footer className="modal__footer">
            <button type="button" className="btn btn--ghost" onClick={onClose} disabled={isSubmitting}>
              Cancel
            </button>
            <button type="submit" className="btn btn--primary" disabled={isSubmitting || !name.trim() || !goal.trim() || !owner.trim()}>
              {isSubmitting ? <Loader2 size={14} className="spin" /> : <Plus size={14} />}
              Create Plan
            </button>
          </footer>
        </form>
      </div>
    </div>
  )
}

// ── Plan Detail Modal ──

interface PlanDetailModalProps {
  planId: string | null
  onClose: () => void
  onAdvance: (action: string) => void
  isActing: boolean
}

function PlanDetailModal({ planId, onClose, onAdvance, isActing }: PlanDetailModalProps) {
  const { data: plan, isLoading, isError } = useIntakePlan(planId)
  const { data: auditData } = useIntakePlanAudit(planId)

  if (!planId) return null

  const actions = plan ? getStageActions(plan.current_stage, plan.status === 'ready') : []

  return (
    <div className="modal-overlay" onClick={onClose} role="dialog" aria-modal="true" aria-labelledby="pipeline-detail-title">
      <div className="modal modal--wide" onClick={(e) => e.stopPropagation()}>
        {isLoading && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', padding: '2rem', justifyContent: 'center', color: 'var(--color-text-muted)' }}>
            <Loader2 size={16} className="spin" />
            <span>Loading plan details…</span>
          </div>
        )}

        {isError && (
          <div className="banner banner--error" role="alert">
            <AlertCircle size={14} />
            <span>Failed to load plan details.</span>
          </div>
        )}

        {plan && (
          <>
            <header className="modal__header">
              <h2 id="pipeline-detail-title" className="modal__title">{plan.name}</h2>
              <button type="button" onClick={onClose} className="modal__close" aria-label="Close plan details">
                <X size={16} />
              </button>
            </header>

            <div className="modal__body">
              {/* Status badges */}
              <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem', flexWrap: 'wrap' }}>
                <span
                  className="pipeline-stage-badge"
                  style={{
                    background: `${STAGE_COLORS[plan.current_stage as PipelineStage] || '#6b7280'}22`,
                    color: STAGE_COLORS[plan.current_stage as PipelineStage] || '#9ca3af',
                  }}
                >
                  Stage: {plan.current_stage}
                </span>
                {plan.status === 'ready' && (
                  <span className="pipeline-stage-badge chip--release-ready">
                    <CheckCircle2 size={12} aria-hidden="true" />
                    Ready
                  </span>
                )}
              </div>

              {/* Key info grid */}
              <div className="plan-detail-grid">
                <div className="plan-detail-field">
                  <span className="plan-detail-label">Goal</span>
                  <span className="plan-detail-value">{plan.goal}</span>
                </div>
                <div className="plan-detail-field">
                  <span className="plan-detail-label">Scope</span>
                  <span className="plan-detail-value">{plan.scope || <em style={{ color: 'var(--color-text-muted)' }}>Not specified</em>}</span>
                </div>
                <div className="plan-detail-field">
                  <span className="plan-detail-label">Owner</span>
                  <span className="plan-detail-value">{plan.owner}</span>
                </div>
                <div className="plan-detail-field">
                  <span className="plan-detail-label">Next Action</span>
                  <span className="plan-detail-value">{plan.next_action || <em style={{ color: 'var(--color-text-muted)' }}>None</em>}</span>
                </div>
                <div className="plan-detail-field">
                  <span className="plan-detail-label">Created</span>
                  <span className="plan-detail-value">{formatDate(plan.created_at)}</span>
                </div>
                <div className="plan-detail-field">
                  <span className="plan-detail-label">Updated</span>
                  <span className="plan-detail-value">{formatDate(plan.updated_at)}</span>
                </div>
              </div>

              {/* Constraints & Assumptions */}
              {(plan.constraints?.length > 0 || plan.assumptions?.length > 0) && (
                <div className="plan-detail-section">
                  {(plan.constraints?.length > 0) && (
                    <div>
                      <h4 className="plan-detail-section-title">Constraints</h4>
                      <ul className="plan-detail-list">
                        {plan.constraints.map((c: string, i: number) => <li key={i}>{c}</li>)}
                      </ul>
                    </div>
                  )}
                  {(plan.assumptions?.length > 0) && (
                    <div>
                      <h4 className="plan-detail-section-title">Assumptions</h4>
                      <ul className="plan-detail-list">
                        {plan.assumptions.map((a: string, i: number) => <li key={i}>{a}</li>)}
                      </ul>
                    </div>
                  )}
                </div>
              )}

              {/* Architecture Summary */}
              {plan.architecture_summary && (
                <div className="plan-detail-section">
                  <h4 className="plan-detail-section-title">
                    <Layers size={14} aria-hidden="true" />
                    Architecture Summary
                  </h4>
                  <p style={{ fontSize: '0.875rem', color: 'var(--color-text-secondary)', margin: 0, whiteSpace: 'pre-wrap' }}>
                    {plan.architecture_summary}
                  </p>
                </div>
              )}

              {/* Core Areas */}
              {plan.core_areas?.length > 0 && (
                <div className="plan-detail-section">
                  <h4 className="plan-detail-section-title">Core Areas ({plan.core_areas.length})</h4>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                    {plan.core_areas.map((area: CoreArea) => (
                      <div key={area.id} className="plan-core-area-card">
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.25rem' }}>
                          <span style={{ fontWeight: 600, fontSize: '0.875rem' }}>{area.name}</span>
                          <span style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)' }}>{area.owning_skill}</span>
                        </div>
                        <p style={{ fontSize: '0.8125rem', color: 'var(--color-text-secondary)', margin: '0 0 0.25rem' }}>{area.description}</p>
                        {area.tasks?.length > 0 && (
                          <ul style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)', margin: 0, paddingLeft: '1rem' }}>
                            {area.tasks.map((t) => <li key={t.id}>{t.title} ({t.status})</li>)}
                          </ul>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Decisions */}
              {plan.decisions?.length > 0 && (
                <div className="plan-detail-section">
                  <h4 className="plan-detail-section-title">Decisions ({plan.decisions.length})</h4>
                  {plan.decisions.map((d: PlanDecision, i: number) => (
                    <div key={i} className="plan-decision-row">
                      <span style={{ fontWeight: 500 }}>{d.title}</span>
                      <p style={{ fontSize: '0.8125rem', color: 'var(--color-text-secondary)', margin: '0.125rem 0' }}>{d.rationale}</p>
                      <span style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)' }}>By {d.decided_by} on {formatDate(d.decided_at)}</span>
                    </div>
                  ))}
                </div>
              )}

              {/* Risks */}
              {plan.risks?.length > 0 && (
                <div className="plan-detail-section">
                  <h4 className="plan-detail-section-title">
                    <Siren size={14} aria-hidden="true" />
                    Risks ({plan.risks.length})
                  </h4>
                  {plan.risks.map((r: PlanRisk, i: number) => (
                    <div key={i} className="plan-risk-row">
                      <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                        <span className={`risk-severity risk-severity--${r.severity}`}>{r.severity}</span>
                        <span style={{ fontWeight: 500 }}>{r.description}</span>
                      </div>
                      <p style={{ fontSize: '0.8125rem', color: 'var(--color-text-secondary)', margin: '0.125rem 0' }}>
                        Mitigation: {r.mitigation} | Owner: {r.owner}
                      </p>
                    </div>
                  ))}
                </div>
              )}

              {/* Work Packets */}
              {plan.work_packets?.length > 0 && (
                <div className="plan-detail-section">
                  <h4 className="plan-detail-section-title">Work Packets ({plan.work_packets.length})</h4>
                  {plan.work_packets.map((wp: WorkPacket) => (
                    <div key={wp.id} className="plan-packet-card">
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <span style={{ fontWeight: 600, fontSize: '0.875rem' }}>{wp.title}</span>
                        <span className="packet-status-badge">{wp.status}</span>
                      </div>
                      <p style={{ fontSize: '0.8125rem', color: 'var(--color-text-secondary)', margin: '0.25rem 0' }}>{wp.description}</p>
                      {wp.target_files?.length > 0 && (
                        <div style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)' }}>
                          Files: {wp.target_files.join(', ')}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}

              {/* Phases */}
              {plan.phases?.length > 0 && (
                <div className="plan-detail-section">
                  <h4 className="plan-detail-section-title">Phases ({plan.phases.length})</h4>
                  {plan.phases.map((ph: PlanPhase) => (
                    <div key={ph.id} className="plan-phase-row">
                      <span style={{ fontWeight: 500 }}>{ph.name}</span>
                      <span style={{ fontSize: '0.8125rem', color: 'var(--color-text-secondary)' }}>{ph.status}</span>
                    </div>
                  ))}
                </div>
              )}

              {/* Audit Log */}
              {(auditData?.entries?.length ?? 0) > 0 && (
                <div className="plan-detail-section">
                  <h4 className="plan-detail-section-title">
                    <History size={14} aria-hidden="true" />
                    Audit Log
                  </h4>
                  <div style={{ maxHeight: '200px', overflowY: 'auto', fontSize: '0.8125rem' }}>
                    {auditData!.entries.map((entry: PlanAuditEntry, i: number) => (
                      <div key={i} style={{ display: 'flex', gap: '0.5rem', padding: '0.25rem 0', borderBottom: '1px solid var(--color-border-default)' }}>
                        <span style={{ color: 'var(--color-text-muted)', whiteSpace: 'nowrap' }}>{formatDate(entry.timestamp)}</span>
                        <span style={{ color: 'var(--color-accent-secondary)', fontWeight: 500 }}>{entry.actor}</span>
                        <span>{entry.action}</span>
                        <span style={{ color: 'var(--color-text-muted)' }}>{entry.detail}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Stage advancement */}
              {actions.length > 0 && (
                <div style={{ marginTop: '1rem', borderTop: '1px solid var(--color-border-default)', paddingTop: '1rem' }}>
                  <h3 style={{ fontSize: '0.875rem', fontWeight: 600, margin: '0 0 0.75rem' }}>Stage Actions</h3>
                  <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                    {actions.map((act) => (
                      <button
                        key={act.action}
                        type="button"
                        className="btn btn--primary"
                        onClick={() => onAdvance(act.action)}
                        disabled={isActing}
                      >
                        {isActing ? <Loader2 size={14} className="spin" /> : <ArrowRight size={14} />}
                        {act.label}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  )
}

// ── Pipeline Column ──

interface PipelineColumnProps {
  stage: (typeof PIPELINE_STAGES)[number]
  plans: IntakePlan[]
  isLoading: boolean
  onPlanClick: (id: string) => void
}

function PipelineColumn({ stage, plans, isLoading, onPlanClick }: PipelineColumnProps) {
  const StageIcon = stage.icon
  const stageKey = stage.key
  const stageColor = STAGE_COLORS[stageKey]

  return (
    <div className="pipeline-column" aria-label={`${stage.label} column`}>
      <div
        className="pipeline-column__header"
        style={{ borderBottomColor: stageColor }}
      >
        <StageIcon size={16} aria-hidden="true" style={{ color: stageColor }} />
        <span className="pipeline-column__title">{stage.label}</span>
        <span className="pipeline-column__count">{plans.length}</span>
      </div>

      <div className="pipeline-column__body">
        {isLoading && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', justifyContent: 'center', padding: '2rem', color: 'var(--color-text-muted)' }}>
            <Loader2 size={14} className="spin" />
            <span style={{ fontSize: '0.8125rem' }}>Loading…</span>
          </div>
        )}

        {!isLoading && plans.length === 0 && (
          <div className="pipeline-column__empty">
            <p style={{ fontSize: '0.8125rem', color: 'var(--color-text-muted)', textAlign: 'center', margin: 0 }}>
              No plans in this stage
            </p>
          </div>
        )}

        {!isLoading && plans.map((plan) => (
          <article
            key={plan.id}
            className="pipeline-card"
            onClick={() => onPlanClick(plan.id)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onPlanClick(plan.id) }}}
            aria-label={`Plan: ${plan.name}`}
          >
            <header className="pipeline-card__header">
              <h3 className="pipeline-card__title">{plan.name}</h3>
              {plan.status === 'ready' && (
                <span className="pipeline-card__ready-badge" title="Ready for execution">
                  <CheckCircle2 size={12} />
                </span>
              )}
            </header>

            <p className="pipeline-card__goal">{plan.goal}</p>

            <div className="pipeline-card__meta">
              <span className="pipeline-card__owner">{plan.owner}</span>
              <span className="pipeline-card__time" title={formatDate(plan.updated_at)}>
                {relativeTime(plan.updated_at)}
              </span>
            </div>

            {plan.work_packets?.length > 0 && (
              <div className="pipeline-card__stats">
                <span>{plan.work_packets.length} packet{plan.work_packets.length !== 1 ? 's' : ''}</span>
              </div>
            )}
          </article>
        ))}
      </div>
    </div>
  )
}

// ── Guided Intake Flow Bar ──

function GuidedIntakeBar() {
  return (
    <div className="guided-intake-bar" role="region" aria-label="Guided intake flow">
      <div className="guided-intake-bar__steps">
        {PIPELINE_STAGES.map((stage, idx) => {
          const StageIcon = stage.icon
          const isLast = idx === PIPELINE_STAGES.length - 1
          return (
            <div key={stage.key} className="guided-intake-step">
              <StageIcon size={14} aria-hidden="true" />
              <span>{stage.label}</span>
              {!isLast && <ChevronRight size={12} className="guided-intake-step__arrow" aria-hidden="true" />}
            </div>
          )
        })}
      </div>
      <p className="guided-intake-bar__hint">
        Create a plan in Intake, then advance it through each stage until it's Ready for execution.
      </p>
    </div>
  )
}

// ── Error Banner ──

function ErrorBanner({ error }: { error: unknown }) {
  return (
    <div className="banner banner--error" role="alert">
      <AlertCircle size={14} aria-hidden="true" />
      <span>{error instanceof Error ? error.message : 'An unexpected error occurred.'}</span>
    </div>
  )
}

// ── Success Banner ──

function SuccessBanner({ message, onDismiss }: { message: string; onDismiss: () => void }) {
  return (
    <div className="banner banner--success" role="status">
      <CheckCircle2 size={14} aria-hidden="true" />
      <span>{message}</span>
      <button type="button" onClick={onDismiss} className="banner__dismiss" aria-label="Dismiss">
        <X size={12} />
      </button>
    </div>
  )
}

// ── Main Intake Pipeline Page ──

export function IntakeBoard() {
  const { actorId } = useSession()
  const [showCreate, setShowCreate] = useState(false)
  const [selectedPlanId, setSelectedPlanId] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const [expandedStage, setExpandedStage] = useState<string | null>(null)

  const { data, isLoading, isError, error, refetch } = useIntakePlans()
  const createPlan = useCreateIntakePlan()
  const refinePlan = useRefineIntakePlan()
  const setArch = useSetIntakeArchitecture()
  const decompose = useDecomposeIntakePlan()
  const packetize = usePacketizeIntakePlan()
  const advanceReady = useAdvanceIntakePlanToReady()

  const allPlans = data?.plans ?? []

  // Group plans by current_stage
  const plansByStage = PIPELINE_STAGES.reduce(
    (acc, stage) => {
      acc[stage.key] = allPlans.filter((p) => (p.current_stage || 'intake') === stage.key)
      return acc
    },
    {} as Record<string, IntakePlan[]>,
  )

  // Toggle expanded stage for mobile/compact view
  const toggleStage = (stage: string) => {
    setExpandedStage((prev) => (prev === stage ? null : stage))
  }

  const handleCreatePlan = (name: string, goal: string, scope: string, owner: string) => {
    createPlan.mutate(
      { name, goal, scope, owner },
      {
        onSuccess: () => {
          setShowCreate(false)
          setSuccessMessage(`Plan "${name}" created successfully.`)
          refetch()
        },
      },
    )
  }

  const handleAdvancePlan = (action: string) => {
    if (!selectedPlanId) return

    const actor = actorId ?? 'unknown'

    switch (action) {
      case 'refine':
        refinePlan.mutate(
          { id: selectedPlanId, body: { constraints: [], assumptions: [], actor } },
          {
            onSuccess: () => {
              setSelectedPlanId(null)
              setSuccessMessage('Plan advanced to Refinement.')
            },
          },
        )
        break
      case 'architecture':
        setArch.mutate(
          { id: selectedPlanId, body: { architecture_summary: '', core_areas: [], actor } },
          {
            onSuccess: () => {
              setSelectedPlanId(null)
              setSuccessMessage('Plan advanced to Architecture.')
            },
          },
        )
        break
      case 'decompose':
        decompose.mutate(
          { id: selectedPlanId, body: { decisions: [], risks: [], phases: [], actor } },
          {
            onSuccess: () => {
              setSelectedPlanId(null)
              setSuccessMessage('Plan advanced to Decomposition.')
            },
          },
        )
        break
      case 'packetize':
        packetize.mutate(
          { id: selectedPlanId, body: { packets: [], actor } },
          {
            onSuccess: () => {
              setSelectedPlanId(null)
              setSuccessMessage('Plan advanced to Packetization.')
            },
          },
        )
        break
      case 'ready':
        advanceReady.mutate(
          { id: selectedPlanId, actor },
          {
            onSuccess: () => {
              setSelectedPlanId(null)
              setSuccessMessage('Plan advanced to Ready.')
            },
          },
        )
        break
    }
  }

  const isActing =
    createPlan.isPending ||
    refinePlan.isPending ||
    setArch.isPending ||
    decompose.isPending ||
    packetize.isPending ||
    advanceReady.isPending

  return (
    <div className="intake-pipeline-page">
      {/* Page header */}
      <div className="page-header">
        <div className="page-header__row">
          <div>
            <h1 className="page-header__title">Intake Pipeline</h1>
            <p className="page-header__description">
              Pipeline intake system — from ideation through ready for execution
            </p>
          </div>
          <button
            type="button"
            className="btn btn--primary"
            onClick={() => setShowCreate(true)}
          >
            <Plus size={14} aria-hidden="true" />
            New Plan
          </button>
        </div>
      </div>

      {/* Guided intake flow indicator */}
      <GuidedIntakeBar />

      {/* Success banner */}
      {successMessage && (
        <SuccessBanner message={successMessage} onDismiss={() => setSuccessMessage(null)} />
      )}

      {/* Error banner */}
      {isError && <ErrorBanner error={error} />}

      {/* Pipeline columns — visible on wide screens */}
      <div className="pipeline-view" role="region" aria-label="Pipeline stage columns">
        {PIPELINE_STAGES.map((stage) => (
          <PipelineColumn
            key={stage.key}
            stage={stage}
            plans={plansByStage[stage.key] || []}
            isLoading={isLoading}
            onPlanClick={(id) => setSelectedPlanId(id)}
          />
        ))}
      </div>

      {/* Mobile/compact accordion view */}
      <div className="pipeline-accordion-view" role="region" aria-label="Pipeline stages accordion">
        {PIPELINE_STAGES.map((stage) => {
          const StageIcon = stage.icon
          const stagePlans = plansByStage[stage.key] || []
          const isExpanded = expandedStage === stage.key
          return (
            <div key={stage.key} className="pipeline-accordion-stage">
              <button
                type="button"
                className="pipeline-accordion-trigger"
                onClick={() => toggleStage(stage.key)}
                aria-expanded={isExpanded}
              >
                <StageIcon size={14} aria-hidden="true" style={{ color: STAGE_COLORS[stage.key] }} />
                <span className="pipeline-accordion-trigger__label">{stage.label}</span>
                <span className="pipeline-accordion-trigger__count">{stagePlans.length}</span>
                <ChevronDown
                  size={14}
                  className={`pipeline-accordion-chevron${isExpanded ? ' pipeline-accordion-chevron--open' : ''}`}
                />
              </button>
              {isExpanded && (
                <div className="pipeline-accordion-body">
                  {isLoading && (
                    <div style={{ padding: '1rem', textAlign: 'center', color: 'var(--color-text-muted)' }}>
                      <Loader2 size={14} className="spin" />
                    </div>
                  )}
                  {!isLoading && stagePlans.length === 0 && (
                    <p style={{ padding: '1rem', textAlign: 'center', color: 'var(--color-text-muted)', fontSize: '0.8125rem', margin: 0 }}>
                      No plans in this stage
                    </p>
                  )}
                  {!isLoading && stagePlans.map((plan) => (
                    <article
                      key={plan.id}
                      className="pipeline-card pipeline-card--compact"
                      onClick={() => setSelectedPlanId(plan.id)}
                      role="button"
                      tabIndex={0}
                      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); setSelectedPlanId(plan.id) }}}
                    >
                      <header className="pipeline-card__header">
                        <h3 className="pipeline-card__title">{plan.name}</h3>
                        {plan.status === 'ready' && <CheckCircle2 size={12} style={{ color: 'var(--color-status-release-ready)' }} />}
                      </header>
                      <p className="pipeline-card__goal">{plan.goal}</p>
                      <div className="pipeline-card__meta">
                        <span className="pipeline-card__owner">{plan.owner}</span>
                        <span className="pipeline-card__time">{relativeTime(plan.updated_at)}</span>
                      </div>
                    </article>
                  ))}
                </div>
              )}
            </div>
          )
        })}
      </div>

      {/* Empty state when no plans at all */}
      {!isLoading && allPlans.length === 0 && !isError && (
        <div className="pipeline-empty-state">
          <ClipboardList size={40} aria-hidden="true" />
          <h3 className="pipeline-empty-state__title">No pipeline plans yet</h3>
          <p className="pipeline-empty-state__desc">
            Create your first plan to start the intake pipeline process.
          </p>
          <button
            type="button"
            className="btn btn--primary"
            onClick={() => setShowCreate(true)}
          >
            <Plus size={14} aria-hidden="true" />
            Create Plan
          </button>
        </div>
      )}

      {/* Create Plan Dialog */}
      {showCreate && (
        <CreatePlanDialog
          onClose={() => setShowCreate(false)}
          onSubmit={handleCreatePlan}
          isSubmitting={createPlan.isPending}
        />
      )}

      {/* Plan Detail Modal */}
      {selectedPlanId && (
        <PlanDetailModal
          planId={selectedPlanId}
          onClose={() => setSelectedPlanId(null)}
          onAdvance={handleAdvancePlan}
          isActing={isActing}
        />
      )}
    </div>
  )
}
