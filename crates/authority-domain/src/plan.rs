// ─────────────────────────────────────────────
// plan.rs — Complete Plan Schema
// ─────────────────────────────────────────────
// Defines the full structured Plan type for the
// intake pipeline: intent → refinement → architecture
// → decomposition → packetization → ready.
//
// Every field is typed. No placeholders.
// ─────────────────────────────────────────────

use crate::PlanStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

// ── Pipeline Stage Enum ──

/// The six stages of the intake pipeline.
/// Each stage has required inputs, validation rules, and outputs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum PipelineStage {
    #[default]
    Intake,
    Refinement,
    Architecture,
    Decomposition,
    Packetization,
    Ready,
}

impl PipelineStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            PipelineStage::Intake => "intake",
            PipelineStage::Refinement => "refinement",
            PipelineStage::Architecture => "architecture",
            PipelineStage::Decomposition => "decomposition",
            PipelineStage::Packetization => "packetization",
            PipelineStage::Ready => "ready",
        }
    }

    /// Returns the next stage, or None if already at Ready.
    pub fn next(&self) -> Option<PipelineStage> {
        match self {
            PipelineStage::Intake => Some(PipelineStage::Refinement),
            PipelineStage::Refinement => Some(PipelineStage::Architecture),
            PipelineStage::Architecture => Some(PipelineStage::Decomposition),
            PipelineStage::Decomposition => Some(PipelineStage::Packetization),
            PipelineStage::Packetization => Some(PipelineStage::Ready),
            PipelineStage::Ready => None,
        }
    }

    /// Returns the previous stage, or None if already at Intake.
    pub fn prev(&self) -> Option<PipelineStage> {
        match self {
            PipelineStage::Intake => None,
            PipelineStage::Refinement => Some(PipelineStage::Intake),
            PipelineStage::Architecture => Some(PipelineStage::Refinement),
            PipelineStage::Decomposition => Some(PipelineStage::Architecture),
            PipelineStage::Packetization => Some(PipelineStage::Decomposition),
            PipelineStage::Ready => Some(PipelineStage::Packetization),
        }
    }
}

// ── Core Area ──

/// A named area of work within a plan (e.g. "Backend", "Frontend", "Data").
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CoreArea {
    /// Unique name within the plan (e.g. "backend-auth").
    pub id: String,
    /// Human-readable name (e.g. "Backend Authentication").
    pub name: String,
    /// One-sentence description of what this area covers.
    pub description: String,
    /// Tasks assigned to this area.
    pub tasks: Vec<AreaTask>,
    /// The skill responsible (e.g. "backend", "frontend").
    pub owning_skill: String,
}

/// A single task within a CoreArea.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AreaTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub target_file_globs: Vec<String>,
    pub estimated_effort_hours: f64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

// ── Decision ──

/// A recorded architectural or design decision.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct PlanDecision {
    /// Short title (e.g. "Use PostgreSQL for audit store").
    pub title: String,
    /// Full rationale.
    pub rationale: String,
    /// Who made this decision.
    pub decided_by: String,
    /// When it was decided.
    pub decided_at: DateTime<Utc>,
    /// Consequences of this decision (trade-offs).
    pub consequences: Vec<String>,
}

// ── Risk ──

/// A documented risk with mitigation.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct PlanRisk {
    pub description: String,
    pub severity: RiskSeverity,
    pub likelihood: RiskLikelihood,
    pub mitigation: String,
    pub owner: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum RiskLikelihood {
    Unlikely,
    Possible,
    Likely,
    Certain,
}

// ── Phase ──

/// A named phase within the plan (e.g. "Phase 1: Auth", "Phase 2: UI").
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct PlanPhase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: u32,
    pub status: PhaseStatus,
    pub target_completion: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum PhaseStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
}

// ── Work Packet ──

