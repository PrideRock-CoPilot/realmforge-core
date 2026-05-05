// ─────────────────────────────────────────────
// intake_service.rs — Intake Pipeline Orchestrator
// ─────────────────────────────────────────────
// Manages the 6-stage intake pipeline:
//   intake → refinement → architecture → decomposition
//   → packetization → ready
//
// Each stage enforces its validation rules before
// allowing advancement. All mutations are logged.
// ─────────────────────────────────────────────

use authority_domain::plan::*;
use authority_domain::PlanStatus;
use chrono::Utc;
use control_store::plan_store::PlanStore;
use std::sync::Arc;

#[derive(Clone)]
pub struct IntakeService {
    store: Arc<dyn PlanStore>,
}

impl IntakeService {
    pub fn new(store: Arc<dyn PlanStore>) -> Self {
        Self { store }
    }

    // ── Stage 1: Intake ──

    /// Create a plan from raw user intent.
    /// This is the first stage — accepts minimal input.
    pub async fn create_plan(
        &self,
        name: String,
        goal: String,
        scope: String,
        owner: String,
    ) -> Result<Plan, String> {
        if name.trim().is_empty() {
            return Err("Plan name is required".to_string());
        }
        if goal.trim().is_empty() {
            return Err("Plan goal is required".to_string());
        }
        if scope.trim().is_empty() {
            return Err("Plan scope is required".to_string());
        }

        let now = Utc::now();
        let plan = Plan {
            id: format!(
                "plan_{}",
                uuid::Uuid::new_v4()
                    .to_string()
                    .split('-')
                    .next()
                    .unwrap_or("0")
            ),
            name: name.trim().to_string(),
            goal: goal.trim().to_string(),
            scope: scope.trim().to_string(),
            constraints: Vec::new(),
            assumptions: Vec::new(),
            architecture_summary: String::new(),
            core_areas: Vec::new(),
            decisions: Vec::new(),
            risks: Vec::new(),
            phases: Vec::new(),
            work_packets: Vec::new(),
            status: PlanStatus::Draft,
            current_stage: PipelineStage::Intake,
            next_action: "Refine the plan goal, scope, and constraints".to_string(),
            owner: owner.trim().to_string(),
            audit_log: vec![PlanAuditEntry {
                timestamp: now,
                actor: owner.trim().to_string(),
                action: "plan.created".to_string(),
                detail: format!("Plan '{}' created at Intake stage", name.trim()),
            }],
            created_at: now,
            updated_at: now,
        };

        self.store
            .save_plan(&plan)
            .await
            .map_err(|e| e.to_string())?;
        Ok(plan)
    }

    // ── Stage 2: Refinement ──

    /// Refine the plan with constraints, assumptions, and risks.
    /// Must be in Intake stage.
    pub async fn refine_plan(
        &self,
        plan_id: &str,
        constraints: Vec<String>,
        assumptions: Vec<String>,
        actor: &str,
    ) -> Result<Plan, String> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await
            .map_err(|e| e.to_string())?;

        if plan.current_stage != PipelineStage::Intake {
            return Err(format!(
                "Plan is in stage '{}' — must be in 'intake' to refine",
                plan.current_stage.as_str()
            ));
        }

        plan.constraints = constraints;
        plan.assumptions = assumptions;
        plan.current_stage = PipelineStage::Refinement;
        plan.next_action = "Define architecture summary and core areas".to_string();
        plan.updated_at = Utc::now();
        plan.audit_log.push(PlanAuditEntry {
            timestamp: plan.updated_at,
            actor: actor.to_string(),
            action: "plan.refined".to_string(),
            detail: format!(
                "Plan refined with {} constraints and {} assumptions",
                plan.constraints.len(),
                plan.assumptions.len()
            ),
        });

