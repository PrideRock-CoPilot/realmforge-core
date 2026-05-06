# Artifact Store Specification

**Crate:** `artifact-store`  
**Status:** Phase 0b — Specification Draft  
**Owner:** Backend Engineering (Dmitri Volkov)  
**Architects:** Data (Chen), API (Marcus), Domain (Yusuf)  
**Date:** 2025-01-30  
**Version:** 0.1.0

---

## 1. Overview

### 1.1 Purpose

The Artifact Store provides **structured artifact storage with versioning** for RealmForge project artifacts. It:

* Stores structured artifacts (requirements, designs, ADRs, test plans, runbooks)
* Versions artifacts immutably (create new version, never modify existing)
* Generates artifacts from templates with variable substitution
* Provides queryable artifact metadata (project_id, phase, type, version)
* Supports artifact retrieval by project/phase/type

**Not Responsible For:**
* File storage (delegated to `object-store` for binary files like mockups)
* Project lifecycle (delegated to `workflow-engine`)
* Team assignments (delegated to `control-store`)

### 1.2 Position in Architecture

```
agent-mcp (MCP tools: artifact_*)
    ↓
artifact-store (service layer)
    ↓
control-store (persistence: artifacts, artifact_templates tables)
    ↓
PostgreSQL
```

**Dependencies:**
* `authority-domain` — ActorId for ownership tracking
* `control-store` — Database persistence
* `workflow-engine` — ProjectId references

**Dependents:**
* `agent-mcp` — MCP tools for artifact management
* `workflow-engine` — Artifact references in phase deliverables

### 1.3 Artifact Types

 Type | Phase | Description | Template |
------|-------|-------------|----------|
 `requirements` | 0 | REQUIREMENTS.md — User stories, constraints | Yes |
 `design` | 1 | Design mockups/diagrams | No (file-based) |
 `technical_assessment` | 2 | TECHNICAL_ASSESSMENT.md — Data/integration audit | Yes |
 `adr` | 3 | Architecture Decision Record | Yes |
 `project_plan` | 5 | PROJECT_PLAN.md — WBS, assignments | Yes |
 `test_plan` | 6 | Test strategy, test cases | Yes |
 `threat_model` | 6 | Security threat model (STRIDE) | Yes |
 `runbook` | 8 | Deployment runbook | Yes |

---

## 2. Domain Model

### 2.1 Core Types

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use workflow_engine::ProjectId;

/// Unique identifier for an artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(Uuid);

impl ArtifactId {
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

/// Artifact type (maps to phase deliverables)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Requirements,
    Design,
    TechnicalAssessment,
    Adr,  // Architecture Decision Record
    ProjectPlan,
    TestPlan,
    ThreatModel,
    Runbook,
}

/// Artifact entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    pub project_id: ProjectId,
    pub phase_number: u8, // 0-9
    pub artifact_type: ArtifactType,
    pub title: String,
    pub content: String, // Markdown content
    pub version: u32, // Version number (1, 2, 3, ...)
    pub created_by_actor_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub replaced_by: Option<ArtifactId>, // Link to newer version
}

/// Artifact template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactTemplate {
    pub id: Uuid,
    pub template_type: ArtifactType,
    pub content: String, // Markdown with {{variable}} placeholders
    pub version: u32,
    pub created_at: DateTime<Utc>,
}

/// Template variables (key-value pairs for substitution)
pub type TemplateVariables = std::collections::HashMap<String, String>;
```

### 2.2 Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("Artifact not found: {0}")]
    NotFound(ArtifactId),
    
    #[error("Project not found: {0:?}")]
    ProjectNotFound(ProjectId),
    
    #[error("Template not found for type: {0:?}")]
    TemplateNotFound(ArtifactType),
    
    #[error("Template variable missing: {0}")]
    MissingVariable(String),
    
    #[error("Artifact already replaced by version {0}")]
    Superseded(u32),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Authorization error: {0}")]
    Unauthorized(String),
}
```

---

## 3. Service Layer (Public API)

### 3.1 ArtifactService

