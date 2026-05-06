# Application Type Registry Specification

**Module:** `application_types` (extends `control-store`)  
**Status:** Phase 0c — Specification Draft  
**Owner:** Backend Engineering (Dmitri Volkov)  
**Architects:** Data (Chen), PM (Alex)  
**Date:** 2025-01-30  
**Version:** 0.1.0

---

## 1. Overview

### 1.1 Purpose

The Application Type Registry stores and queries the **15 application types** with their metadata (from APPLICATION_TYPE_MATRIX.md). It:

* Stores application type definitions (complexity, timeline, components, personas)
* Provides query interface (by complexity, timeline, required skills)
* Returns application type requirements (components, deliverables, considerations)

**Not Responsible For:**
* Project creation (delegated to `workflow-engine`)
* Team assignment (delegated to `team-assignments`)

### 1.2 Application Types

From APPLICATION_TYPE_MATRIX.md:

1. `01_static_website` — Static Landing Page
2. `02_static_site_forms` — Static Site + Forms
3. `03_dashboard_web_app` — Dashboard Web App
4. `04_crud_web_app` — CRUD Web App
5. `05_realtime_web_app` — Real-Time Web App
6. `06_ml_web_app` — ML-Powered Web App
7. `07_mobile_app` — Mobile App
8. `08_desktop_app` — Desktop App
9. `09_embedded_agent` — Embedded AI Agent
10. `10_api_service` — API Service
11. `11_data_pipeline` — Data Pipeline
12. `12_ml_pipeline` — ML Pipeline
13. `13_iot_system` — IoT System
14. `14_blockchain_app` — Blockchain DApp
15. `15_distributed_system` — Distributed System

---

## 2. Domain Model

### 2.1 Core Types

```rust
use serde::{Deserialize, Serialize};

/// Application type entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationType {
    pub id: String, // e.g., '01_static_website'
    pub name: String,
    pub description: String,
    pub complexity: Complexity,
    pub timeline_weeks: String, // e.g., '1-2', '4-8'
    pub components: Components,
    pub required_personas: Vec<String>,
    pub deliverables: Vec<String>,
    pub special_considerations: Option<String>,
}

/// Application complexity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Complexity {
    Low,
    Medium,
    MediumHigh,
    High,
}

/// Application components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub frontend: bool,
    pub backend: bool,
    pub database: bool,
    pub external_apis: bool,
    pub authentication: bool,
    pub realtime: bool,
    pub mobile: bool,
    pub ml_model: bool,
}
```

### 2.2 Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationTypeError {
    #[error("Application type not found: {0}")]
    NotFound(String),
    
    #[error("Database error: {0}")]
    Database(String),
}
```

---

## 3. Service Layer

```rust
pub struct ApplicationTypeService {
    store: Arc<dyn ApplicationTypeStore>,
}

impl ApplicationTypeService {
    /// List application types (with filters)
    pub async fn list_application_types(
        &self,
        complexity: Option<Complexity>,
        max_timeline_weeks: Option<u32>,
    ) -> Result<Vec<ApplicationType>, ApplicationTypeError> {
        // Query with filters
    }
    
    /// Get application type by ID
    pub async fn get_application_type(
        &self,
        id: String,
    ) -> Result<ApplicationType, ApplicationTypeError> {
        // Query by ID
    }
    
    /// Get requirements for application type
    pub async fn get_requirements(
        &self,
        id: String,
    ) -> Result<ApplicationTypeRequirements, ApplicationTypeError> {
        // Return components, personas, deliverables
    }
}

/// Application type requirements summary
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationTypeRequirements {
    pub required_components: Vec<String>,
    pub required_personas: Vec<String>,
    pub deliverables: Vec<String>,
}
```

---

## 4. Database Schema

```sql
-- application_types table
CREATE TABLE application_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    complexity TEXT NOT NULL CHECK (complexity IN ('low', 'medium', 'medium_high', 'high')),
    timeline_weeks TEXT NOT NULL,
    components JSONB NOT NULL,
    required_personas JSONB NOT NULL,
    deliverables JSONB NOT NULL,
    special_considerations TEXT
);

CREATE INDEX idx_application_types_complexity ON application_types(complexity);

-- Seed data from APPLICATION_TYPE_MATRIX.md
INSERT INTO application_types (id, name, description, complexity, timeline_weeks, components, required_personas, deliverables, special_considerations) VALUES
('01_static_website', 'Static Landing Page', 'Simple HTML/CSS website', 'low', '1-2', 
 '{"frontend":true,"backend":false,"database":false,"external_apis":false,"authentication":false,"realtime":false,"mobile":false,"ml_model":false}',
 '["frontend","tech-writer"]',
 '["landing_page","documentation"]',
 'Focus on SEO, accessibility, and performance'),
-- ... (14 more application types)
```

**Migration:** `db/migrations/007_application_types.sql`

---

## 5. MCP Tools

```json
[
  {
    "name": "apptype_list",
    "description": "List application types with filters",
    "inputSchema": {
      "type": "object",
      "properties": {
        "complexity": {"type": "string", "enum": ["low", "medium", "medium_high", "high"]},
        "max_timeline_weeks": {"type": "integer"}
      }
    }
  },
  {
    "name": "apptype_get",
    "description": "Get application type details",
    "inputSchema": {
      "type": "object",
      "properties": {
        "id": {"type": "string"}
      },
      "required": ["id"]
    }
  },
  {
    "name": "apptype_get_requirements",
    "description": "Get requirements for application type",
    "inputSchema": {
      "type": "object",
      "properties": {
        "id": {"type": "string"}
      },
      "required": ["id"]
    }
  }
]
```

---

## 6. Seed Data

From APPLICATION_TYPE_MATRIX.md, create SQL INSERT statements for all 15 types:

```sql
INSERT INTO application_types VALUES
('01_static_website', 'Static Landing Page', ...),
('02_static_site_forms', 'Static Site + Forms', ...),
...
('15_distributed_system', 'Distributed System', ...);
```

---

**Implementation Checklist:**
- [ ] Extend `control-store` with `application_types` module
- [ ] Create database migration with seed data
- [ ] Implement service layer in `control-service`
- [ ] Add MCP tools in `agent-mcp`
- [ ] Write acceptance tests
- [ ] Submit for CTO review