        self.store
            .save_plan(&plan)
            .await
            .map_err(|e| e.to_string())?;
        Ok(plan)
    }

    // ── Stage 3: Architecture ──

    /// Set the architecture summary and core areas.
    /// Must be in Refinement stage.
    pub async fn set_architecture(
        &self,
        plan_id: &str,
        architecture_summary: String,
        core_areas: Vec<CoreArea>,
        actor: &str,
    ) -> Result<Plan, String> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await
            .map_err(|e| e.to_string())?;

        if plan.current_stage != PipelineStage::Refinement {
            return Err(format!(
                "Plan is in stage '{}' — must be in 'refinement' to set architecture",
                plan.current_stage.as_str()
            ));
        }

        plan.architecture_summary = architecture_summary;
        plan.core_areas = core_areas;
        plan.current_stage = PipelineStage::Architecture;
        plan.next_action = "Decompose core areas into tasks and record decisions".to_string();
        plan.updated_at = Utc::now();
        plan.audit_log.push(PlanAuditEntry {
            timestamp: plan.updated_at,
            actor: actor.to_string(),
            action: "plan.architecture_set".to_string(),
            detail: format!("Architecture set with {} core areas", plan.core_areas.len()),
        });

        self.store
            .save_plan(&plan)
            .await
            .map_err(|e| e.to_string())?;
        Ok(plan)
    }

    // ── Stage 4: Decomposition ──

    /// Record decisions and decompose areas into tasks.
    /// Must be in Architecture stage.
    pub async fn decompose(
        &self,
        plan_id: &str,
        decisions: Vec<PlanDecision>,
        risks: Vec<PlanRisk>,
        phases: Vec<PlanPhase>,
        actor: &str,
    ) -> Result<Plan, String> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await
            .map_err(|e| e.to_string())?;

        if plan.current_stage != PipelineStage::Architecture {
            return Err(format!(
                "Plan is in stage '{}' — must be in 'architecture' to decompose",
                plan.current_stage.as_str()
            ));
        }

        if decisions.is_empty() {
            return Err("At least one decision is required before advancing".to_string());
        }

        plan.decisions = decisions;
        plan.risks = risks;
        plan.phases = phases;
        plan.current_stage = PipelineStage::Decomposition;
        plan.next_action = "Generate work packets from tasks".to_string();
        plan.updated_at = Utc::now();
        plan.audit_log.push(PlanAuditEntry {
            timestamp: plan.updated_at,
            actor: actor.to_string(),
            action: "plan.decomposed".to_string(),
            detail: format!(
                "Decomposition complete: {} decisions, {} risks, {} phases",
                plan.decisions.len(),
                plan.risks.len(),
                plan.phases.len()
            ),
        });

        self.store
            .save_plan(&plan)
            .await
            .map_err(|e| e.to_string())?;
        Ok(plan)
    }

    // ── Stage 5: Packetization ──

    /// Generate work packets from core area tasks.
    /// Must be in Decomposition stage.
    pub async fn generate_packets(
        &self,
        plan_id: &str,
        packets: Vec<WorkPacket>,
        actor: &str,
    ) -> Result<Plan, String> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await
            .map_err(|e| e.to_string())?;

        if plan.current_stage != PipelineStage::Decomposition {
            return Err(format!(
                "Plan is in stage '{}' — must be in 'decomposition' to generate packets",
                plan.current_stage.as_str()
            ));
        }

        if packets.is_empty() {
            return Err("At least one work packet is required".to_string());
        }

        plan.work_packets = packets;
        plan.current_stage = PipelineStage::Packetization;
        plan.next_action = "Review work packets and advance to Ready".to_string();
        plan.updated_at = Utc::now();
        plan.audit_log.push(PlanAuditEntry {
            timestamp: plan.updated_at,
            actor: actor.to_string(),
            action: "plan.packets_generated".to_string(),
            detail: format!("{} work packets generated", plan.work_packets.len()),
        });

        self.store
            .save_plan(&plan)
            .await
            .map_err(|e| e.to_string())?;
        Ok(plan)
    }

    // ── Stage 6: Ready ──

    /// Advance the plan to Ready.
    /// Validates the plan first. Must be in Packetization stage.
    pub async fn advance_to_ready(&self, plan_id: &str, actor: &str) -> Result<Plan, String> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await
            .map_err(|e| e.to_string())?;

        if plan.current_stage != PipelineStage::Packetization {
            return Err(format!(
                "Plan is in stage '{}' — must be in 'packetization' to advance to Ready",
                plan.current_stage.as_str()
            ));
        }

        // Run full validation
        plan.can_advance()
            .map_err(|errors| format!("Validation failed:\n{}", errors.join("\n")))?;

        // Re-check validation specifically for Ready transition
        let check_errors = plan.validate();
        if !check_errors.is_empty() {
            return Err(format!(
                "Cannot advance to Ready:\n{}",
                check_errors.join("\n")
            ));
        }

        plan.status = PlanStatus::Approved;
        plan.current_stage = PipelineStage::Ready;
        plan.next_action = "Execute work packets in dependency order".to_string();
        plan.updated_at = Utc::now();
        plan.audit_log.push(PlanAuditEntry {
            timestamp: plan.updated_at,
            actor: actor.to_string(),
            action: "plan.ready".to_string(),
            detail: "Plan advanced to Ready stage — work packets are executable".to_string(),
        });

        self.store
            .save_plan(&plan)
            .await
            .map_err(|e| e.to_string())?;
        Ok(plan)
    }

    // ── Query ──

    /// List all plans, optionally filtered by stage.
    pub async fn list_plans(&self, stage: Option<&str>) -> Result<Vec<Plan>, String> {
        self.store
            .list_plans(stage)
            .await
            .map_err(|e| e.to_string())
    }

    /// Get a single plan by ID.
    pub async fn get_plan(&self, plan_id: &str) -> Result<Plan, String> {
        self.store
            .get_plan(plan_id)
            .await
            .map_err(|e| e.to_string())
    }

    /// Get audit log for a plan.
    pub async fn get_plan_audit(&self, plan_id: &str) -> Result<Vec<PlanAuditEntry>, String> {
        let plan = self
            .store
            .get_plan(plan_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(plan.audit_log)
    }

    /// Add a stage transition to the plan directly (used by approval flow).
    pub async fn advance_stage(&self, plan_id: &str, actor: &str) -> Result<Plan, String> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await
            .map_err(|e| e.to_string())?;

        let next_stage = plan.current_stage.next().ok_or_else(|| {
            "Plan is already in Ready stage — no further advancement possible".to_string()
        })?;

        // Run validation before advancing
        let check_errors = plan.validate();
        if !check_errors.is_empty() {
            return Err(format!(
                "Cannot advance from '{}' to '{}':\n{}",
                plan.current_stage.as_str(),
                next_stage.as_str(),
                check_errors.join("\n")
            ));
        }

        plan.current_stage = next_stage;
        plan.updated_at = Utc::now();
        plan.audit_log.push(PlanAuditEntry {
            timestamp: plan.updated_at,
            actor: actor.to_string(),
            action: format!("stage.{}", plan.current_stage.as_str()),
            detail: format!("Advanced to '{}' stage", plan.current_stage.as_str()),
        });

        // Update next_action based on new stage
        plan.next_action = match plan.current_stage {
            PipelineStage::Intake => "Refine the plan goal, scope, and constraints".to_string(),
            PipelineStage::Refinement => "Define architecture summary and core areas".to_string(),
            PipelineStage::Architecture => {
                "Decompose core areas into tasks and record decisions".to_string()
            }
            PipelineStage::Decomposition => "Generate work packets from tasks".to_string(),
            PipelineStage::Packetization => "Review work packets and advance to Ready".to_string(),
            PipelineStage::Ready => "Execute work packets in dependency order".to_string(),
        };

        self.store
            .save_plan(&plan)
            .await
            .map_err(|e| e.to_string())?;
        Ok(plan)
    }
}