```rust
use authority_domain::ActorId;

/// Service for artifact storage and retrieval
pub struct ArtifactService {
    store: Arc<dyn ArtifactStore>,
}

impl ArtifactService {
    /// Create a new artifact
    pub async fn create_artifact(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        phase_number: u8,
        artifact_type: ArtifactType,
        title: String,
        content: String,
    ) -> Result<Artifact, ArtifactError> {
        // Validate project exists
        // Check actor has write permission
        // Create artifact with version = 1
        // Store in database
        // Return artifact
    }
    
    /// Get artifact by ID
    pub async fn get_artifact(
        &self,
        actor_id: ActorId,
        artifact_id: ArtifactId,
    ) -> Result<Artifact, ArtifactError> {
        // Check read permission
        // Query database
    }
    
    /// List artifacts (with filters)
    pub async fn list_artifacts(
        &self,
        actor_id: ActorId,
        project_id: Option<ProjectId>,
        phase_number: Option<u8>,
        artifact_type: Option<ArtifactType>,
        latest_only: bool, // Only return latest versions
    ) -> Result<Vec<Artifact>, ArtifactError> {
        // Check read permission for project
        // Query with filters
        // If latest_only, exclude superseded artifacts
    }
    
    /// Update artifact (creates new version)
    pub async fn update_artifact(
        &self,
        actor_id: ActorId,
        artifact_id: ArtifactId,
        content: String,
    ) -> Result<Artifact, ArtifactError> {
        // Get existing artifact
        // Check write permission
        // Create new version (version + 1)
        // Mark old artifact as replaced_by new version
        // Return new artifact
    }
    
    /// Generate artifact from template
    pub async fn generate_from_template(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        template_type: ArtifactType,
        variables: TemplateVariables,
    ) -> Result<String, ArtifactError> {
        // Get template for type
        // Validate all required variables present
        // Substitute {{variable}} with values
        // Return rendered content
    }
    
    /// Create or update template
    pub async fn upsert_template(
        &self,
        actor_id: ActorId,
        template_type: ArtifactType,
        content: String,
    ) -> Result<ArtifactTemplate, ArtifactError> {
        // Check actor has template:write permission
        // Create or update template
        // Increment version if updating
    }
    
    /// Get template for artifact type
    pub async fn get_template(
        &self,
        artifact_type: ArtifactType,
    ) -> Result<ArtifactTemplate, ArtifactError> {
        // Query template (latest version)
    }
}
```

---

## 4. Database Schema

### 4.1 Tables

```sql
-- artifacts table
CREATE TABLE artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phase_number INT NOT NULL CHECK (phase_number BETWEEN 0 AND 9),
    artifact_type TEXT NOT NULL CHECK (artifact_type IN (
        'requirements',
        'design',
        'technical_assessment',
        'adr',
        'project_plan',
        'test_plan',
        'threat_model',
        'runbook'
    )),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1 CHECK (version > 0),
    created_by_actor_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    replaced_by UUID REFERENCES artifacts(id),
    UNIQUE(project_id, artifact_type, version)
);

CREATE INDEX idx_artifacts_project ON artifacts(project_id);
CREATE INDEX idx_artifacts_type ON artifacts(artifact_type);
CREATE INDEX idx_artifacts_phase ON artifacts(phase_number);
CREATE INDEX idx_artifacts_latest ON artifacts(project_id, artifact_type) WHERE replaced_by IS NULL;

-- artifact_templates table
CREATE TABLE artifact_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_type TEXT NOT NULL CHECK (template_type IN (
        'requirements',
        'technical_assessment',
        'adr',
        'project_plan',
        'test_plan',
        'threat_model',
        'runbook'
    )),
    content TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(template_type, version)
);

CREATE INDEX idx_templates_type_latest ON artifact_templates(template_type, version DESC);
```

### 4.2 Migration

**Location:** `db/migrations/005_artifact_store.sql`

**Dependencies:**
* `003_workflow_engine.sql` (projects table)
* Actors table (from foundation migrations)

---

## 5. MCP Tools

