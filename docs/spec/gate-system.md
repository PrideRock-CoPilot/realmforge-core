# Gate System Specification

**Crate:** `gate-system`  
**Status:** Phase 0a — Specification Draft  
**Owner:** Backend Engineering (Dmitri Volkov)  
**Architects:** Security (Fatima), API (Marcus), Data (Chen)  
**Date:** 2025-01-30  
**Version:** 0.1.0

---

## 1. Overview

### 1.1 Purpose

The Gate System provides **approval workflow management** for quality gates in the RealmForge 9-phase workflow. It:

* Defines quality gates (user approval, council approval, CEO approval, security approval, test approval)
* Manages approval requests and responses
* Tracks approval status (pending, approved, rejected)
* Blocks phase transitions until required approvals obtained
* Provides audit trail of all approval decisions

**Not Responsible For:**
* Project lifecycle management (delegated to `workflow-engine`)
* Artifact storage (delegated to `artifact-store`)
* Notification delivery (future: notification system)

### 1.2 Position in Architecture

```
agent-mcp (MCP tools: gate_*)
    ↓
gate-system (service layer)
    ↓
control-store (persistence: gates, gate_approvals tables)
    ↓
PostgreSQL
```

**Dependencies:**
* `authority-domain` — ActorId, SessionToken
* `control-store` — Database persistence

**Dependents:**
* `workflow-engine` — Gate validation for phase transitions
* `agent-mcp` — MCP tools for approval workflow
* `control-api` — REST endpoints (future)

### 1.3 Quality Gates

| Gate Type | Phase Protects | Required Approver | Description |
|-----------|----------------|-------------------|-------------|
| `user_approval` | 0→1, 8→Production | User/Stakeholder | User sign-off on requirements or UAT |
| `council_approval` | 3→4 | Decision Council | Architecture approval |
| `ceo_approval` | 4→5 | CEO (Victor) | Budget approval |
| `security_approval` | 6→7 | Security Architect (Fatima) | Security review sign-off |
| `test_approval` | 7→8 | QA (Meg) | All tests passing |

---

## 2. Domain Model

### 2.1 Core Types

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use authority_domain::ActorId;

/// Unique identifier for a gate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GateId(Uuid);

impl GateId {
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

/// Gate type (which approval is required)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateType {
    /// User/stakeholder approval
    UserApproval,
    /// Council (multi-stakeholder) approval
    CouncilApproval,
    /// CEO budget approval
    CeoApproval,
    /// Security architect sign-off
    SecurityApproval,
    /// All tests passing
    TestApproval,
}

impl GateType {
    /// Get the required approver role for this gate type
    pub fn required_approver_role(&self) -> &str {
        match self {
            GateType::UserApproval => "user",
            GateType::CouncilApproval => "council",
            GateType::CeoApproval => "ceo",
            GateType::SecurityApproval => "security_architect",
            GateType::TestApproval => "qa",
        }
    }
}

/// Gate status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    /// Approval pending
    Pending,
    /// Approved (gate passed)
    Approved,
    /// Rejected (gate failed)
    Rejected,
}

/// Approval decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Approve,
    Reject,
}

/// Gate entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    pub id: GateId,
    pub project_id: Uuid, // ProjectId from workflow-engine
    pub phase_number: u8, // Which phase this gate protects (0-9)
    pub gate_type: GateType,
    pub status: GateStatus,
    pub required_approver_role: String,
    pub requested_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Gate approval entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateApproval {
    pub id: Uuid,
    pub gate_id: GateId,
    pub approver_actor_id: Uuid, // ActorId from authority-domain
    pub decision: ApprovalDecision,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Unique identifier for an approval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApprovalId(Uuid);

impl ApprovalId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### 2.2 Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GateError {
    #[error("Gate not found: {0}")]
    GateNotFound(GateId),
    
    #[error("Gate already resolved: {gate_id} (status: {status:?})")]
    AlreadyResolved { gate_id: GateId, status: GateStatus },
    
    #[error("Unauthorized: actor {actor_id} does not have role {required_role}")]
    Unauthorized { actor_id: Uuid, required_role: String },
    
    #[error("Invalid phase number: {0} (must be 0-9)")]
    InvalidPhase(u8),
    
    #[error("Gate already exists: project {project_id}, phase {phase_number}, type {gate_type:?}")]
    GateAlreadyExists {
        project_id: Uuid,
        phase_number: u8,
        gate_type: GateType,
    },
    
    #[error("Database error: {0}")]
    Database(String),
}
```

---

## 3. Service Layer (Public API)

### 3.1 GateService

```rust
/// Service for gate management and approval workflow
pub struct GateService {
    store: Arc<dyn GateStore>,
    role_checker: Arc<dyn RoleChecker>,
}

