# Task Orchestration System Specification

**Module:** Extends `control-service` (enhances work packet system)  
**Status:** Phase 0c — Specification Draft  
**Owner:** Backend Engineering (Dmitri Volkov)  
**Architects:** Domain (Yusuf), PM (Alex)  
**Date:** 2025-01-30  
**Version:** 0.1.0

---

## 1. Overview

### 1.1 Purpose

The Task Orchestration System provides **work breakdown structure (WBS)** with task dependencies and progress tracking. It:

* Breaks projects into tasks (hierarchical WBS)
* Tracks task status (not_started, in_progress, completed, blocked)
* Manages task dependencies (task B depends on task A)
* Queries task progress and identifies ready tasks
* Assigns tasks to personas

**Not Responsible For:**
* Project lifecycle (delegated to `workflow-engine`)
* Team assignment (delegated to `team-assignments`)
* Artifact storage (delegated to `artifact-store`)

### 1.2 Position in Architecture

```
agent-mcp (MCP tools: task_*)
    ↓
control-service (task orchestration)
    ↓
control-store (persistence: tasks, task_dependencies tables)
    ↓
PostgreSQL
```

---

## 2. Domain Model

### 2.1 Core Types

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use workflow_engine::ProjectId;

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl TaskId {
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

/// Task entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub phase_number: u8, // 0-9
    pub title: String,
    pub description: Option<String>,
    pub assigned_persona: Option<String>, // 'backend', 'frontend', etc.
    pub status: TaskStatus,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task has not started
    NotStarted,
    /// Task is currently in progress
    InProgress,
    /// Task completed successfully
    Completed,
    /// Task is blocked (waiting on dependency)
    Blocked,
}

/// Task dependency (task depends on another task)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub id: Uuid,
    pub task_id: TaskId,
    pub depends_on_task_id: TaskId,
}
```

### 2.2 Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskOrchestrationError {
    #[error("Task not found: {0:?}")]
    TaskNotFound(TaskId),
    
    #[error("Project not found: {0:?}")]
    ProjectNotFound(ProjectId),
    
    #[error("Circular dependency detected: {0:?} -> {1:?}")]
    CircularDependency(TaskId, TaskId),
    
    #[error("Dependency not found: {0:?}")]
    DependencyNotFound(Uuid),
    
    #[error("Database error: {0}")]
    Database(String),
}
```

---

## 3. Service Layer

### 3.1 TaskOrchestrationService

```rust
use authority_domain::ActorId;

pub struct TaskOrchestrationService {
    store: Arc<dyn TaskStore>,
}

impl TaskOrchestrationService {
    /// Create a new task
    pub async fn create_task(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        phase_number: u8,
        title: String,
        description: Option<String>,
        assigned_persona: Option<String>,
        estimated_hours: Option<f64>,
    ) -> Result<Task, TaskOrchestrationError> {
        // Validate project exists
        // Create task entity
        // Store in database
    }
    
    /// Update task status
    pub async fn update_task_status(
        &self,
        actor_id: ActorId,
        task_id: TaskId,
        status: TaskStatus,
    ) -> Result<Task, TaskOrchestrationError> {
        // Check permission
        // Update status
        // If completed, set completed_at
    }
    
    /// Add task dependency
    pub async fn add_dependency(
        &self,
        actor_id: ActorId,
        task_id: TaskId,
        depends_on_task_id: TaskId,
    ) -> Result<TaskDependency, TaskOrchestrationError> {
        // Validate both tasks exist
        // Check for circular dependencies
        // Create dependency
    }
    
    /// Remove task dependency
    pub async fn remove_dependency(
        &self,
        actor_id: ActorId,
        task_id: TaskId,
        depends_on_task_id: TaskId,
    ) -> Result<(), TaskOrchestrationError> {
        // Remove dependency
    }
    
    /// List tasks with filters
    pub async fn list_tasks(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
        phase_number: Option<u8>,
        status: Option<TaskStatus>,
    ) -> Result<Vec<Task>, TaskOrchestrationError> {
        // Query with filters
    }
    
    /// Get ready tasks (no incomplete dependencies)
    pub async fn get_ready_tasks(
        &self,
        actor_id: ActorId,
        project_id: ProjectId,
    ) -> Result<Vec<Task>, TaskOrchestrationError> {
        // Query tasks with status=not_started
        // Filter out tasks with incomplete dependencies
    }
    
    /// Get task dependencies
    pub async fn get_dependencies(
        &self,
        actor_id: ActorId,
        task_id: TaskId,
    ) -> Result<Vec<Task>, TaskOrchestrationError> {
        // Get all tasks this task depends on
    }
    
    /// Get dependent tasks (tasks that depend on this task)
    pub async fn get_dependent_tasks(
        &self,
        actor_id: ActorId,
        task_id: TaskId,
    ) -> Result<Vec<Task>, TaskOrchestrationError> {
        // Get all tasks that depend on this task
    }
}
```

---

## 4. Database Schema

```sql
-- tasks table
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phase_number INT NOT NULL CHECK (phase_number BETWEEN 0 AND 9),
    title TEXT NOT NULL,
    description TEXT,
    assigned_persona TEXT,
    status TEXT NOT NULL DEFAULT 'not_started' CHECK (status IN ('not_started', 'in_progress', 'completed', 'blocked')),
    estimated_hours NUMERIC,
    actual_hours NUMERIC,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_tasks_project ON tasks(project_id);
CREATE INDEX idx_tasks_phase ON tasks(phase_number);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_persona ON tasks(assigned_persona);

-- task_dependencies table
CREATE TABLE task_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    depends_on_task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    UNIQUE(task_id, depends_on_task_id)
);

CREATE INDEX idx_task_dependencies_task ON task_dependencies(task_id);
CREATE INDEX idx_task_dependencies_depends ON task_dependencies(depends_on_task_id);
```

