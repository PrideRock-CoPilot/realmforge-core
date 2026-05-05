// ─────────────────────────────────────────────
// API hooks — TanStack Query hooks for live data
// ─────────────────────────────────────────────
// Each hook wraps a backend API endpoint:
//   - Uses customFetch from client.ts for auth headers
//   - Provides React Query caching, retry, loading, error states
//
// BOARD STATE MAPPING (backend status → frontend BoardState):
//   draft        → intake
//   in_review    → review
//   approved     → approved
//   in_progress  → execution
//   completed    → released
//   blocked      → blocked
//   archived     → archived

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { customFetch } from './client'

// ── Types ──

export interface BoardPlan {
  id: string
  title: string
  status: string
  work_path_refs: string[]
  created_at: string
  updated_at: string
}

export interface BoardPlanListResponse {
  plans: BoardPlan[]
  total: number
}

export interface CreateBoardPlanRequest {
  title: string
  work_path_refs?: string[]
}

export interface WorkPathSummary {
  id: string
  name: string
  description: string
  node_count: number
}

export interface AuditEvent {
  id: string
  event_type: string
  actor_id: string
  entity_type: string
  entity_id: string
  payload: Record<string, unknown>
  timestamp: string
  chain_hash: string
}

export interface AuditEventsResponse {
  events: AuditEvent[]
  total: number
}

export interface Bundle {
  id: string
  name: string
  version: string
  status: string
  created_at: string
}

export interface BundleListResponse {
  bundles: Bundle[]
  total: number
}

export interface RollbackAnchor {
  id: string
  snapshot_id: string
  created_at: string
  description: string
}

// ── Intake Pipeline Types ──

export interface CoreArea {
  id: string
  name: string
  description: string
  tasks: AreaTask[]
  owning_skill: string
}

export interface AreaTask {
  id: string
  title: string
  description: string
  status: string
  target_file_globs: string[]
  estimated_effort_hours: number
}

export interface PlanDecision {
  title: string
  rationale: string
  decided_by: string
  decided_at: string
  consequences: string[]
}

export interface PlanRisk {
  description: string
  severity: string
  likelihood: string
  mitigation: string
  owner: string
}

export interface PlanPhase {
  id: string
  name: string
  description: string
  order: number
  status: string
  target_completion: string | null
}

export interface WorkPacket {
  id: string
  title: string
  description: string
  target_files: string[]
  allowed_files: string[]
  forbidden_files: string[]
  dependencies: string[]
  inputs: Record<string, unknown>
  expected_output: string
  validation_rules: string[]
  test_requirements: string[]
  acceptance_criteria: string[]
  status: string
}

export interface PlanAuditEntry {
  timestamp: string
  actor: string
  action: string
  detail: string
}

export interface IntakePlan {
  id: string
  name: string
  goal: string
  scope: string
  constraints: string[]
  assumptions: string[]
  architecture_summary: string
  core_areas: CoreArea[]
  decisions: PlanDecision[]
  risks: PlanRisk[]
  phases: PlanPhase[]
  work_packets: WorkPacket[]
  status: string
  current_stage: string
  next_action: string
  owner: string
  audit_log: PlanAuditEntry[]
  created_at: string
  updated_at: string
}

export interface IntakePlanListResponse {
  plans: IntakePlan[]
  total: number
}

export interface CreateIntakePlanRequest {
  name: string
  goal: string
  scope: string
  owner: string
}

export interface RefinePlanRequest {
  constraints: string[]
  assumptions: string[]
  actor: string
}

export interface SetArchitectureRequest {
  architecture_summary: string
  core_areas: CoreArea[]
  actor: string
}

export interface DecomposeRequest {
  decisions: PlanDecision[]
  risks: PlanRisk[]
  phases: PlanPhase[]
  actor: string
}

export interface GeneratePacketsRequest {
  packets: WorkPacket[]
  actor: string
}

export interface StageActionRequest {
  actor: string
}

export interface IntakeAuditResponse {
  entries: PlanAuditEntry[]
  total: number
}

// ── Query key factories ──

export const queryKeys = {
  boardPlans: ['board-plans'] as const,
  boardPlan: (id: string) => ['board-plan', id] as const,
  workPath: (id: string) => ['work-path', id] as const,
  auditEvents: (params?: Record<string, unknown>) => ['audit-events', params] as const,
  bundles: ['bundles'] as const,
  bundle: (id: string) => ['bundle', id] as const,
  snapshots: ['snapshots'] as const,
  intakePlans: ['intake-plans'] as const,
  intakePlan: (id: string) => ['intake-plan', id] as const,
}

// ── Board Plans Hooks ──

export function useBoardPlans(status?: string) {
  const params = status ? `?status=${encodeURIComponent(status)}` : ''
  return useQuery<BoardPlanListResponse>({
    queryKey: [...queryKeys.boardPlans, status],
    queryFn: () => customFetch<BoardPlanListResponse>(`/v1/boards/plans${params}`),
  })
}

export function useCreateBoardPlan() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: CreateBoardPlanRequest) =>
      customFetch<BoardPlan>('/v1/boards/plans', {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boardPlans })
    },
  })
}

export function useSubmitPlan() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (planId: string) =>
      customFetch<BoardPlan>(`/v1/boards/plans/${encodeURIComponent(planId)}/submit`, {
        method: 'POST',
        body: '{}',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boardPlans })
    },
  })
}