impl GateService {
    /// Create a new gate for a project phase
    pub async fn create_gate(
        &self,
        actor_id: ActorId,
        project_id: Uuid,
        phase_number: u8,
        gate_type: GateType,
    ) -> Result<Gate, GateError> {
        // Validate phase number (0-9)
        if phase_number > 9 {
            return Err(GateError::InvalidPhase(phase_number));
        }
        
        // Check if gate already exists (unique constraint)
        if self.store.get_gate_by_project_phase_type(project_id, phase_number, gate_type).await?.is_some() {
            return Err(GateError::GateAlreadyExists { project_id, phase_number, gate_type });
        }
        
        // Create gate
        let gate = Gate {
            id: GateId::new(),
            project_id,
            phase_number,
            gate_type,
            status: GateStatus::Pending,
            required_approver_role: gate_type.required_approver_role().to_string(),
            requested_at: Utc::now(),
            resolved_at: None,
        };
        
        self.store.create_gate(&gate).await?;
        Ok(gate)
    }
    
    /// Submit an approval decision (approve or reject)
    pub async fn submit_approval(
        &self,
        actor_id: ActorId,
        gate_id: GateId,
        decision: ApprovalDecision,
        reason: Option<String>,
    ) -> Result<GateApproval, GateError> {
        // Get gate
        let mut gate = self.store.get_gate(gate_id).await?
            .ok_or(GateError::GateNotFound(gate_id))?;
        
        // Check if gate already resolved
        if gate.status != GateStatus::Pending {
            return Err(GateError::AlreadyResolved {
                gate_id,
                status: gate.status,
            });
        }
        
        // Check if actor has required role
        if !self.role_checker.has_role(actor_id, &gate.required_approver_role).await? {
            return Err(GateError::Unauthorized {
                actor_id: actor_id.as_uuid(),
                required_role: gate.required_approver_role.clone(),
            });
        }
        
        // Create approval record
        let approval = GateApproval {
            id: Uuid::new_v4(),
            gate_id,
            approver_actor_id: actor_id.as_uuid(),
            decision,
            reason,
            timestamp: Utc::now(),
        };
        
        self.store.create_approval(&approval).await?;
        
        // Update gate status
        gate.status = match decision {
            ApprovalDecision::Approve => GateStatus::Approved,
            ApprovalDecision::Reject => GateStatus::Rejected,
        };
        gate.resolved_at = Some(Utc::now());
        
        self.store.update_gate(&gate).await?;
        
        Ok(approval)
    }
    
    /// Get pending gates for an approver
    pub async fn get_pending_gates(
        &self,
        actor_id: ActorId,
    ) -> Result<Vec<Gate>, GateError> {
        // Get actor's roles
        let roles = self.role_checker.get_actor_roles(actor_id).await?;
        
        // Query pending gates matching any of actor's roles
        self.store.list_pending_gates_by_roles(&roles).await
    }
    
    /// Get gate by ID
    pub async fn get_gate(
        &self,
        actor_id: ActorId,
        gate_id: GateId,
    ) -> Result<Gate, GateError> {
        self.store.get_gate(gate_id).await?
            .ok_or(GateError::GateNotFound(gate_id))
    }
    
    /// List all gates for a project
    pub async fn list_project_gates(
        &self,
        actor_id: ActorId,
        project_id: Uuid,
    ) -> Result<Vec<Gate>, GateError> {
        self.store.list_gates_by_project(project_id).await
    }
    
