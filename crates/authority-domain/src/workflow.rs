//! # Workflow Lifecycle Domain Types
//!
//! Domain types for the focused workflow lifecycle agent system.
//! These types define the stages, artifacts, gates, and context
//! that the `workflow-engine` crate uses to orchestrate work.
//!
//! ## Lifecycle
//!
//! ```text
//! Idea → Planning → [Design] → Architecture → [Council] →
//! Development → Peer Review → Code Review → Testing → Documentation
//! ```

use crate::{DomainError, ProjectId, TenantId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// LifecycleStage
// ---------------------------------------------------------------------------

/// All stages in the focused workflow lifecycle, including extended stages.
///
/// Core stages are always present in every workflow.
/// Extended stages (Design, Council) are optional and configured per workflow.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStage {
    /// Problem statement, user outcome, constraints
    Idea,
    /// Work slice, scope boundary, acceptance criteria
    Planning,
    /// Design document, bounded context, state model (extended)
    Design,
    /// Architecture contract, affected files, risks
    Architecture,
    /// Decision record, positions, resolution (extended)
    Council,
    /// Focused implementation diff
    Development,
    /// Area readiness evidence
    PeerReview,
    /// File review records
    CodeReview,
    /// Test results, acceptance evidence
    Testing,
    /// Updated specs, roadmaps, verification
    Documentation,
}

impl LifecycleStage {
    /// The display name for this stage (used in reports and messages).
    pub fn label(&self) -> &'static str {
        use LifecycleStage::*;
        match self {
            Idea => "Idea",
            Planning => "Planning",
            Design => "Design",
            Architecture => "Architecture",
            Council => "Council",
            Development => "Development",
            PeerReview => "Peer Review",
            CodeReview => "Code Review",
            Testing => "Testing",
            Documentation => "Documentation",
        }
    }

    /// Whether this stage is a core (always-present) stage.
    pub fn is_core(&self) -> bool {
        use LifecycleStage::*;
        matches!(
            self,
            Idea | Planning
                | Architecture
                | Development
                | PeerReview
                | CodeReview
                | Testing
                | Documentation
        )
    }

    /// Whether this stage is an extended (optional) stage.
    pub fn is_extended(&self) -> bool {
        !self.is_core()
    }

    /// The skill names that own this stage.
    pub fn owner_skills(&self) -> &'static [&'static str] {
        use LifecycleStage::*;
        match self {
            Idea => &["biz-user", "pm"],
            Planning => &["pm", "orchestrator"],
            Design => &["domain-architect"],
            Architecture => &["cto"],
            Council => &["council"],
            Development => &["backend", "frontend", "data-engineer"],
            PeerReview => &["peer-review"],
            CodeReview => &["code-review"],
            Testing => &["qa"],
            Documentation => &["tech-writer", "pm", "qa"],
        }
    }

    /// The required artifact description for this stage.
    pub fn required_artifact_description(&self) -> &'static str {
        use LifecycleStage::*;
        match self {
            Idea => "Problem statement, user outcome, constraints, non-goals",
            Planning => "Work slice, scope boundary, acceptance criteria, handoff map",
            Design => "Design document, bounded context, state model",
            Architecture => "Boundary contract, affected crates/files, rollback path, risks",
            Council => "Decision record, positions, resolution",
            Development => "Focused implementation diff within approved files",
            PeerReview => "Area readiness evidence and next-gate recommendation",
            CodeReview => "File review records and area code-review recommendation",
            Testing => "Test results, acceptance evidence, defects or residual risk",
            Documentation => "Updated specs, phase gates, roadmap notes, verification evidence",
        }
    }

    /// The exit gate description for this stage.
    pub fn exit_gate_description(&self) -> &'static str {
        use LifecycleStage::*;
        match self {
            Idea => "The request is clear enough to plan or has named open questions",
            Planning => "The next implementable slice is named and blockers are visible",
            Design => "Design is reviewed and approved by domain-architect",
            Architecture => "Layer law is satisfied and unresolved decisions are recorded",
            Council => "Decision is closed or deferred with documented rationale",
            Development => "Code follows docs-first law and does not expand scope silently",
            PeerReview => "Work is coherent enough to enter file-level code review",
            CodeReview => "Every in-scope file has a review record and no blocking finding remains",
            Testing => "Required gates are run or explicitly blocked",
            Documentation => "Docs match implementation and do not over-certify",
        }
    }

    /// Return the next stage in the default workflow sequence.
    /// Extended stages are included based on the `include_extended` flag.
    pub fn next(&self, include_extended: bool) -> Option<LifecycleStage> {
        use LifecycleStage::*;
        let seq: &[LifecycleStage] = if include_extended {
            &[
                Idea,
                Planning,
                Design,
                Architecture,
                Council,
                Development,
                PeerReview,
                CodeReview,
                Testing,
                Documentation,
            ]
        } else {
            &[
                Idea,
                Planning,
                Architecture,
                Development,
                PeerReview,
                CodeReview,
                Testing,
                Documentation,
            ]
        };
        seq.iter()
            .position(|s| s == self)
            .and_then(|i| seq.get(i + 1))
            .cloned()
    }

    /// Return the previous stage in the default workflow sequence.
    pub fn prev(&self, include_extended: bool) -> Option<LifecycleStage> {
        use LifecycleStage::*;
        let seq: &[LifecycleStage] = if include_extended {
            &[
                Idea,
                Planning,
                Design,
                Architecture,
                Council,
                Development,
                PeerReview,
                CodeReview,
                Testing,
                Documentation,
            ]
        } else {
            &[
                Idea,
                Planning,
                Architecture,
                Development,
                PeerReview,
                CodeReview,
                Testing,
                Documentation,
            ]
        };
        seq.iter().position(|s| s == self).and_then(|i| {
            if i > 0 {
                Some(seq[i - 1].clone())
            } else {
                None
            }
        })
    }
}