export function useApprovePlan() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ planId, approver, comment }: { planId: string; approver: string; comment?: string }) =>
      customFetch<BoardPlan>(`/v1/boards/plans/${encodeURIComponent(planId)}/approve`, {
        method: 'POST',
        body: JSON.stringify({ approver, comment }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boardPlans })
    },
  })
}

export function useRejectPlan() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ planId, approver, comment }: { planId: string; approver: string; comment?: string }) =>
      customFetch<BoardPlan>(`/v1/boards/plans/${encodeURIComponent(planId)}/reject`, {
        method: 'POST',
        body: JSON.stringify({ approver, comment }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boardPlans })
    },
  })
}

// ── Work Path Hooks ──

export function useWorkPath(id: string | null) {
  return useQuery<WorkPathSummary>({
    queryKey: queryKeys.workPath(id ?? ''),
    queryFn: () => customFetch<WorkPathSummary>(`/v1/work-paths/${encodeURIComponent(id!)}`),
    enabled: !!id,
  })
}

// ── Audit / Evidence Hooks ──

export function useAuditEvents(params?: {
  event_type?: string
  limit?: number
  offset?: number
}) {
  const searchParams = new URLSearchParams()
  if (params?.event_type) searchParams.set('event_type', params.event_type)
  if (params?.limit) searchParams.set('limit', String(params.limit))
  if (params?.offset) searchParams.set('offset', String(params.offset))

  const query = searchParams.toString()
  return useQuery<AuditEventsResponse>({
    queryKey: queryKeys.auditEvents(params),
    queryFn: () => customFetch<AuditEventsResponse>(`/v1/audit/events${query ? `?${query}` : ''}`),
  })
}

// ── Bundle / Release Hooks ──

export function useBundles() {
  return useQuery<BundleListResponse>({
    queryKey: queryKeys.bundles,
    queryFn: () => customFetch<BundleListResponse>('/v1/bundles'),
  })
}

// ── Intake Pipeline Hooks ──

export function useIntakePlans(stage?: string) {
  const params = stage ? `?stage=${encodeURIComponent(stage)}` : ''
  return useQuery<IntakePlanListResponse>({
    queryKey: [...queryKeys.intakePlans, stage],
    queryFn: () => customFetch<IntakePlanListResponse>(`/v1/intake/plans${params}`),
  })
}

export function useIntakePlan(id: string | null) {
  return useQuery<IntakePlan>({
    queryKey: queryKeys.intakePlan(id ?? ''),
    queryFn: () => customFetch<IntakePlan>(`/v1/intake/plans/${encodeURIComponent(id!)}`),
    enabled: !!id,
  })
}

export function useCreateIntakePlan() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: CreateIntakePlanRequest) =>
      customFetch<IntakePlan>('/v1/intake/plans', {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlans })
    },
  })
}

export function useRefineIntakePlan() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, body }: { id: string; body: RefinePlanRequest }) =>
      customFetch<IntakePlan>(`/v1/intake/plans/${encodeURIComponent(id)}/refine`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: (_data, vars) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlans })
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlan(vars.id) })
    },
  })
}

export function useSetIntakeArchitecture() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, body }: { id: string; body: SetArchitectureRequest }) =>
      customFetch<IntakePlan>(`/v1/intake/plans/${encodeURIComponent(id)}/architecture`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: (_data, vars) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlans })
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlan(vars.id) })
    },
  })
}

export function useDecomposeIntakePlan() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, body }: { id: string; body: DecomposeRequest }) =>
      customFetch<IntakePlan>(`/v1/intake/plans/${encodeURIComponent(id)}/decompose`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: (_data, vars) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlans })
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlan(vars.id) })
    },
  })
}

export function usePacketizeIntakePlan() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, body }: { id: string; body: GeneratePacketsRequest }) =>
      customFetch<IntakePlan>(`/v1/intake/plans/${encodeURIComponent(id)}/packetize`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: (_data, vars) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlans })
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlan(vars.id) })
    },
  })
}

export function useAdvanceIntakePlanToReady() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, actor }: { id: string; actor: string }) =>
      customFetch<IntakePlan>(`/v1/intake/plans/${encodeURIComponent(id)}/ready`, {
        method: 'POST',
        body: JSON.stringify({ actor }),
      }),
    onSuccess: (_data, vars) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlans })
      queryClient.invalidateQueries({ queryKey: queryKeys.intakePlan(vars.id) })
    },
  })
}

export function useIntakePlanAudit(id: string | null) {
  return useQuery<IntakeAuditResponse>({
    queryKey: [...queryKeys.intakePlan(id ?? ''), 'audit'],
    queryFn: () => customFetch<IntakeAuditResponse>(`/v1/intake/plans/${encodeURIComponent(id!)}/audit`),
    enabled: !!id,
  })
}

// ── Helper: Map backend status to BoardState ──

export function mapStatusToBoardState(status: string): import('@/components/ui/board-state-chip').BoardState {
  switch (status) {
    case 'draft':          return 'intake'
    case 'in_review':      return 'review'
    case 'approved':       return 'approved'
    case 'in_progress':    return 'execution'
    case 'completed':      return 'released'
    case 'blocked':        return 'blocked'
    case 'archived':       return 'archived'
    default:               return 'intake'
  }
}