    /// Check if all gates are passed for a project phase
    /// Returns Ok(()) if all gates passed
    /// Returns Err with list of failed gate IDs if any gates pending/rejected
    pub async fn validate_phase_gates(
        &self,
        project_id: Uuid,
        phase_number: u8,
    ) -> Result<(), Vec<GateId>> {
        let gates = self.store.list_gates_by_project_and_phase(project_id, phase_number).await
            .map_err(|_| vec![])?;
        
        let failed_gates: Vec<GateId> = gates.iter()
            .filter(|g| g.status != GateStatus::Approved)
            .map(|g| g.id)
            .collect();
        
        if failed_gates.is_empty() {
            Ok(())
        } else {
            Err(failed_gates)
        }
    }
    
    /// Get approval history for a gate
    pub async fn get_gate_approvals(
        &self,
        actor_id: ActorId,
        gate_id: GateId,
    ) -> Result<Vec<GateApproval>, GateError> {
        // Verify gate exists
        self.get_gate(actor_id, gate_id).await?;
        
        self.store.list_approvals_by_gate(gate_id).await
    }
}
```

### 3.2 GateStore Trait

```rust
/// Storage interface for gate data
#[async_trait]
pub trait GateStore: Send + Sync {
    async fn create_gate(&self, gate: &Gate) -> Result<(), GateError>;
    async fn get_gate(&self, id: GateId) -> Result<Option<Gate>, GateError>;
    async fn update_gate(&self, gate: &Gate) -> Result<(), GateError>;
    
    async fn get_gate_by_project_phase_type(
        &self,
        project_id: Uuid,
        phase_number: u8,
        gate_type: GateType,
    ) -> Result<Option<Gate>, GateError>;
    
    async fn list_gates_by_project(&self, project_id: Uuid) -> Result<Vec<Gate>, GateError>;
    
    async fn list_gates_by_project_and_phase(
        &self,
        project_id: Uuid,
        phase_number: u8,
    ) -> Result<Vec<Gate>, GateError>;
    
    async fn list_pending_gates_by_roles(&self, roles: &[String]) -> Result<Vec<Gate>, GateError>;
    
    async fn create_approval(&self, approval: &GateApproval) -> Result<(), GateError>;
    async fn list_approvals_by_gate(&self, gate_id: GateId) -> Result<Vec<GateApproval>, GateError>;
}
```

### 3.3 RoleChecker Trait

```rust
/// Interface to authority system for role validation
#[async_trait]
pub trait RoleChecker: Send + Sync {
    /// Check if actor has a specific role
    async fn has_role(&self, actor_id: ActorId, role: &str) -> Result<bool, GateError>;
    
    /// Get all roles for an actor
    async fn get_actor_roles(&self, actor_id: ActorId) -> Result<Vec<String>, GateError>;
}
```

---

## 4. Database Schema

### 4.1 Migration: `0011_gate_system.sql`

```sql
-- gates table
CREATE TABLE gates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phase_number SMALLINT NOT NULL CHECK (phase_number >= 0 AND phase_number <= 9),
    gate_type TEXT NOT NULL CHECK (gate_type IN ('user_approval', 'council_approval', 'ceo_approval', 'security_approval', 'test_approval')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    required_approver_role TEXT NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    UNIQUE(project_id, phase_number, gate_type)
);

-- Index for querying gates by project
CREATE INDEX idx_gates_project ON gates(project_id);

-- Index for finding pending gates by role
CREATE INDEX idx_gates_pending_by_role ON gates(required_approver_role, status) WHERE status = 'pending';

-- Index for phase gate validation
CREATE INDEX idx_gates_project_phase ON gates(project_id, phase_number);

-- gate_approvals table
CREATE TABLE gate_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gate_id UUID NOT NULL REFERENCES gates(id) ON DELETE CASCADE,
    approver_actor_id UUID NOT NULL,
    decision TEXT NOT NULL CHECK (decision IN ('approve', 'reject')),
    reason TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for querying approvals by gate
CREATE INDEX idx_gate_approvals_gate ON gate_approvals(gate_id);

-- Index for querying approvals by actor
CREATE INDEX idx_gate_approvals_actor ON gate_approvals(approver_actor_id);