### 5.1 Tool Definitions

```json
[
  {
    "name": "artifact_create",
    "description": "Create a new artifact for a project",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"},
        "phase_number": {"type": "integer", "minimum": 0, "maximum": 9},
        "artifact_type": {
          "type": "string",
          "enum": ["requirements", "design", "technical_assessment", "adr", "project_plan", "test_plan", "threat_model", "runbook"]
        },
        "title": {"type": "string"},
        "content": {"type": "string", "description": "Markdown content"}
      },
      "required": ["project_id", "phase_number", "artifact_type", "title", "content"]
    }
  },
  {
    "name": "artifact_get",
    "description": "Get artifact by ID",
    "inputSchema": {
      "type": "object",
      "properties": {
        "artifact_id": {"type": "string", "format": "uuid"}
      },
      "required": ["artifact_id"]
    }
  },
  {
    "name": "artifact_list",
    "description": "List artifacts with optional filters",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"},
        "phase_number": {"type": "integer", "minimum": 0, "maximum": 9},
        "artifact_type": {"type": "string"},
        "latest_only": {"type": "boolean", "default": true}
      }
    }
  },
  {
    "name": "artifact_update",
    "description": "Update artifact (creates new version)",
    "inputSchema": {
      "type": "object",
      "properties": {
        "artifact_id": {"type": "string", "format": "uuid"},
        "content": {"type": "string"}
      },
      "required": ["artifact_id", "content"]
    }
  },
  {
    "name": "artifact_generate_from_template",
    "description": "Generate artifact content from template",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"},
        "template_type": {"type": "string"},
        "variables": {
          "type": "object",
          "additionalProperties": {"type": "string"}
        }
      },
      "required": ["project_id", "template_type", "variables"]
    }
  },
  {
    "name": "template_create",
    "description": "Create or update artifact template",
    "inputSchema": {
      "type": "object",
      "properties": {
        "template_type": {"type": "string"},
        "content": {"type": "string", "description": "Template with {{variable}} placeholders"}
      },
      "required": ["template_type", "content"]
    }
  },
  {
    "name": "template_get",
    "description": "Get template for artifact type",
    "inputSchema": {
      "type": "object",
      "properties": {
        "template_type": {"type": "string"}
      },
      "required": ["template_type"]
    }
  }
]
```

---

## 6. Template System

### 6.1 Template Syntax

Templates use Handlebars-style `{{variable}}` syntax:

```markdown
# Requirements Document

**Project:** {{project_name}}  
**Owner:** {{owner_name}}  
**Date:** {{date}}

## User Stories

{{user_stories}}

## Constraints

{{constraints}}
```

### 6.2 Seed Templates

**Requirements Template:**

```markdown
# Requirements Document

**Project:** {{project_name}}  
**Application Type:** {{application_type}}  
**Owner:** {{owner_name}}  
**Date:** {{date}}

## Overview

{{overview}}

## User Stories

{{user_stories}}

## Acceptance Criteria

{{acceptance_criteria}}

## Constraints

### Technical Constraints
{{technical_constraints}}

### Business Constraints
{{business_constraints}}

### Regulatory Constraints
{{regulatory_constraints}}

## Out of Scope

{{out_of_scope}}
```

**ADR Template:**

```markdown
# ADR-{{number}}: {{title}}

**Date:** {{date}}  
**Status:** {{status}}  
**Deciders:** {{deciders}}

## Context

{{context}}

## Decision

{{decision}}

## Consequences

### Positive
{{positive_consequences}}

### Negative
{{negative_consequences}}

## Alternatives Considered

{{alternatives}}
```

**Project Plan Template:**

```markdown
# Project Plan: {{project_name}}

**Owner:** {{owner_name}}  
**Start Date:** {{start_date}}  
**Target Date:** {{target_date}}

## Work Breakdown Structure

{{wbs}}

## Team Assignments

{{team_assignments}}

## Dependencies

{{dependencies}}

## Risks

{{risks}}
```

---

## 7. Acceptance Tests

### 7.1 Versioning Tests