impl std::fmt::Display for LifecycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ---------------------------------------------------------------------------
// StageStatus
// ---------------------------------------------------------------------------

/// The status of a stage in a workflow execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    /// Stage has not started yet.
    Pending,
    /// Stage is currently being executed.
    InProgress,
    /// Stage completed successfully.
    Completed,
    /// Stage execution failed — requires intervention.
    Failed,
    /// Stage is blocked on an external decision.
    Blocked,
    /// Stage was explicitly skipped/bypassed with a decision record.
    Bypassed,
}

impl std::fmt::Display for StageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Blocked => write!(f, "blocked"),
            Self::Bypassed => write!(f, "bypassed"),
        }
    }
}

// ---------------------------------------------------------------------------
// StageResult
// ---------------------------------------------------------------------------

/// The result of executing a stage agent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageResult {
    /// The stage that was executed.
    pub stage: LifecycleStage,
    /// Whether the stage passed.
    pub passed: bool,
    /// Detailed findings from stage execution.
    pub findings: Vec<StageFinding>,
    /// Whether the stage is blocked on an external decision.
    pub blocked_on: Option<String>,
    /// Artifacts produced by this stage.
    pub artifacts: Vec<StageArtifact>,
}

impl StageResult {
    /// Create a passing result.
    pub fn passed(stage: LifecycleStage) -> Self {
        Self {
            stage,
            passed: true,
            findings: Vec::new(),
            blocked_on: None,
            artifacts: Vec::new(),
        }
    }

    /// Create a failing result with findings.
    pub fn failed(stage: LifecycleStage, findings: Vec<StageFinding>) -> Self {
        Self {
            stage,
            passed: false,
            findings,
            blocked_on: None,
            artifacts: Vec::new(),
        }
    }

    /// Create a blocked result.
    pub fn blocked(stage: LifecycleStage, decision_id: impl Into<String>) -> Self {
        Self {
            stage,
            passed: false,
            findings: Vec::new(),
            blocked_on: Some(decision_id.into()),
            artifacts: Vec::new(),
        }
    }

    /// Add an artifact to this result.
    pub fn with_artifact(mut self, artifact: StageArtifact) -> Self {
        self.artifacts.push(artifact);
        self
    }
}

// ---------------------------------------------------------------------------
// StageFinding
// ---------------------------------------------------------------------------

/// A single finding from a stage execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageFinding {
    /// Short identifier for this finding (e.g., "MISSING_SCOPE").
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: FindingSeverity,
    /// Optional path to the relevant file or resource.
    pub file_path: Option<String>,
}

impl StageFinding {
    /// Create a new finding.
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        severity: FindingSeverity,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity,
            file_path: None,
        }
    }

    /// Set the file path for this finding.
    pub fn at_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }
}

// ---------------------------------------------------------------------------
// FindingSeverity
// ---------------------------------------------------------------------------

/// Severity of a stage finding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// Informational — no action required.
    Note,
    /// Should be fixed but does not block handoff.
    Warning,
    /// Must be fixed before handoff.
    Error,
    /// Blocks the entire workflow.
    Blocker,
}

impl std::fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Note => write!(f, "note"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Blocker => write!(f, "blocker"),
        }
    }
}

// ---------------------------------------------------------------------------
// GateResult
// ---------------------------------------------------------------------------