**Migration:** `db/migrations/008_task_orchestration.sql`

---

## 5. MCP Tools

```json
[
  {
    "name": "task_create",
    "description": "Create a new task",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"},
        "phase_number": {"type": "integer", "minimum": 0, "maximum": 9},
        "title": {"type": "string"},
        "description": {"type": "string"},
        "assigned_persona": {"type": "string"},
        "estimated_hours": {"type": "number"}
      },
      "required": ["project_id", "phase_number", "title"]
    }
  },
  {
    "name": "task_update_status",
    "description": "Update task status",
    "inputSchema": {
      "type": "object",
      "properties": {
        "task_id": {"type": "string", "format": "uuid"},
        "status": {"type": "string", "enum": ["not_started", "in_progress", "completed", "blocked"]}
      },
      "required": ["task_id", "status"]
    }
  },
  {
    "name": "task_add_dependency",
    "description": "Add task dependency",
    "inputSchema": {
      "type": "object",
      "properties": {
        "task_id": {"type": "string", "format": "uuid"},
        "depends_on_task_id": {"type": "string", "format": "uuid"}
      },
      "required": ["task_id", "depends_on_task_id"]
    }
  },
  {
    "name": "task_list",
    "description": "List tasks with filters",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"},
        "phase_number": {"type": "integer"},
        "status": {"type": "string"}
      },
      "required": ["project_id"]
    }
  },
  {
    "name": "task_get_ready",
    "description": "Get ready tasks (no incomplete dependencies)",
    "inputSchema": {
      "type": "object",
      "properties": {
        "project_id": {"type": "string", "format": "uuid"}
      },
      "required": ["project_id"]
    }
  }
]
```

---

## 6. Dependency Resolution

### 6.1 Circular Dependency Detection

```rust
impl TaskOrchestrationService {
    /// Check if adding dependency would create a cycle
    async fn would_create_cycle(
        &self,
        task_id: TaskId,
        depends_on_task_id: TaskId,
    ) -> Result<bool, TaskOrchestrationError> {
        // Use DFS to detect cycles
        let mut visited = HashSet::new();
        let mut stack = vec![depends_on_task_id];
        
        while let Some(current) = stack.pop() {
            if current == task_id {
                return Ok(true); // Cycle detected
            }
            
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            
            // Get dependencies of current task
            let deps = self.store.get_dependencies(current).await?;
            stack.extend(deps.into_iter().map(|d| d.depends_on_task_id));
        }
        
        Ok(false) // No cycle
    }
}
```

### 6.2 Ready Task Query

```rust
impl TaskOrchestrationService {
    /// Get tasks with no incomplete dependencies
    async fn get_ready_tasks_impl(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<Task>, TaskOrchestrationError> {
        let all_tasks = self.store.list_tasks(project_id, None, Some(TaskStatus::NotStarted)).await?;
        let mut ready = Vec::new();
        
        for task in all_tasks {
            let deps = self.store.get_dependencies(task.id).await?;
            let all_completed = deps.into_iter().all(|d| d.status == TaskStatus::Completed);
            
            if all_completed {
                ready.push(task);
            }
        }
        
        Ok(ready)
    }
}
```

---

## 7. Acceptance Tests

```rust
#[tokio::test]
async fn test_create_task_with_dependencies() {
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    
    // Create task A
    let task_a = service.create_task(..., "Task A".into(), ...).await.unwrap();
    
    // Create task B that depends on A
    let task_b = service.create_task(..., "Task B".into(), ...).await.unwrap();
    service.add_dependency(..., task_b.id, task_a.id).await.unwrap();
    
    // Get dependencies
    let deps = service.get_dependencies(..., task_b.id).await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].id, task_a.id);
}

#[tokio::test]
async fn test_circular_dependency_rejected() {
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    
    let task_a = service.create_task(...).await.unwrap();
    let task_b = service.create_task(...).await.unwrap();
    
    // A depends on B
    service.add_dependency(..., task_a.id, task_b.id).await.unwrap();
    
    // B depends on A should fail
    let result = service.add_dependency(..., task_b.id, task_a.id).await;
    assert!(matches!(result, Err(TaskOrchestrationError::CircularDependency(_, _))));
}

#[tokio::test]
async fn test_get_ready_tasks() {
    let service = setup_test_service().await;
    let project_id = create_test_project().await;
    
    let task_a = service.create_task(...).await.unwrap();
    let task_b = service.create_task(...).await.unwrap();
    service.add_dependency(..., task_b.id, task_a.id).await.unwrap();
    
    // Initially, only task_a is ready
    let ready = service.get_ready_tasks(..., project_id).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, task_a.id);
    
    // Complete task_a
    service.update_task_status(..., task_a.id, TaskStatus::Completed).await.unwrap();
    
    // Now task_b is ready
    let ready = service.get_ready_tasks(..., project_id).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, task_b.id);
}
```

---

## 8. Open Questions

1. Should tasks support sub-tasks (hierarchical WBS)?
2. Should we track actual hours automatically or require manual input?
3. Should tasks support time estimates with confidence intervals?
4. Should we support critical path analysis?

---

**Implementation Checklist:**
- [ ] Extend `control-service` with task orchestration
- [ ] Add persistence layer in `control-store`
- [ ] Create database migration
- [ ] Implement circular dependency detection
- [ ] Implement MCP tools in `agent-mcp`
- [ ] Write acceptance tests
- [ ] Submit for CTO review