/// A self-contained unit of work that smaller models can execute.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct WorkPacket {
    pub id: String,
    pub title: String,
    pub description: String,
    /// Files this packet is allowed to create/modify.
    pub target_files: Vec<String>,
    /// Additional files that may be read.
    pub allowed_files: Vec<String>,
    /// Files this packet must NOT touch.
    pub forbidden_files: Vec<String>,
    /// IDs of work packets that must complete before this one.
    pub dependencies: Vec<String>,
    /// Input data / context required to start this packet.
    pub inputs: HashMap<String, serde_json::Value>,
    /// What this packet must produce.
    pub expected_output: String,
    /// Validation rules to run after completion.
    pub validation_rules: Vec<String>,
    /// Test requirements.
    pub test_requirements: Vec<String>,
    /// Acceptance criteria as exact statements.
    pub acceptance_criteria: Vec<String>,
    pub status: WorkPacketStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum WorkPacketStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

// ── Audit Entry ──

/// A single audit log entry for the plan.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct PlanAuditEntry {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub detail: String,
}

// ── The Complete Plan ──

/// The top-level Plan type.
/// This replaces the old BoardPlan with a structured,
/// multi-stage pipeline artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
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
    pub status: PlanStatus,
    pub current_stage: PipelineStage,
    pub next_action: String,
    pub owner: String,
    pub audit_log: Vec<PlanAuditEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Validation ──

impl Plan {
    /// Validate that this plan is structurally complete.
    /// Returns a list of all validation failures.
    pub fn validate(&self) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();

        if self.name.trim().is_empty() {
            errors.push("Plan name is required".to_string());
        }
        if self.goal.trim().is_empty() {
            errors.push("Plan goal is required".to_string());
        }
        if self.scope.trim().is_empty() {
            errors.push("Plan scope is required".to_string());
        }
        if self.owner.trim().is_empty() {
            errors.push("Plan must have an owner".to_string());
        }

        // Stage-specific validation
        if self.current_stage as u32 >= PipelineStage::Architecture as u32 {
            if self.architecture_summary.trim().is_empty() {
                errors
                    .push("Architecture summary is required before Architecture stage".to_string());
            }
            if self.core_areas.is_empty() {
                errors.push(
                    "At least one core area is required before Architecture stage".to_string(),
                );
            }
        }

        if self.current_stage as u32 >= PipelineStage::Decomposition as u32 {
            if self.decisions.is_empty() {
                errors.push(
                    "At least one decision must be recorded before Decomposition".to_string(),
                );
            }
            for area in &self.core_areas {
                if area.tasks.is_empty() {
                    errors.push(format!(
                        "Core area '{}' must have at least one task before Decomposition",
                        area.id
                    ));
                }
            }
        }

        if self.current_stage as u32 >= PipelineStage::Packetization as u32 {
            if self.work_packets.is_empty() {
                errors
                    .push("At least one work packet is required before Packetization".to_string());
            }
            for packet in &self.work_packets {
                if packet.target_files.is_empty() {
                    errors.push(format!(
                        "Work packet '{}' must have at least one target file",
                        packet.id
                    ));
                }
                if packet.acceptance_criteria.is_empty() {
                    errors.push(format!(
                        "Work packet '{}' must have at least one acceptance criterion",
                        packet.id
                    ));
                }
            }
        }

        if self.current_stage as u32 >= PipelineStage::Ready as u32 {
            if self.phases.is_empty() {
                errors.push("At least one phase is required before Ready".to_string());
            }
            // Verify all work packets have non-pending status
            let pending_count = self
                .work_packets
                .iter()
                .filter(|p| p.status == WorkPacketStatus::Pending)
                .count();
            if pending_count > 0 {
                errors.push(format!(
                    "{} work packet(s) are still Pending. All must be resolved before Ready.",
                    pending_count
                ));
            }
        }

        errors
    }

    /// Check if the plan can advance to the next stage.
    pub fn can_advance(&self) -> Result<(), Vec<String>> {
        let errors = self.validate();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ── Re-export helpers ──
