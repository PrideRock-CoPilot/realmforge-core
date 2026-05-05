// ─────────────────────────────────────────────
// intake.rs — Intake Pipeline API Routes
// ─────────────────────────────────────────────
// Six-stage pipeline: intake → refinement → architecture
// → decomposition → packetization → ready
//
// Each endpoint advances the plan through exactly one stage.
// ─────────────────────────────────────────────

use authority_domain::plan::{
    CoreArea, Plan, PlanAuditEntry, PlanDecision, PlanPhase, PlanRisk, WorkPacket,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use control_service::ServiceContext;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

// ── Request Types ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakePlanRequest {
    /// Short name for the plan (e.g. "Login Vertical — Phase 9")
    pub name: String,
    /// One-sentence user outcome
    pub goal: String,
    /// Scope boundary — what is and is not included
    pub scope: String,
    /// Owner actor ID
    pub owner: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RefinePlanRequest {
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub actor: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetArchitectureRequest {
    pub architecture_summary: String,
    pub core_areas: Vec<CoreArea>,
    pub actor: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DecomposeRequest {
    pub decisions: Vec<PlanDecision>,
    pub risks: Vec<PlanRisk>,
    pub phases: Vec<PlanPhase>,
    pub actor: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GeneratePacketsRequest {
    pub packets: Vec<WorkPacket>,
    pub actor: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct StageActionRequest {
    pub actor: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct ListIntakePlansQuery {
    /// Filter by pipeline stage: intake, refinement, architecture, decomposition, packetization, ready
    pub stage: Option<String>,
}

// ── Response Types ──

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntakePlanResponse {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub scope: String,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub architecture_summary: String,
    pub core_areas: Vec<CoreArea>,
    pub decisions: Vec<PlanDecision>,
    pub risks: Vec<PlanRisk>,
    pub phases: Vec<PlanPhase>,
    pub work_packets: Vec<WorkPacket>,
    pub status: String,
    pub current_stage: String,
    pub next_action: String,
    pub owner: String,
    pub audit_log: Vec<PlanAuditEntry>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntakePlanListResponse {
    pub plans: Vec<IntakePlanResponse>,
    pub total: u64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntakeAuditResponse {
    pub entries: Vec<PlanAuditEntry>,
    pub total: u64,
}

// ── Helpers ──

fn plan_to_response(plan: &Plan) -> IntakePlanResponse {
    IntakePlanResponse {
        id: plan.id.clone(),
        name: plan.name.clone(),
        goal: plan.goal.clone(),
        scope: plan.scope.clone(),
        constraints: plan.constraints.clone(),
        assumptions: plan.assumptions.clone(),
        architecture_summary: plan.architecture_summary.clone(),
        core_areas: plan.core_areas.clone(),
        decisions: plan.decisions.clone(),
        risks: plan.risks.clone(),
        phases: plan.phases.clone(),
        work_packets: plan.work_packets.clone(),
        status: plan.status.to_string(),
        current_stage: plan.current_stage.as_str().to_string(),
        next_action: plan.next_action.clone(),
        owner: plan.owner.clone(),
        audit_log: plan.audit_log.clone(),
        created_at: plan.created_at.to_rfc3339(),
        updated_at: plan.updated_at.to_rfc3339(),
    }
}

fn response(plan: &Plan) -> Json<IntakePlanResponse> {
    Json(plan_to_response(plan))
}

// ── Route Handlers ──

/// POST /v1/intake/plans — Stage 1: Create plan from user intent.
#[utoipa::path(
    post,
    path = "/v1/intake/plans",
    operation_id = "create_intake_plan",
    summary = "Stage 1: Create intake plan from raw intent.",
    tag = "intake",
    request_body = CreateIntakePlanRequest,
    responses(
        (status = 201, description = "Plan created in Intake stage", body = IntakePlanResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn create_plan(
    State(ctx): State<ServiceContext>,
    Json(body): Json<CreateIntakePlanRequest>,
) -> Result<(StatusCode, Json<IntakePlanResponse>), ApiError> {
    let plan = ctx
        .intake
        .create_plan(body.name, body.goal, body.scope, body.owner)
        .await
        .map_err(ApiError::BadRequest)?;
    Ok((StatusCode::CREATED, response(&plan)))
}

/// POST /v1/intake/plans/:id/refine — Stage 2: Add constraints and assumptions.
#[utoipa::path(
    post,
    path = "/v1/intake/plans/{id}/refine",
    operation_id = "refine_intake_plan",
    summary = "Stage 2: Refine plan with constraints and assumptions.",
    tag = "intake",
    params(("id" = String, Path, description = "Plan ID")),
    request_body = RefinePlanRequest,
    responses(
        (status = 200, description = "Plan advanced to Refinement", body = IntakePlanResponse),
        (status = 400, description = "Invalid request or stage", body = crate::error::ProblemDetails),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn refine_plan(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
    Json(body): Json<RefinePlanRequest>,
) -> Result<Json<IntakePlanResponse>, ApiError> {
    let plan = ctx
        .intake
        .refine_plan(&id, body.constraints, body.assumptions, &body.actor)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::NotFound(e)
            } else {
                ApiError::BadRequest(e)
            }
        })?;
    Ok(response(&plan))
}

/// POST /v1/intake/plans/:id/architecture — Stage 3: Set architecture and core areas.
#[utoipa::path(
    post,
    path = "/v1/intake/plans/{id}/architecture",
    operation_id = "set_intake_architecture",
    summary = "Stage 3: Set architecture summary and core areas.",
    tag = "intake",
    params(("id" = String, Path, description = "Plan ID")),
    request_body = SetArchitectureRequest,
    responses(
        (status = 200, description = "Plan advanced to Architecture", body = IntakePlanResponse),
        (status = 400, description = "Invalid request or stage", body = crate::error::ProblemDetails),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn set_architecture(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
    Json(body): Json<SetArchitectureRequest>,
) -> Result<Json<IntakePlanResponse>, ApiError> {
    let plan = ctx
        .intake
        .set_architecture(&id, body.architecture_summary, body.core_areas, &body.actor)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::NotFound(e)
            } else {
                ApiError::BadRequest(e)
            }
        })?;
    Ok(response(&plan))
}

/// POST /v1/intake/plans/:id/decompose — Stage 4: Record decisions, risks, and phases.
#[utoipa::path(
    post,
    path = "/v1/intake/plans/{id}/decompose",
    operation_id = "decompose_intake_plan",
    summary = "Stage 4: Decompose into decisions, risks, and phases.",
    tag = "intake",
    params(("id" = String, Path, description = "Plan ID")),
    request_body = DecomposeRequest,
    responses(
        (status = 200, description = "Plan advanced to Decomposition", body = IntakePlanResponse),
        (status = 400, description = "Invalid request or stage", body = crate::error::ProblemDetails),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn decompose(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
    Json(body): Json<DecomposeRequest>,
) -> Result<Json<IntakePlanResponse>, ApiError> {
    let plan = ctx
        .intake
        .decompose(&id, body.decisions, body.risks, body.phases, &body.actor)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::NotFound(e)
            } else {
                ApiError::BadRequest(e)
            }
        })?;
    Ok(response(&plan))
}

/// POST /v1/intake/plans/:id/packetize — Stage 5: Generate work packets.
#[utoipa::path(
    post,
    path = "/v1/intake/plans/{id}/packetize",
    operation_id = "packetize_intake_plan",
    summary = "Stage 5: Generate work packets from core area tasks.",
    tag = "intake",
    params(("id" = String, Path, description = "Plan ID")),
    request_body = GeneratePacketsRequest,
    responses(
        (status = 200, description = "Plan advanced to Packetization", body = IntakePlanResponse),
        (status = 400, description = "Invalid request or stage", body = crate::error::ProblemDetails),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn generate_packets(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
    Json(body): Json<GeneratePacketsRequest>,
) -> Result<Json<IntakePlanResponse>, ApiError> {
    let plan = ctx
        .intake
        .generate_packets(&id, body.packets, &body.actor)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::NotFound(e)
            } else {
                ApiError::BadRequest(e)
            }
        })?;
    Ok(response(&plan))
}

/// POST /v1/intake/plans/:id/ready — Stage 6: Advance to Ready.
#[utoipa::path(
    post,
    path = "/v1/intake/plans/{id}/ready",
    operation_id = "ready_intake_plan",
    summary = "Stage 6: Validate and advance plan to Ready.",
    tag = "intake",
    params(("id" = String, Path, description = "Plan ID")),
    request_body = StageActionRequest,
    responses(
        (status = 200, description = "Plan advanced to Ready", body = IntakePlanResponse),
        (status = 400, description = "Validation failed or wrong stage", body = crate::error::ProblemDetails),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn advance_to_ready(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
    Json(body): Json<StageActionRequest>,
) -> Result<Json<IntakePlanResponse>, ApiError> {
    let plan = ctx
        .intake
        .advance_to_ready(&id, &body.actor)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::NotFound(e)
            } else {
                ApiError::BadRequest(e)
            }
        })?;
    Ok(response(&plan))
}