-- Index for audit trail (chronological)
CREATE INDEX idx_gate_approvals_timestamp ON gate_approvals(timestamp DESC);
```

### 4.2 Schema Notes

* **Unique Constraint:** One gate per (project_id, phase_number, gate_type) triple
* **Foreign Keys:** 
  - `gates.project_id` references `projects.id` (from workflow-engine)
  - `gates.id` referenced by `gate_approvals.gate_id`
* **Cascade Deletion:** Deleting a project deletes all its gates; deleting a gate deletes all its approvals
* **Approval History:** Multiple approvals can exist per gate (e.g., council votes), but gate status reflects final decision

---

## 5. MCP Tools

### 5.1 Tool Definitions

```rust
// In agent-mcp/src/tool_definitions.rs

pub fn gate_create_tool() -> ToolDefinition {
    ToolDefinition {
        name: "gate_create".to_string(),
        description: "Create a quality gate for a project phase".to_string(),
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
                "gate_type": {
                    "type": "string",
                    "enum": ["user_approval", "council_approval", "ceo_approval", "security_approval", "test_approval"],
                    "description": "Type of approval required"
                }
            },
            "required": ["project_id", "phase_number", "gate_type"]
        }),
    }
}

pub fn gate_submit_approval_tool() -> ToolDefinition {
    ToolDefinition {
        name: "gate_submit_approval".to_string(),
        description: "Submit an approval decision (approve or reject) for a gate".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "gate_id": {
                    "type": "string",
                    "description": "Gate UUID"
                },
                "decision": {
                    "type": "string",
                    "enum": ["approve", "reject"],
                    "description": "Approval decision"
                },
                "reason": {
                    "type": "string",
                    "description": "Optional reason for decision"
                }
            },
            "required": ["gate_id", "decision"]
        }),
    }
}

pub fn gate_get_pending_tool() -> ToolDefinition {
    ToolDefinition {
        name: "gate_get_pending".to_string(),
        description: "Get all pending gates for the current actor".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    }
}

pub fn gate_get_tool() -> ToolDefinition {
    ToolDefinition {
        name: "gate_get".to_string(),
        description: "Get gate details by ID".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "gate_id": {
                    "type": "string",
                    "description": "Gate UUID"
                }
            },
            "required": ["gate_id"]
        }),
    }
}