/// The result of checking a stage's exit gate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GateResult {
    /// Whether the gate is satisfied.
    pub satisfied: bool,
    /// Individual gate checks.
    pub checks: Vec<GateCheck>,
    /// Summary message.
    pub summary: String,
}

impl GateResult {
    /// Create a passing gate result.
    pub fn satisfied(summary: impl Into<String>) -> Self {
        Self {
            satisfied: true,
            checks: Vec::new(),
            summary: summary.into(),
        }
    }

    /// Create a failing gate result.
    pub fn unsatisfied(checks: Vec<GateCheck>, summary: impl Into<String>) -> Self {
        Self {
            satisfied: false,
            checks,
            summary: summary.into(),
        }
    }
}

/// A single check within an exit gate evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GateCheck {
    /// Short description of the check.
    pub description: String,
    /// Whether this check passed.
    pub passed: bool,
    /// If failed, the reason.
    pub reason: Option<String>,
}

impl GateCheck {
    pub fn passed(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            passed: true,
            reason: None,
        }
    }

    pub fn failed(description: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            passed: false,
            reason: Some(reason.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// StageArtifact
// ---------------------------------------------------------------------------

/// Describes the kind of artifact a stage produces.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    ProblemStatement,
    WorkSlice,
    ScopeBoundary,
    AcceptanceCriteria,
    DesignDocument,
    DecisionRecord,
    ArchitectureContract,
    ImplementationDiff,
    ReviewEvidence,
    FileReviewRecord,
    TestResults,
    DocumentationUpdate,
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProblemStatement => write!(f, "problem_statement"),
            Self::WorkSlice => write!(f, "work_slice"),
            Self::ScopeBoundary => write!(f, "scope_boundary"),
            Self::AcceptanceCriteria => write!(f, "acceptance_criteria"),
            Self::DesignDocument => write!(f, "design_document"),
            Self::DecisionRecord => write!(f, "decision_record"),
            Self::ArchitectureContract => write!(f, "architecture_contract"),
            Self::ImplementationDiff => write!(f, "implementation_diff"),
            Self::ReviewEvidence => write!(f, "review_evidence"),
            Self::FileReviewRecord => write!(f, "file_review_record"),
            Self::TestResults => write!(f, "test_results"),
            Self::DocumentationUpdate => write!(f, "documentation_update"),
        }
    }
}

/// A structured artifact produced by a stage agent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageArtifact {
    pub kind: ArtifactKind,
    pub description: String,
    pub file_paths: Vec<String>,
    pub validated_at: DateTime<Utc>,
    pub validator: String,
}

impl StageArtifact {
    pub fn new(
        kind: ArtifactKind,
        description: impl Into<String>,
        validator: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            description: description.into(),
            file_paths: Vec::new(),
            validated_at: Utc::now(),
            validator: validator.into(),
        }
    }

    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.file_paths.push(path.into());
        self
    }
}

// ---------------------------------------------------------------------------
// WorkflowContext
// ---------------------------------------------------------------------------

/// The shared context for a workflow execution.
///
/// Carries state across all stages in the workflow lifecycle.
/// Each stage agent reads from and writes to this context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Unique workflow instance ID.
    pub workflow_id: String,
    /// Tenant this workflow belongs to.
    pub tenant_id: TenantId,
    /// Project this workflow belongs to.
    pub project_id: ProjectId,
    /// Whether to include extended stages (Design, Council).
    pub include_extended: bool,
    /// Current stage in the workflow.
    pub current_stage: LifecycleStage,
    /// Status of each stage.
    pub stage_statuses: HashMap<LifecycleStage, StageStatus>,
    /// Results from completed stages.
    pub stage_results: HashMap<LifecycleStage, StageResult>,
    /// Artifacts produced across all stages.
    pub artifacts: Vec<StageArtifact>,
    /// Decisions recorded during the workflow (decision_id → summary).
    pub decisions: HashMap<String, String>,
    /// Custom metadata for stage-specific data.
    pub metadata: HashMap<String, String>,
    /// When the workflow was created.
    pub created_at: DateTime<Utc>,
    /// When the workflow was last updated.
    pub updated_at: DateTime<Utc>,
}

