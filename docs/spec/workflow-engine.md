# Workflow Engine Specification

**Crate:** `workflow-engine`  
**Status:** Phase 0a — Specification Draft  
**Owner:** Backend Engineering (Dmitri Volkov)  
**Architects:** Domain (Yusuf), API (Marcus), Data (Chen)  
**Date:** 2025-01-30  
**Version:** 0.1.0

---

## 1. Overview

### 1.1 Purpose

The Workflow Engine provides **project lifecycle management** for the RealmForge 9-phase workflow system. It:

* Manages project entities (creation, metadata, status tracking)
* Enforces the 9-phase state machine (Phases 0-9)
* Validates phase transitions against quality gates
* Tracks deliverables per phase
* Provides query interface for project status

**Not Responsible For:**
* Gate approval logic (delegated to `gate-system` crate)
* Artifact storage (delegated to `artifact-store` crate)
* Team assignment (delegated to `control-store` team assignments module)
* Task orchestration (delegated to `control-service` task module)

### 1.2 Position in Architecture

```
agent-mcp (MCP tools: workflow_*)
    ↓
workflow-engine (service layer)
    ↓
control-store (persistence: projects, workflow_phases tables)
    ↓
PostgreSQL
```

**Dependencies:**
* `authority-domain` — ActorId, SessionToken
* `control-store` — Database persistence
* `gate-system` — Gate validation (phase transition checks)

**Dependents:**
* `agent-mcp` — MCP tools for agent interaction
* `control-api` — REST endpoints (future)

### 1.3 The 9-Phase Workflow

| Phase | Name | Description | Gate Required |
|-------|------|-------------|---------------|
| 0 | Intake & Requirements | User prompt, requirements gathering | None → Phase 1 requires user approval |
| 1 | Design & Mockup | Design artifacts, UX review | User approval required |
| 2 | Technical Discovery | Data audit, integration assessment | None |
| 3 | Architecture & Design | ADRs, architecture diagrams | Council approval required |
| 4 | Financial Review | Cost estimation, budget | CEO approval required |
| 5 | Detailed Planning | Work breakdown, task assignment | None |
| 6 | Quality & Security Planning | Test plans, threat models | Security approval required |
| 7 | Implementation | Code, tests, documentation | None → Phase 8 requires tests passing |
| 8 | Release | Deployment, smoke tests | User sign-off required |
| 9 | Post-Launch | Monitoring, bugs, feedback | None (continuous) |

---

## 2. Domain Model

### 2.1 Core Types

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for a project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(Uuid);

impl ProjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }
    
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

/// Project status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    /// Project is actively being worked on
    Active,
    /// Project is temporarily paused
    Paused,
    /// Project completed successfully
    Completed,
    /// Project was cancelled
    Cancelled,
}

/// Workflow phase number (0-9)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PhaseNumber(u8);

impl PhaseNumber {
    /// Create a phase number (0-9)
    pub fn new(phase: u8) -> Result<Self, InvalidPhaseError> {
        if phase > 9 {
            return Err(InvalidPhaseError(phase));
        }
        Ok(Self(phase))
    }
    
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    
    /// Get the next phase number, or None if already at Phase 9
    pub fn next(&self) -> Option<Self> {
        if self.0 < 9 {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }
    
    /// Get the previous phase number, or None if at Phase 0
    pub fn previous(&self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self(self.0 - 1))
        } else {
            None
        }
    }
}

/// Phase status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    /// Phase has not started yet
    NotStarted,
    /// Phase is currently in progress
    InProgress,
    /// Phase completed successfully
    Completed,
    /// Phase is blocked (waiting on gate or dependency)
    Blocked,
}

/// Application type identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationTypeId(String);