pub fn gate_list_project_tool() -> ToolDefinition {
    ToolDefinition {
        name: "gate_list_project".to_string(),
        description: "List all gates for a project".to_string(),
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

pub fn gate_check_phase_tool() -> ToolDefinition {
    ToolDefinition {
        name: "gate_check_phase".to_string(),
        description: "Check if all gates are passed for a project phase".to_string(),
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

pub fn gate_get_approvals_tool() -> ToolDefinition {
    ToolDefinition {
        name: "gate_get_approvals".to_string(),
        description: "Get approval history for a gate".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "gate_id": {
                    "type": "string",
                    "description": "Gate UUID"
                }
            },
            "required": ["gate_id"]
        }),
    }
}
```

---

## 6. Acceptance Tests

### 6.1 Create Gate

```rust
#[tokio::test]
async fn test_create_gate() {
    // Given: A valid project
    let service = setup_test_service().await;
    let actor_id = create_test_actor();
    let project_id = create_test_project().await;
    
    // When: Create user approval gate for Phase 1
    let gate = service.create_gate(
        actor_id,
        project_id,
        1,
        GateType::UserApproval,
    ).await.unwrap();
    
    // Then: Gate created with pending status
    assert_eq!(gate.project_id, project_id);
    assert_eq!(gate.phase_number, 1);
    assert_eq!(gate.gate_type, GateType::UserApproval);
    assert_eq!(gate.status, GateStatus::Pending);
    assert_eq!(gate.required_approver_role, "user");
}
```

### 6.2 Submit Approval (Success)

```rust
#[tokio::test]
async fn test_submit_approval_success() {
    // Given: Pending gate and actor with correct role
    let service = setup_test_service().await;
    let actor_id = create_test_actor_with_role("user");
    let gate = create_test_gate(&service, GateType::UserApproval).await;
    
    // When: Actor approves gate
    let approval = service.submit_approval(
        actor_id,
        gate.id,
        ApprovalDecision::Approve,
        Some("Requirements look good".to_string()),
    ).await.unwrap();
    
    // Then: Approval recorded
    assert_eq!(approval.decision, ApprovalDecision::Approve);
    assert_eq!(approval.reason, Some("Requirements look good".to_string()));
    
    // And: Gate status updated to approved
    let updated_gate = service.get_gate(actor_id, gate.id).await.unwrap();
    assert_eq!(updated_gate.status, GateStatus::Approved);
    assert!(updated_gate.resolved_at.is_some());
}
```

### 6.3 Submit Approval (Unauthorized)

```rust
#[tokio::test]
async fn test_submit_approval_unauthorized() {
    // Given: Pending CEO gate and actor without CEO role
    let service = setup_test_service().await;
    let actor_id = create_test_actor_with_role("developer"); // Wrong role
    let gate = create_test_gate(&service, GateType::CeoApproval).await;
    
    // When: Actor attempts to approve gate
    let result = service.submit_approval(
        actor_id,
        gate.id,
        ApprovalDecision::Approve,
        None,
    ).await;
    
    // Then: Fails with Unauthorized error
    assert!(matches!(result, Err(GateError::Unauthorized { .. })));
    
    // And: Gate status unchanged
    let gate = service.get_gate(actor_id, gate.id).await.unwrap();
    assert_eq!(gate.status, GateStatus::Pending);
}
```

### 6.4 Submit Approval (Already Resolved)

```rust
#[tokio::test]
async fn test_submit_approval_already_resolved() {
    // Given: Gate already approved
    let service = setup_test_service().await;
    let actor_id = create_test_actor_with_role("user");
    let gate = create_test_gate(&service, GateType::UserApproval).await;
    service.submit_approval(actor_id, gate.id, ApprovalDecision::Approve, None).await.unwrap();
    
    // When: Actor attempts to approve again
    let result = service.submit_approval(
        actor_id,
        gate.id,
        ApprovalDecision::Approve,
        None,
    ).await;
    
    // Then: Fails with AlreadyResolved error
    assert!(matches!(result, Err(GateError::AlreadyResolved { .. })));
}
```

### 6.5 Get Pending Gates

```rust
#[tokio::test]
async fn test_get_pending_gates() {
    // Given: Multiple gates with different roles
    let service = setup_test_service().await;
    let ceo_actor = create_test_actor_with_role("ceo");
    let user_actor = create_test_actor_with_role("user");
    
    let gate1 = create_test_gate(&service, GateType::CeoApproval).await; // Pending
    let gate2 = create_test_gate(&service, GateType::UserApproval).await; // Pending
    let gate3 = create_test_gate(&service, GateType::SecurityApproval).await; // Pending
    
    // When: CEO actor queries pending gates
    let ceo_pending = service.get_pending_gates(ceo_actor).await.unwrap();
    
    // Then: Only CEO gates returned
    assert_eq!(ceo_pending.len(), 1);
    assert_eq!(ceo_pending[0].id, gate1.id);
    
    // When: User actor queries pending gates
    let user_pending = service.get_pending_gates(user_actor).await.unwrap();
    
    // Then: Only user gates returned
    assert_eq!(user_pending.len(), 1);
    assert_eq!(user_pending[0].id, gate2.id);
}
```

### 6.6 Validate Phase Gates

```rust
#[tokio::test]
async fn test_validate_phase_gates_all_passed() {
    // Given: Phase 1 with multiple gates, all approved
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    let actor_id = create_test_actor_with_role("user");
    
    let gate1 = service.create_gate(actor_id, project_id, 1, GateType::UserApproval).await.unwrap();
    service.submit_approval(actor_id, gate1.id, ApprovalDecision::Approve, None).await.unwrap();
    
    // When: Validate phase gates
    let result = service.validate_phase_gates(project_id, 1).await;
    
    // Then: Validation passes
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_phase_gates_some_pending() {
    // Given: Phase 3 with two gates, one pending
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    let actor_id = create_test_actor();
    
    let gate1 = service.create_gate(actor_id, project_id, 3, GateType::CouncilApproval).await.unwrap();
    let gate2 = service.create_gate(actor_id, project_id, 3, GateType::SecurityApproval).await.unwrap();
    
    // Approve gate1, leave gate2 pending
    let council_actor = create_test_actor_with_role("council");
    service.submit_approval(council_actor, gate1.id, ApprovalDecision::Approve, None).await.unwrap();
    
    // When: Validate phase gates
    let result = service.validate_phase_gates(project_id, 3).await;
    
    // Then: Validation fails with pending gate IDs
    assert!(result.is_err());
    let failed_gates = result.unwrap_err();
    assert_eq!(failed_gates.len(), 1);
    assert_eq!(failed_gates[0], gate2.id);
}
```

---

## 7. Open Decisions

### 7.1 USER_APPROVAL_REQUIRED

**Decision:** Should council approval require unanimous vote or majority vote?

**Options:**
1. Unanimous — All council members must approve (stricter)
2. Majority — > 50% council members must approve (more flexible)
3. Quorum + Majority — At least N members must vote, then majority wins

**Recommendation:** Option 3 (Quorum + Majority) with quorum = 3 members. Balances rigor with practicality.

**Status:** PENDING USER APPROVAL

---

### 7.2 COUNCIL_DECISION_REQUIRED

**Decision:** How should the system handle rejected gates?

**Options:**
1. Block forever — Requires manual gate deletion and recreation
2. Allow retry — Submitter can resubmit after addressing concerns
3. Reset — Gate returns to pending status, previous rejection recorded in history

**Recommendation:** Option 3 (Reset to pending). Approval history preserved for audit trail.

**Status:** PENDING COUNCIL REVIEW

---

## 8. Implementation Notes

### 8.1 Crate Structure

```
crates/gate-system/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Module coordinator (pub use only)
│   ├── domain.rs           # Domain types (GateId, GateType, etc.)
│   ├── service.rs          # GateService implementation
│   ├── store.rs            # GateStore trait
│   ├── role_checker.rs     # RoleChecker trait
│   └── error.rs            # Error types
└── tests/
    └── integration.rs      # Acceptance tests
```

### 8.2 Dependencies

```toml
[dependencies]
authority-domain = { path = "../authority-domain" }
control-store = { path = "../control-store" }

uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
async-trait = "0.1"
```

### 8.3 Integration with workflow-engine

The `workflow-engine` will call `gate-system` via the `GateValidator` trait:

```rust
// In workflow-engine/src/gate_validator.rs
impl GateValidator for GateSystemValidator {
    async fn validate_phase_gates(
        &self,
        project_id: ProjectId,
        phase_number: PhaseNumber,
    ) -> Result<(), Vec<String>> {
        self.gate_service
            .validate_phase_gates(project_id.as_uuid(), phase_number.as_u8())
            .await
            .map_err(|gate_ids| gate_ids.iter().map(|id| id.to_string()).collect())
    }
}
```

---

## 9. Handoff Protocol

### 9.1 From PM (Alex)

**Received:**
* This spec document
* Dependency on workflow-engine spec

**Needed Before Implementation:**
* Security Architect (Fatima) security review ⏳ (next step)
* Data Architect (Chen) schema review ⏳ (next step)
* API Architect (Marcus) MCP tool review ⏳ (next step)

### 9.2 To Security Architect (Fatima)

**Please Review:**
1. Approval authorization model (role-based)
2. Audit trail (gate_approvals table)
3. Protection against approval replay/tampering

**Questions:**
* Is role-based authorization sufficient, or should we add capability-based?
* Any concerns with storing approval history indefinitely?

### 9.3 To Data Architect (Chen)

**Please Review:**
1. Database schema (Section 4)
2. Unique constraint on (project_id, phase_number, gate_type)
3. Cascade delete behavior

### 9.4 To API Architect (Marcus)

**Please Review:**
1. MCP tool definitions (Section 5)
2. Error responses for unauthorized approval attempts

---

## 10. Next Steps

1. **Fatima (Security):** Review authorization model → Approve or request changes
2. **Chen (Data):** Review schema → Approve or request changes
3. **Marcus (API):** Review MCP tools → Approve or request changes
4. **Council:** Review open decisions (council voting rules, rejected gate handling)
5. **Dmitri (Backend):** Implement crate after approvals
6. **Meg (QA):** Write integration tests

---

**END OF SPECIFICATION**