```rust
#[tokio::test]
async fn test_artifact_update_creates_new_version() {
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    
    // Create v1
    let v1 = service.create_artifact(
        test_actor_id(),
        project_id,
        0,
        ArtifactType::Requirements,
        "Requirements".into(),
        "Initial content".into(),
    ).await.unwrap();
    
    assert_eq!(v1.version, 1);
    assert!(v1.replaced_by.is_none());
    
    // Update creates v2
    let v2 = service.update_artifact(
        test_actor_id(),
        v1.id,
        "Updated content".into(),
    ).await.unwrap();
    
    assert_eq!(v2.version, 2);
    assert!(v2.replaced_by.is_none());
    
    // v1 now points to v2
    let v1_updated = service.get_artifact(test_actor_id(), v1.id).await.unwrap();
    assert_eq!(v1_updated.replaced_by, Some(v2.id));
}

#[tokio::test]
async fn test_list_artifacts_latest_only() {
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    
    // Create and update artifact
    let v1 = service.create_artifact(...).await.unwrap();
    let v2 = service.update_artifact(..., v1.id, ...).await.unwrap();
    
    // List with latest_only=true
    let artifacts = service.list_artifacts(
        test_actor_id(),
        Some(project_id),
        None,
        None,
        true, // latest_only
    ).await.unwrap();
    
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].id, v2.id);
}
```

### 7.2 Template Tests

```rust
#[tokio::test]
async fn test_generate_from_template() {
    let service = setup_test_service().await;
    
    // Create template
    service.upsert_template(
        test_actor_id(),
        ArtifactType::Requirements,
        "# {{project_name}}\n\n{{content}}".into(),
    ).await.unwrap();
    
    // Generate artifact
    let mut variables = HashMap::new();
    variables.insert("project_name".into(), "Test Project".into());
    variables.insert("content".into(), "This is a test.".into());
    
    let rendered = service.generate_from_template(
        test_actor_id(),
        test_project_id(),
        ArtifactType::Requirements,
        variables,
    ).await.unwrap();
    
    assert_eq!(rendered, "# Test Project\n\nThis is a test.");
}

#[tokio::test]
async fn test_missing_template_variable_errors() {
    let service = setup_test_service().await;
    
    service.upsert_template(
        test_actor_id(),
        ArtifactType::Requirements,
        "{{required_var}}".into(),
    ).await.unwrap();
    
    let result = service.generate_from_template(
        test_actor_id(),
        test_project_id(),
        ArtifactType::Requirements,
        HashMap::new(), // missing required_var
    ).await;
    
    assert!(matches!(result, Err(ArtifactError::MissingVariable(_))));
}
```

---

## 8. Security Considerations

### 8.1 Permissions

* **artifact:create** — Create artifacts for projects actor has access to
* **artifact:read** — Read artifacts for projects actor has access to
* **artifact:update** — Update artifacts for projects actor owns or is assigned to
* **template:write** — Create/update templates (admin only)

### 8.2 Audit Events

All artifact operations emit audit events:
* `artifact.created` — New artifact created
* `artifact.updated` — New version created
* `template.created` — New template created
* `template.updated` — Template updated

---

## 9. Open Questions

1. Should artifacts support comments/annotations?
2. Should we support artifact branching (e.g., alternative designs)?
3. Should templates support loops/conditionals or just variable substitution?
4. Should we support artifact attachments (e.g., images in requirements)?

---

## 10. Implementation Checklist

- [ ] Create `artifact-store` crate with domain types
- [ ] Implement template substitution engine
- [ ] Add persistence layer in `control-store`
- [ ] Create database migration `005_artifact_store.sql`
- [ ] Seed templates for all artifact types
- [ ] Implement MCP tools in `agent-mcp`
- [ ] Write acceptance tests
- [ ] Document in crate README
- [ ] Submit for CTO review

---

**Next Steps:**
1. Submit this spec to Rena (CTO) for architecture review
2. Submit database schema to Chen (Data Architect) for review
3. Create team-assignments and application-types specs
4. Begin implementation after approvals