impl ApplicationTypeId {
    /// Create from string (e.g., "01_static_website")
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Phase deliverables (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDeliverables {
    /// Artifact IDs produced in this phase
    pub artifacts: Vec<Uuid>,
    /// Notes or summary
    pub notes: Option<String>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Complete project entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub application_type_id: ApplicationTypeId,
    pub current_phase: PhaseNumber,
    pub status: ProjectStatus,
    pub owner_actor_id: Uuid, // ActorId from authority-domain
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Workflow phase entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPhase {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub phase_number: PhaseNumber,
    pub status: PhaseStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub blocked_reason: Option<String>,
    pub deliverables: Option<PhaseDeliverables>,
}
```

### 2.2 Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Invalid phase number: {0} (must be 0-9)")]
    InvalidPhase(u8),
    
    #[error("Project not found: {0}")]
    ProjectNotFound(ProjectId),
    
    #[error("Phase transition blocked: {reason}")]
    PhaseTransitionBlocked { reason: String },
    
    #[error("Gate requirements not met for phase {phase}: {gates:?}")]
    GatesNotMet { phase: PhaseNumber, gates: Vec<String> },
    
    #[error("Project status prevents operation: {status:?}")]
    InvalidStatus { status: ProjectStatus },
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Authorization error: {0}")]
    Unauthorized(String),
}

#[derive(Debug, Error)]
#[error("Invalid phase number: {0} (must be 0-9)")]
pub struct InvalidPhaseError(u8);
```

---

## 3. Service Layer (Public API)

### 3.1 WorkflowService

```rust
use authority_domain::ActorId;

/// Service for project lifecycle management
pub struct WorkflowService {
    store: Arc<dyn WorkflowStore>,
    gate_validator: Arc<dyn GateValidator>,
}

impl WorkflowService {
    /// Create a new project
    pub async fn create_project(
        &self,
        actor_id: ActorId,
        name: String,
        description: Option<String>,
        application_type_id: ApplicationTypeId,
    ) -> Result<Project, WorkflowError> {
        // Validate application type exists
        // Create project entity
        // Initialize Phase 0 as InProgress
        // Initialize Phases 1-9 as NotStarted
        // Store in database
        // Return project
    }
    
    /// Get project by ID
    pub async fn get_project(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
    ) -> Result<Project, WorkflowError> {
        // Check read permission
        // Query database
    }
    
    /// List projects (with filters)
    pub async fn list_projects(
        &self,
        actor_id: ActorId,
        status: Option<ProjectStatus>,
        owner_actor_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Project>, WorkflowError> {
        // Check list permission
        // Query database with filters
    }
    
    /// Update project metadata
    pub async fn update_project(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        name: Option<String>,
        description: Option<String>,
        status: Option<ProjectStatus>,
    ) -> Result<Project, WorkflowError> {
        // Check update permission
        // Update fields
        // Store
    }
    
    /// Advance project to next phase (with gate validation)
    pub async fn advance_phase(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        force: bool,
    ) -> Result<WorkflowPhase, WorkflowError> {
        // Get project
        // Check if project status is Active
        // Get current phase
        // Check if current phase is Completed
        // Determine next phase
        // If Phase 9, return error (cannot advance from Phase 9)
        // If not force, validate gates for next phase
        // If gates fail, return GatesNotMet error
        // Mark current phase as Completed
        // Mark next phase as InProgress
        // Update project.current_phase
        // Store changes
        // Return new phase
    }
    
    /// Block a phase (with reason)
    pub async fn block_phase(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        phase_number: PhaseNumber,
        reason: String,
    ) -> Result<WorkflowPhase, WorkflowError> {
        // Check permission
        // Get phase
        // Set status to Blocked
        // Set blocked_reason
        // Store
    }
    
    /// Unblock a phase
    pub async fn unblock_phase(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        phase_number: PhaseNumber,
    ) -> Result<WorkflowPhase, WorkflowError> {
        // Check permission
        // Get phase
        // Set status to InProgress or NotStarted (based on previous state)
        // Clear blocked_reason
        // Store
    }
    
    /// Update phase deliverables
    pub async fn update_phase_deliverables(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        phase_number: PhaseNumber,
        deliverables: PhaseDeliverables,
    ) -> Result<WorkflowPhase, WorkflowError> {
        // Check permission
        // Get phase
        // Update deliverables
        // Store
    }
    
    /// Get all phases for a project
    pub async fn get_project_phases(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
    ) -> Result<Vec<WorkflowPhase>, WorkflowError> {
        // Check permission
        // Query all phases for project
    }
    
    /// Get specific phase status
    pub async fn get_phase(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        phase_number: PhaseNumber,
    ) -> Result<WorkflowPhase, WorkflowError> {
        // Check permission
        // Query phase
    }
}
```

### 3.2 WorkflowStore Trait

```rust
/// Storage interface for workflow data
#[async_trait]
pub trait WorkflowStore: Send + Sync {
    async fn create_project(&self, project: &Project) -> Result<(), WorkflowError>;
    async fn get_project(&self, id: ProjectId) -> Result<Option<Project>, WorkflowError>;
    async fn update_project(&self, project: &Project) -> Result<(), WorkflowError>;
    async fn list_projects(
        &self,
        status: Option<ProjectStatus>,
        owner_actor_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Project>, WorkflowError>;
    
    async fn create_phase(&self, phase: &WorkflowPhase) -> Result<(), WorkflowError>;
    async fn get_phase(
        &self,
        project_id: ProjectId,
        phase_number: PhaseNumber,
    ) -> Result<Option<WorkflowPhase>, WorkflowError>;
    async fn update_phase(&self, phase: &WorkflowPhase) -> Result<(), WorkflowError>;
    async fn list_phases(&self, project_id: ProjectId) -> Result<Vec<WorkflowPhase>, WorkflowError>;
}
```

### 3.3 GateValidator Trait

```rust
/// Interface to gate-system for validation
#[async_trait]
pub trait GateValidator: Send + Sync {
    /// Check if all gates are passed for a given project phase
    /// Returns Ok(()) if all gates passed
    /// Returns Err with list of failed gate IDs if any gates pending/rejected
    async fn validate_phase_gates(
        &self,
        project_id: ProjectId,
        phase_number: PhaseNumber,
    ) -> Result<(), Vec<String>>;
}
```

---

## 4. Database Schema

### 4.1 Migration: `0010_workflow_engine.sql`

```sql
-- projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    application_type_id TEXT NOT NULL,
    current_phase SMALLINT NOT NULL DEFAULT 0 CHECK (current_phase >= 0 AND current_phase <= 9),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'completed', 'cancelled')),
    owner_actor_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for listing projects by owner
CREATE INDEX idx_projects_owner ON projects(owner_actor_id);

-- Index for filtering by status
CREATE INDEX idx_projects_status ON projects(status);

-- Index for sorting by creation date
CREATE INDEX idx_projects_created_at ON projects(created_at DESC);

-- workflow_phases table
CREATE TABLE workflow_phases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phase_number SMALLINT NOT NULL CHECK (phase_number >= 0 AND phase_number <= 9),
    status TEXT NOT NULL DEFAULT 'not_started' CHECK (status IN ('not_started', 'in_progress', 'completed', 'blocked')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    blocked_reason TEXT,
    deliverables JSONB,
    UNIQUE(project_id, phase_number)
);

-- Index for querying phases by project
CREATE INDEX idx_workflow_phases_project ON workflow_phases(project_id, phase_number);

-- Index for finding blocked phases
CREATE INDEX idx_workflow_phases_blocked ON workflow_phases(status) WHERE status = 'blocked';

-- Trigger to update projects.updated_at on any change
CREATE OR REPLACE FUNCTION update_projects_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER projects_updated_at_trigger
BEFORE UPDATE ON projects
FOR EACH ROW
EXECUTE FUNCTION update_projects_updated_at();

-- Seed phase records when project is created
CREATE OR REPLACE FUNCTION seed_workflow_phases()
RETURNS TRIGGER AS $$
DECLARE
    i INTEGER;
BEGIN
    -- Create 10 phases (0-9) for the new project
    FOR i IN 0..9 LOOP
        INSERT INTO workflow_phases (project_id, phase_number, status)
        VALUES (
            NEW.id,
            i,
            CASE WHEN i = 0 THEN 'in_progress' ELSE 'not_started' END
        );
    END LOOP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER seed_phases_on_project_create
AFTER INSERT ON projects
FOR EACH ROW
EXECUTE FUNCTION seed_workflow_phases();
```

### 4.2 Schema Notes

* **Foreign Keys:** `projects.application_type_id` references `application_types.id` (from `control-store`, to be added in Phase 0c)
* **Cascade Deletion:** Deleting a project deletes all its phases
* **Automatic Seeding:** When a project is created, all 10 phases (0-9) are automatically inserted (Phase 0 as `in_progress`, others as `not_started`)
* **Updated Timestamp:** `projects.updated_at` automatically updates on any change

---

## 5. MCP Tools

### 5.1 Tool Definitions

```rust
// In agent-mcp/src/tool_definitions.rs

pub fn workflow_create_project_tool() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_create_project".to_string(),
        description: "Create a new project in the workflow system".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Project name"
                },
                "description": {
                    "type": "string",
                    "description": "Optional project description"
                },
                "application_type_id": {
                    "type": "string",
                    "description": "Application type ID (e.g., '01_static_website')"
                }
            },
            "required": ["name", "application_type_id"]
        }),
    }
}

pub fn workflow_get_project_tool() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_get_project".to_string(),
        description: "Get project details by ID".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {
                    "type": "string",
                    "description": "Project UUID"
                }
            },
            "required": ["project_id"]
        }),
    }
}

pub fn workflow_list_projects_tool() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_list_projects".to_string(),
        description: "List projects with optional filters".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["active", "paused", "completed", "cancelled"],
                    "description": "Filter by project status"
                },
                "owner_actor_id": {
                    "type": "string",
                    "description": "Filter by owner actor UUID"
                },
                "limit": {
                    "type": "number",
                    "default": 50,
                    "description": "Maximum number of results"
                },
                "offset": {
                    "type": "number",
                    "default": 0,
                    "description": "Results offset for pagination"
                }
            }
        }),
    }
}

pub fn workflow_advance_phase_tool() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_advance_phase".to_string(),
        description: "Advance project to next phase (validates gates unless force=true)".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {
                    "type": "string",
                    "description": "Project UUID"
                },
                "force": {
                    "type": "boolean",
                    "default": false,
                    "description": "Skip gate validation (use with caution)"
                }
            },
            "required": ["project_id"]
        }),
    }
}

pub fn workflow_block_phase_tool() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_block_phase".to_string(),
        description: "Mark a phase as blocked with reason".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {
                    "type": "string",
                    "description": "Project UUID"
                },
                "phase_number": {
                    "type": "number",
                    "description": "Phase number (0-9)"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for blocking"
                }
            },
            "required": ["project_id", "phase_number", "reason"]
        }),
    }
}

pub fn workflow_unblock_phase_tool() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_unblock_phase".to_string(),
        description: "Remove block from a phase".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {
                    "type": "string",
                    "description": "Project UUID"
                },
                "phase_number": {
                    "type": "number",
                    "description": "Phase number (0-9)"
                }
            },
            "required": ["project_id", "phase_number"]
        }),
    }
}

pub fn workflow_update_deliverables_tool() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_update_deliverables".to_string(),
        description: "Update phase deliverables (artifacts, notes)".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {
                    "type": "string",
                    "description": "Project UUID"
                },
                "phase_number": {
                    "type": "number",
                    "description": "Phase number (0-9)"
                },
                "artifact_ids": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of artifact UUIDs produced in this phase"
                },
                "notes": {
                    "type": "string",
                    "description": "Summary notes for this phase"
                }
            },
            "required": ["project_id", "phase_number"]
        }),
    }
}

pub fn workflow_get_phases_tool() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_get_phases".to_string(),
        description: "Get all phases for a project".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {
                    "type": "string",
                    "description": "Project UUID"
                }
            },
            "required": ["project_id"]
        }),
    }
}
```

---

## 6. Acceptance Tests

### 6.1 Project Creation

```rust
#[tokio::test]
async fn test_create_project() {
    // Given: A valid actor and application type
    let service = setup_test_service().await;
    let actor_id = create_test_actor();
    
    // When: Create project
    let project = service.create_project(
        actor_id,
        "Test Project".to_string(),
        Some("Description".to_string()),
        ApplicationTypeId::new("01_static_website"),
    ).await.unwrap();
    
    // Then: Project created with Phase 0 in progress
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.current_phase.as_u8(), 0);
    assert_eq!(project.status, ProjectStatus::Active);
    
    // And: All 10 phases seeded
    let phases = service.get_project_phases(actor_id, project.id).await.unwrap();
    assert_eq!(phases.len(), 10);
    assert_eq!(phases[0].status, PhaseStatus::InProgress);
    assert_eq!(phases[1].status, PhaseStatus::NotStarted);
}
```

### 6.2 Phase Advancement (Success)

```rust
#[tokio::test]
async fn test_advance_phase_success() {
    // Given: Project in Phase 0 with all gates passed
    let service = setup_test_service().await;
    let actor_id = create_test_actor();
    let project = create_test_project(&service, actor_id).await;
    
    // Mock gate validator to return success
    mock_gates_passed(&service, project.id, PhaseNumber::new(1).unwrap());
    
    // When: Advance phase
    let new_phase = service.advance_phase(actor_id, project.id, false).await.unwrap();
    
    // Then: Project now in Phase 1
    let updated_project = service.get_project(actor_id, project.id).await.unwrap();
    assert_eq!(updated_project.current_phase.as_u8(), 1);
    assert_eq!(new_phase.phase_number.as_u8(), 1);
    assert_eq!(new_phase.status, PhaseStatus::InProgress);
    
    // And: Phase 0 marked completed
    let phase0 = service.get_phase(actor_id, project.id, PhaseNumber::new(0).unwrap()).await.unwrap();
    assert_eq!(phase0.status, PhaseStatus::Completed);
    assert!(phase0.completed_at.is_some());
}
```

### 6.3 Phase Advancement (Gates Blocked)

```rust
#[tokio::test]
async fn test_advance_phase_gates_blocked() {
    // Given: Project in Phase 0 with gates NOT passed
    let service = setup_test_service().await;
    let actor_id = create_test_actor();
    let project = create_test_project(&service, actor_id).await;
    
    // Mock gate validator to return failure
    mock_gates_failed(&service, project.id, PhaseNumber::new(1).unwrap(), vec!["user_approval".to_string()]);
    
    // When: Attempt to advance phase
    let result = service.advance_phase(actor_id, project.id, false).await;
    
    // Then: Fails with GatesNotMet error
    assert!(matches!(result, Err(WorkflowError::GatesNotMet { .. })));
    
    // And: Project still in Phase 0
    let project = service.get_project(actor_id, project.id).await.unwrap();
    assert_eq!(project.current_phase.as_u8(), 0);
}
```

### 6.4 Force Advance (Skip Gates)

```rust
#[tokio::test]
async fn test_advance_phase_force() {
    // Given: Project with gates not passed
    let service = setup_test_service().await;
    let actor_id = create_test_actor();
    let project = create_test_project(&service, actor_id).await;
    mock_gates_failed(&service, project.id, PhaseNumber::new(1).unwrap(), vec!["user_approval".to_string()]);
    
    // When: Force advance (skip gates)
    let new_phase = service.advance_phase(actor_id, project.id, true).await.unwrap();
    
    // Then: Advancement succeeds despite failed gates
    assert_eq!(new_phase.phase_number.as_u8(), 1);
    let project = service.get_project(actor_id, project.id).await.unwrap();
    assert_eq!(project.current_phase.as_u8(), 1);
}
```

### 6.5 Block/Unblock Phase

```rust
#[tokio::test]
async fn test_block_unblock_phase() {
    // Given: Project in Phase 2
    let service = setup_test_service().await;
    let actor_id = create_test_actor();
    let project = create_test_project(&service, actor_id).await;
    advance_to_phase(&service, actor_id, project.id, 2).await;
    
    // When: Block Phase 2
    let blocked_phase = service.block_phase(
        actor_id,
        project.id,
        PhaseNumber::new(2).unwrap(),
        "Waiting on external API access".to_string(),
    ).await.unwrap();
    
    // Then: Phase marked as blocked
    assert_eq!(blocked_phase.status, PhaseStatus::Blocked);
    assert_eq!(blocked_phase.blocked_reason.as_ref().unwrap(), "Waiting on external API access");
    
    // When: Unblock Phase 2
    let unblocked_phase = service.unblock_phase(
        actor_id,
        project.id,
        PhaseNumber::new(2).unwrap(),
    ).await.unwrap();
    
    // Then: Phase marked as in progress
    assert_eq!(unblocked_phase.status, PhaseStatus::InProgress);
    assert!(unblocked_phase.blocked_reason.is_none());
}
```

### 6.6 Update Deliverables

```rust
#[tokio::test]
async fn test_update_deliverables() {
    // Given: Project in Phase 0
    let service = setup_test_service().await;
    let actor_id = create_test_actor();
    let project = create_test_project(&service, actor_id).await;
    
    let artifact_id = Uuid::new_v4();
    
    // When: Update Phase 0 deliverables
    let deliverables = PhaseDeliverables {
        artifacts: vec![artifact_id],
        notes: Some("Requirements document completed".to_string()),
        completed_at: Some(Utc::now()),
    };
    
    let updated_phase = service.update_phase_deliverables(
        actor_id,
        project.id,
        PhaseNumber::new(0).unwrap(),
        deliverables.clone(),
    ).await.unwrap();
    
    // Then: Deliverables stored
    assert_eq!(updated_phase.deliverables.as_ref().unwrap().artifacts, vec![artifact_id]);
    assert_eq!(updated_phase.deliverables.as_ref().unwrap().notes, Some("Requirements document completed".to_string()));
}
```

---

## 7. Open Decisions

### 7.1 USER_APPROVAL_REQUIRED

**Decision:** Should `force` parameter on `advance_phase` be available to agents, or restricted to operators only?

**Options:**
1. Available to agents (with audit trail)
2. Restricted to operator CLI only
3. Require special skill grant (e.g., `workflow:force_advance`)

**Recommendation:** Option 3 — Require special skill grant. This balances flexibility with governance.

**Status:** PENDING USER APPROVAL

---

### 7.2 COUNCIL_DECISION_REQUIRED

**Decision:** Should projects support "rollback to previous phase" or is workflow strictly forward-only?

**Options:**
1. Forward-only (current design) — simpler state machine
2. Allow rollback — adds complexity but enables "redo" scenarios

**Considerations:**
* Rollback would need to handle artifact cleanup
* Rollback would need to invalidate completed gates
* Current audit-log + snapshot system already provides historical view

**Recommendation:** Forward-only for Phase 0a. Rollback can be added later if user demand exists.

**Status:** PENDING COUNCIL REVIEW

---

## 8. Implementation Notes

### 8.1 Crate Structure

```
crates/workflow-engine/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Module coordinator (pub use only)
│   ├── domain.rs           # Domain types (ProjectId, PhaseNumber, etc.)
│   ├── service.rs          # WorkflowService implementation
│   ├── store.rs            # WorkflowStore trait
│   ├── gate_validator.rs   # GateValidator trait
│   └── error.rs            # Error types
└── tests/
    └── integration.rs      # Acceptance tests
```

### 8.2 Dependencies

```toml
[dependencies]
authority-domain = { path = "../authority-domain" }
control-store = { path = "../control-store" }
gate-system = { path = "../gate-system" }

uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
async-trait = "0.1"
```

### 8.3 Layer Boundaries

* **Domain:** Pure logic, no IO (domain.rs, error.rs)
* **Service:** Business logic, orchestration (service.rs)
* **Store:** Persistence interface (store.rs, gate_validator.rs)
* **Store Implementation:** Lives in `control-store` crate (PostgreSQL)

---

## 9. Handoff Protocol

### 9.1 From PM (Alex)

**Received:**
* This spec document
* User requirements from gap analysis

**Needed Before Implementation:**
* CTO (Rena) architecture review ✅ (layer boundaries confirmed)
* Data Architect (Chen) schema review ⏳ (next step)
* API Architect (Marcus) MCP tool review ⏳ (next step)
* Domain Architect (Yusuf) domain model review ⏳ (next step)

### 9.2 To Data Architect (Chen)

**Please Review:**
1. Database schema (Section 4)
2. Indexes (sufficient for query patterns?)
3. Triggers (seed phases on project create)
4. Foreign key references to `application_types` table

**Questions:**
* Should `application_type_id` be a foreign key or a string reference?
* Any concerns with CASCADE DELETE on workflow_phases?

### 9.3 To API Architect (Marcus)

**Please Review:**
1. MCP tool definitions (Section 5)
2. Error responses for tool handlers
3. Naming consistency with existing tools

### 9.4 To Domain Architect (Yusuf)

**Please Review:**
1. Domain types (Section 2.1)
2. PhaseNumber newtype with validation
3. State transitions (PhaseStatus enum)

**Questions:**
* Should `ProjectId` be a newtype or use raw Uuid?
* Any missing domain invariants?

---

## 10. Next Steps

1. **Chen (Data Architect):** Review database schema → Approve or request changes
2. **Marcus (API Architect):** Review MCP tools → Approve or request changes
3. **Yusuf (Domain Architect):** Review domain model → Approve or request changes
4. **Council:** Review open decisions (force advancement, rollback support)
5. **Dmitri (Backend):** Implement crate after approvals
6. **Meg (QA):** Write integration tests based on acceptance criteria

---

**END OF SPECIFICATION**