impl WorkflowContext {
    /// Create a new workflow context.
    pub fn new(
        workflow_id: impl Into<String>,
        tenant_id: TenantId,
        project_id: ProjectId,
        include_extended: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            workflow_id: workflow_id.into(),
            tenant_id,
            project_id,
            include_extended,
            current_stage: LifecycleStage::Idea,
            stage_statuses: HashMap::new(),
            stage_results: HashMap::new(),
            artifacts: Vec::new(),
            decisions: HashMap::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the status of a specific stage.
    pub fn stage_status(&self, stage: &LifecycleStage) -> StageStatus {
        self.stage_statuses
            .get(stage)
            .cloned()
            .unwrap_or(StageStatus::Pending)
    }

    /// Set the status of a stage.
    pub fn set_stage_status(&mut self, stage: LifecycleStage, status: StageStatus) {
        self.stage_statuses.insert(stage, status);
        self.updated_at = Utc::now();
    }

    /// Record a stage result.
    pub fn record_result(&mut self, result: StageResult) {
        let stage = result.stage.clone();
        self.stage_results.insert(stage.clone(), result);
        self.updated_at = Utc::now();
    }

    /// Add an artifact to the workflow.
    pub fn add_artifact(&mut self, artifact: StageArtifact) {
        self.artifacts.push(artifact);
        self.updated_at = Utc::now();
    }

    /// Record a decision.
    pub fn record_decision(&mut self, decision_id: impl Into<String>, summary: impl Into<String>) {
        self.decisions.insert(decision_id.into(), summary.into());
        self.updated_at = Utc::now();
    }

    /// Advance to the next stage.
    pub fn advance(&mut self) -> Result<(), DomainError> {
        let next = self
            .current_stage
            .next(self.include_extended)
            .ok_or_else(|| {
                DomainError::invalid_state(
                    "already at final stage — no further advancement possible",
                )
            })?;
        self.current_stage = next;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if all core stages are completed.
    pub fn is_complete(&self) -> bool {
        use LifecycleStage::*;
        let core_stages = [
            Idea,
            Planning,
            Architecture,
            Development,
            PeerReview,
            CodeReview,
            Testing,
            Documentation,
        ];
        core_stages
            .iter()
            .all(|s| self.stage_status(s) == StageStatus::Completed)
    }
}

/// Artifact descriptor — metadata about what a stage produces.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactDescriptor {
    pub kind: ArtifactKind,
    pub description: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_stage_core_and_extended() {
        assert!(LifecycleStage::Idea.is_core());
        assert!(!LifecycleStage::Idea.is_extended());
        assert!(LifecycleStage::Design.is_extended());
        assert!(!LifecycleStage::Design.is_core());
        assert!(LifecycleStage::Council.is_extended());
        assert!(LifecycleStage::Documentation.is_core());
    }

    #[test]
    fn lifecycle_stage_display_labels() {
        assert_eq!(LifecycleStage::Idea.label(), "Idea");
        assert_eq!(LifecycleStage::PeerReview.label(), "Peer Review");
        assert_eq!(LifecycleStage::CodeReview.label(), "Code Review");
        assert_eq!(LifecycleStage::Documentation.label(), "Documentation");
    }

    #[test]
    fn stage_sequence_without_extended() {
        let seq = [
            LifecycleStage::Idea,
            LifecycleStage::Planning,
            LifecycleStage::Architecture,
            LifecycleStage::Development,
            LifecycleStage::PeerReview,
            LifecycleStage::CodeReview,
            LifecycleStage::Testing,
            LifecycleStage::Documentation,
        ];

        for i in 0..seq.len() - 1 {
            assert_eq!(
                seq[i].next(false),
                Some(seq[i + 1].clone()),
                "{} should follow {}",
                seq[i + 1].label(),
                seq[i].label()
            );
        }
        assert_eq!(seq[seq.len() - 1].next(false), None);
    }

    #[test]
    fn stage_sequence_with_extended() {
        assert_eq!(
            LifecycleStage::Planning.next(true),
            Some(LifecycleStage::Design)
        );
        assert_eq!(
            LifecycleStage::Design.next(true),
            Some(LifecycleStage::Architecture)
        );
        assert_eq!(
            LifecycleStage::Architecture.next(true),
            Some(LifecycleStage::Council)
        );
        assert_eq!(
            LifecycleStage::Council.next(true),
            Some(LifecycleStage::Development)
        );
    }

    #[test]
    fn prev_stage() {
        assert_eq!(
            LifecycleStage::Planning.prev(false),
            Some(LifecycleStage::Idea)
        );
        assert_eq!(LifecycleStage::Idea.prev(false), None);
        assert_eq!(
            LifecycleStage::Development.prev(true),
            Some(LifecycleStage::Council)
        );
    }

    #[test]
    fn owner_skills_not_empty() {
        for stage in &[
            LifecycleStage::Idea,
            LifecycleStage::Planning,
            LifecycleStage::Design,
            LifecycleStage::Architecture,
            LifecycleStage::Council,
            LifecycleStage::Development,
            LifecycleStage::PeerReview,
            LifecycleStage::CodeReview,
            LifecycleStage::Testing,
            LifecycleStage::Documentation,
        ] {
            assert!(
                !stage.owner_skills().is_empty(),
                "{} must have at least one owner skill",
                stage.label()
            );
        }
    }

    #[test]
    fn stage_result_passed_and_failed() {
        let passed = StageResult::passed(LifecycleStage::Idea);
        assert!(passed.passed);
        assert!(passed.findings.is_empty());

        let findings = vec![StageFinding::new(
            "NO_GOAL",
            "No user outcome",
            FindingSeverity::Error,
        )];
        let failed = StageResult::failed(LifecycleStage::Idea, findings);
        assert!(!failed.passed);
        assert_eq!(failed.findings.len(), 1);
    }

    #[test]
    fn stage_result_blocked() {
        let blocked = StageResult::blocked(LifecycleStage::Architecture, "DEC-COUNCIL-001");
        assert!(!blocked.passed);
        assert_eq!(blocked.blocked_on, Some("DEC-COUNCIL-001".to_string()));
    }

    #[test]
    fn gate_result_checks() {
        let passed = GateResult::satisfied("all checks passed");
        assert!(passed.satisfied);

        let checks = vec![
            GateCheck::passed("spec exists"),
            GateCheck::failed("layer law", "API directly accesses store"),
        ];
        let unsatisfied = GateResult::unsatisfied(checks, "2 checks, 1 failed");
        assert!(!unsatisfied.satisfied);
        assert_eq!(unsatisfied.checks.len(), 2);
    }

    #[test]
    fn workflow_context_creation_and_advance() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        assert_eq!(ctx.current_stage, LifecycleStage::Idea);
        assert_eq!(
            ctx.stage_status(&LifecycleStage::Idea),
            StageStatus::Pending
        );

        ctx.set_stage_status(LifecycleStage::Idea, StageStatus::Completed);
        ctx.record_result(StageResult::passed(LifecycleStage::Idea));
        assert!(ctx.advance().is_ok());
        assert_eq!(ctx.current_stage, LifecycleStage::Planning);

        // Record decision
        ctx.record_decision("DEC-001", "Approved planning scope");
        assert_eq!(
            ctx.decisions.get("DEC-001").unwrap(),
            "Approved planning scope"
        );
    }