/// GET /v1/intake/plans — List plans, optionally filtered by stage.
#[utoipa::path(
    get,
    path = "/v1/intake/plans",
    operation_id = "list_intake_plans",
    summary = "List intake plans with optional stage filter.",
    tag = "intake",
    params(ListIntakePlansQuery),
    responses(
        (status = 200, description = "Plans listed", body = IntakePlanListResponse),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn list_plans(
    State(ctx): State<ServiceContext>,
    Query(query): Query<ListIntakePlansQuery>,
) -> Result<Json<IntakePlanListResponse>, ApiError> {
    let plans = ctx
        .intake
        .list_plans(query.stage.as_deref())
        .await
        .map_err(ApiError::Internal)?;
    let items: Vec<IntakePlanResponse> = plans.iter().map(plan_to_response).collect();
    let total = items.len() as u64;
    Ok(Json(IntakePlanListResponse {
        plans: items,
        total,
    }))
}

/// GET /v1/intake/plans/:id — Get a single plan by ID.
#[utoipa::path(
    get,
    path = "/v1/intake/plans/{id}",
    operation_id = "get_intake_plan",
    summary = "Get intake plan details.",
    tag = "intake",
    params(("id" = String, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "Plan details", body = IntakePlanResponse),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_plan(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
) -> Result<Json<IntakePlanResponse>, ApiError> {
    let plan = ctx.intake.get_plan(&id).await.map_err(ApiError::NotFound)?;
    Ok(response(&plan))
}

/// GET /v1/intake/plans/:id/audit — Get audit log for a plan.
#[utoipa::path(
    get,
    path = "/v1/intake/plans/{id}/audit",
    operation_id = "get_intake_plan_audit",
    summary = "Get audit log for an intake plan.",
    tag = "intake",
    params(("id" = String, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "Audit log", body = IntakeAuditResponse),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_plan_audit(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
) -> Result<Json<IntakeAuditResponse>, ApiError> {
    let entries = ctx
        .intake
        .get_plan_audit(&id)
        .await
        .map_err(ApiError::NotFound)?;
    let total = entries.len() as u64;
    Ok(Json(IntakeAuditResponse { entries, total }))
}
