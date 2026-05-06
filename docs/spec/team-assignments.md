# Team Assignment System Specification

**Module:** `team_assignments` (extends `control-store`)  
**Status:** Phase 0b — Specification Draft  
**Owner:** Backend Engineering (Dmitri Volkov)  
**Architects:** Data (Chen), Domain (Yusuf)  
**Date:** 2025-01-30  
**Version:** 0.1.0

---

## 1. Overview

### 1.1 Purpose

The Team Assignment System tracks **which personas/skills are assigned to projects**. It:

* Assigns personas to projects (e.g., "Dmitri (backend) assigned to Project X")
* Tracks skill requirements per project
* Provides queries for: "Who's assigned to Project X?" and "What projects is Dmitri on?"
* Supports assignment lifecycle (active, completed)

**Not Responsible For:**
* Skill definitions (delegated to `authority-domain`)
* Project lifecycle (delegated to `workflow-engine`)
* Task assignment (delegated to `task-orchestration`)

### 1.2 Position in Architecture

```
agent-mcp (MCP tools: team_*)
    ↓
control-service (service layer)
    ↓
control-store (persistence: team_assignments table)
    ↓
PostgreSQL
```

### 1.3 Persona Types

From RealmForge skill system:
* **Engineering:** `backend`, `frontend`, `data-engineer`
* **Architecture:** `domain-architect`, `security-architect`, `api-architect`, `data-architect`
* **Quality:** `qa`
* **Management:** `pm`, `ceo`
* **Domain:** `tech-writer`, `biz-user`, `accountant`, `release-manager`

---

## 2. Domain Model

### 2.1 Core Types

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use workflow_engine::ProjectId;

/// Team assignment entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAssignment {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub persona_name: String, // 'backend', 'frontend', 'qa', etc.
    pub skill_name: Option<String>, // Optional specific skill grant
    pub assigned_at: DateTime<Utc>,
    pub assigned_by_actor_id: Uuid,
    pub status: AssignmentStatus,
}

/// Assignment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStatus {
    /// Assignment is active
    Active,
    /// Assignment completed (project done or persona rotated off)
    Completed,
}
```

### 2.2 Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TeamAssignmentError {
    #[error("Project not found: {0:?}")]
    ProjectNotFound(ProjectId),
    
    #[error("Persona already assigned: {0}")]
    AlreadyAssigned(String),
    
    #[error("Assignment not found: {0}")]
    NotFound(Uuid),
    
    #[error("Database error: {0}")]
    Database(String),
}
```

---

## 3. Service Layer

### 3.1 TeamAssignmentService

```rust
use authority_domain::ActorId;

pub struct TeamAssignmentService {
    store: Arc<dyn TeamAssignmentStore>,
}

impl TeamAssignmentService {
    /// Assign persona to project
    pub async fn assign_persona(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        persona_name: String,
        skill_name: Option<String>,
    ) -> Result<TeamAssignment, TeamAssignmentError> {
        // Validate project exists
        // Check persona not already assigned
        // Create assignment
    }
    
    /// Unassign persona from project
    pub async fn unassign_persona(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        persona_name: String,
    ) -> Result<(), TeamAssignmentError> {
        // Mark assignment as completed
    }
    
    /// Get assignments for project
    pub async fn get_assignments(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
    ) -> Result<Vec<TeamAssignment>, TeamAssignmentError> {
        // Query active assignments
    }
    
    /// Get projects for persona
    pub async fn get_persona_projects(
        &self,
        actor_id: ActorId,
        persona_name: String,
    ) -> Result<Vec<ProjectId>, TeamAssignmentError> {
        // Query active assignments
    }
}
```

---

## 4. Database Schema

```sql
-- team_assignments table
CREATE TABLE team_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    persona_name TEXT NOT NULL,
    skill_name TEXT,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    assigned_by_actor_id UUID NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed')),
    UNIQUE(project_id, persona_name) WHERE status = 'active'
);

CREATE INDEX idx_team_assignments_project ON team_assignments(project_id);
CREATE INDEX idx_team_assignments_persona ON team_assignments(persona_name);
CREATE INDEX idx_team_assignments_status ON team_assignments(status);
```

**Migration:** `db/migrations/006_team_assignments.sql`

---

## 5. MCP Tools

```json
[
  {
    "name": "team_assign",
    "description": "Assign persona to project",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"},
        "persona_name": {"type": "string"},
        "skill_name": {"type": "string"}
      },
      "required": ["project_id", "persona_name"]
    }
  },
  {
    "name": "team_unassign",
    "description": "Unassign persona from project",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"},
        "persona_name": {"type": "string"}
      },
      "required": ["project_id", "persona_name"]
    }
  },
  {
    "name": "team_get_assignments",
    "description": "Get team assignments for project",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"}
      },
      "required": ["project_id"]
    }
  },
  {
    "name": "team_get_persona_projects",
    "description": "Get projects for persona",
    "inputSchema": {
      "type": "object",
      "properties": {
        "persona_name": {"type": "string"}
      },
      "required": ["persona_name"]
    }
  }
]
```

---

## 6. Acceptance Tests

```rust
#[tokio::test]
async fn test_assign_persona_to_project() {
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    
    let assignment = service.assign_persona(
        test_actor_id(),
        project_id,
        "backend".into(),
        None,
    ).await.unwrap();
    
    assert_eq!(assignment.persona_name, "backend");
    assert_eq!(assignment.status, AssignmentStatus::Active);
}

#[tokio::test]
async fn test_cannot_double_assign_persona() {
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    
    service.assign_persona(..., "backend".into(), None).await.unwrap();
    
    let result = service.assign_persona(..., "backend".into(), None).await;
    assert!(matches!(result, Err(TeamAssignmentError::AlreadyAssigned(_))));
}
```

---

**Implementation Checklist:**
- [ ] Extend `control-store` with `team_assignments` module
- [ ] Create database migration
- [ ] Implement service layer in `control-service`
- [ ] Add MCP tools in `agent-mcp`
- [ ] Write acceptance tests
- [ ] Submit for CTO review