    #[test]
    fn workflow_context_is_complete() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        use LifecycleStage::*;
        let stages = [
            Idea,
            Planning,
            Architecture,
            Development,
            PeerReview,
            CodeReview,
            Testing,
            Documentation,
        ];
        for stage in &stages {
            ctx.set_stage_status(stage.clone(), StageStatus::Completed);
        }

        assert!(ctx.is_complete());
    }

    #[test]
    fn workflow_context_not_complete_if_missing_stage() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        ctx.set_stage_status(LifecycleStage::Idea, StageStatus::Completed);
        ctx.set_stage_status(LifecycleStage::Planning, StageStatus::Completed);
        // Architecture is still Pending

        assert!(!ctx.is_complete());
    }

    #[test]
    fn stage_finding_with_path() {
        let finding = StageFinding::new(
            "FILE_TOO_LARGE",
            "File exceeds 500 lines",
            FindingSeverity::Warning,
        )
        .at_path("src/main.rs");
        assert_eq!(finding.file_path, Some("src/main.rs".to_string()));
    }

    #[test]
    fn artifact_kind_display() {
        assert_eq!(
            ArtifactKind::ProblemStatement.to_string(),
            "problem_statement"
        );
        assert_eq!(ArtifactKind::DesignDocument.to_string(), "design_document");
        assert_eq!(ArtifactKind::DecisionRecord.to_string(), "decision_record");
        assert_eq!(ArtifactKind::TestResults.to_string(), "test_results");
    }

    #[test]
    fn stage_artifact_creation() {
        let artifact = StageArtifact::new(
            ArtifactKind::ArchitectureContract,
            "API boundary contract",
            "cto",
        )
        .with_file("docs/architecture/api_boundary.md");
        assert_eq!(artifact.kind, ArtifactKind::ArchitectureContract);
        assert_eq!(artifact.file_paths.len(), 1);
        assert_eq!(artifact.validator, "cto");
    }

    #[test]
    fn stage_status_display() {
        assert_eq!(StageStatus::Pending.to_string(), "pending");
        assert_eq!(StageStatus::InProgress.to_string(), "in_progress");
        assert_eq!(StageStatus::Completed.to_string(), "completed");
        assert_eq!(StageStatus::Bypassed.to_string(), "bypassed");
    }
}
