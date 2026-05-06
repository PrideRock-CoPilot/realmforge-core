---
doc_id: DOC-SPEC-025
title: "Intake System Design — Structured Deterministic Intake Engine"
status: draft
owner: backend
reviewers: [cto, domain-architect, security-architect, pm, qa, tech-writer, frontend]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: intake
work_path_ids: [WP-CORE-001]
related_decision_ids: [DEC-COUNCIL-INTAKE-001]
related_file_ids: [FILE-CRATE-DOMAIN-LIB, FILE-CRATE-SERVICE-LIB, FILE-CRATE-STORE-LIB, FILE-CRATE-API-LIB, FILE-CRATE-MCP-LIB, FILE-CRATE-CLI-MAIN, FILE-CRATE-INTAKE-LIB, FILE-CRATE-INTAKE-TREE, FILE-CRATE-INTAKE-CONDITION, FILE-CRATE-INTAKE-ENGINE, FILE-CRATE-INTAKE-MAPPER, FILE-CRATE-INTAKE-VALIDATE, FILE-CRATE-INTAKE-ERROR, FILE-CRATE-STORE-INTAKE, FILE-DB-013, FILE-TREE-STATIC-SITE, FILE-TREE-WEB-APP, FILE-TREE-API-SERVICE]
visual_node_ids: [VN-INTAKE-SYSTEM]
visual_edge_ids: [VE-INTAKE-TO-PLAN, VE-INTAKE-TO-SERVICE]
approval_state: pending
---

# RealmForge Intake System Design

> **Intake Decision Record:** Council Session 2026-05-06 — Decision: adopted with conditions.
> **DRI:** Rena (CTO) — **Date:** 2026-05-06

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [System Architecture](#2-system-architecture)
3. [Decision Tree Format](#3-decision-tree-format)
4. [Application Type Registry](#4-application-type-registry)
5. [AI Assistance Layer](#5-ai-assistance-layer)
6. [Learning Mechanism](#6-learning-mechanism)
7. [Admin Portal](#7-admin-portal)
8. [Integrations](#8-integrations)
9. [Crate & Module Layout](#9-crate--module-layout)
10. [Database Schema](#10-database-schema)
11. [API / MCP / CLI Contracts](#11-api--mcp--cli-contracts)
12. [Phased Rollout Plan](#12-phased-rollout-plan)
13. [Security & Threat Model](#13-security--threat-model)
14. [Testing Strategy](#14-testing-strategy)
15. [Open Questions](#15-open-questions)

---

## 1. Executive Summary

### Problem

The current intake pipeline (`intake_service.rs`) has a `create_plan()` method that accepts raw `name`, `goal`, `scope` strings — no structured questioning, no application-type awareness, no conditional branching. Every intake is unstructured agent-driven clarification:
- Inconsistent requirement quality across sessions
- No automated module/persona assignment from intake data
- Heavy AI token spend for every routine question
- No learning mechanism from past intakes

### Solution

A **structured deterministic intake engine** that:
- Is **95% rule-driven**: decision trees handle all defined application types with conditional branching
- Is **<5% AI-assisted**: AI handles edge cases, unknown paths, and user confusion
- Has a **built-in learning mechanism**: AI-asked questions not in the decision tree are saved; repeated occurrences trigger promotion to canonical form
- Is **admin-managed**: operators can view, add, merge, edit, and deprecate questions/forms via admin portal
- **Reduces token cost by ~95%** on routine intakes

### Council Decision Summary

| Element | Decision |
|---|---|
| Architecture | Deterministic intake engine (95%) with AI fallback (<5%) |
| Decision tree format | Data-driven JSON/YAML — not a DSL |
| Max tree depth | 5 levels |
| AI observation promotion | Requires admin + security review for module/capability-affecting questions |
| Phased rollout | 3 types → 5 more → remaining 7 (10 weeks total) |
| Crate placement | `intake-engine` (pure logic), `intake-store` in `control-store`, observations in `control-service` |
| Admin surface | Standard `control-api` routes |

---

## 2. System Architecture

### High-Level Flow

```text
                      ┌────────────────────┐
                      │    User / Agent     │
                      └────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │   Intake Session      │
                    │   (question session)  │
                    └────────┬─────────────┘
                             │
                    ┌────────▼─────────────┐
                    │  Decision Tree        │
                    │  Engine               │
                    │  (intake-engine)      │
                    │                       │
                    │  ┌─────────────────┐  │
                    │  │ Match app type   │  │
                    │  │ → Select tree    │  │
                    │  └─────────────────┘  │
                    │  ┌─────────────────┐  │
                    │  │ Walk tree        │  │
                    │  │ → Conditional    │  │
                    │  │   questions      │  │
                    │  └─────────────────┘  │
                    │  ┌─────────────────┐  │
                    │  │ Map responses    │  │
                    │  │ → Plan features  │  │
                    │  └─────────────────┘  │
                    └────────┬─────────────┘
                             │
                    ┌────────▼─────────────┐
                    │   AI Fallback?        │
                    │   (<5% of cases)      │
                    │                       │
                    │  ┌─────────────────┐  │
                    │  │ AI asked question│  │
                    │  │ not in tree      │  │
                    │  │ → Engine holds   │  │
                    │  │   context        │  │
                    │  └─────────────────┘  │
                    │  ┌─────────────────┐  │
                    │  │ Record as        │  │
                    │  │ observation      │  │
                    │  │ (for learning)   │  │
                    │  └─────────────────┘  │
                    └────────┬─────────────┘
                             │
                             ▼
                    ┌──────────────────────┐
                    │   Session Complete    │
                    │   → Generate Plan     │
                    │   → Intake Service    │
                    └──────────────────────┘
```

### Layer Architecture

```text
┌──────────────────────────────────────────────────────────┐
│                      User / Agent                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │   API    │  │   MCP    │  │   CLI    │               │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘               │
│       │              │              │                      │
│       └──────────────┼──────────────┘                      │
│                      ▼                                     │
│           ┌──────────────────────┐                        │
│           │   control-service    │                        │
│           │  (intake_service.rs  │                        │
│           │   observation_svc)   │                        │
│           └────┬───────────┬─────┘                        │
│                │           │                                 │
│         ┌──────▼──┐  ┌────▼────────┐                     │
│         │intake-  │  │intake-store │                     │
│         │engine   │  │(in control- │                     │
│         │(pure)   │  │ store)      │                     │
│         │         │  │             │                     │
│         │ Tree    │  │ Templates   │                     │
│         │ eval    │  │ Sessions    │                     │
│         │ Path    │  │ Responses   │                     │
│         │ mapping │  │ Observations│                     │
│         └─────────┘  └──────┬──────┘                     │
│                              │                              │
│                              ▼                              │
│                    ┌──────────────────┐                    │
│                    │   PostgreSQL      │                    │
│                    │   (3 tables)      │                    │
│                    └──────────────────┘                    │
└──────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | IO |
|---|---|---|
| `intake-engine` | Decision tree parsing, path evaluation, condition matching, feature mapping | Pure — no IO |
| `control-service::intake_service` | Session orchestration, plan creation from intake results | Orchestrates |
| `control-service::intake_observation_service` | Recording AI observations, promotion thresholds | Minimal IO |
| `control-store::intake` | Persisting templates, sessions, responses, observations | Persistence |
| `control-api::intake` | REST routes for form rendering, session management, admin | Thin dispatch |
| `agent-mcp::intake` | MCP tools for agent-driven intake | Thin dispatch |

---

## 3. Decision Tree Format

### File Type

Decision trees are stored as **JSON files** (human-editable, schema-validated, versionable).

### Schema

```jsonc
{
  "$schema": "https://realmforge.dev/schemas/decision-tree-v1.json",
  "tree_id": "tree-web-app",
  "name": "Web Application Intake",
  "version": "1.0.0",
  "applies_to": ["static-site", "rest-api", "web-app"],
  "description": "Decision tree for web-based application types",
  "max_depth": 5,

  "questions": [
    {
      "question_id": "hosting_type",
      "type": "single_choice",
      "text": "Where will this application be hosted?",
      "options": [
        { "value": "cloud", "label": "Cloud (AWS/Azure/GCP)" },
        { "value": "on-prem", "label": "On-Premise / Self-Hosted" },
        { "value": "edge", "label": "Edge / CDN" },
        { "value": "not-sure", "label": "I don't know" }
      ],
      "order": 1,
      "required": true,
      "ai_assist_prompt": "The user needs help choosing a hosting provider based on their requirements.",
      "ai_fallback_eligible": true
    },
    {
      "question_id": "auth_required",
      "type": "boolean",
      "text": "Does this application require user authentication?",
      "order": 2,
      "required": true,
      "ai_assist_prompt": "Explain what authentication means and help them decide if they need it.",
      "ai_fallback_eligible": true,

      "conditions": {
        "if": { "equals": ["question:hosting_type", "not-sure"] },
        "then_show_ai_banner": true
      }
    },
    {
      "question_id": "auth_type",
      "type": "multiple_choice",
      "text": "What authentication methods are needed?",
      "options": [
        { "value": "email-password", "label": "Email + Password" },
        { "value": "sso-saml", "label": "SSO (SAML/OIDC)" },
        { "value": "google", "label": "Google Login" },
        { "value": "apple", "label": "Apple Login" },
        { "value": "github", "label": "GitHub Login" },
        { "value": "magic-link", "label": "Magic Link / Passwordless" }
      ],
      "order": 3,
      "required": true,
      "min_selections": 1,
      "max_selections": 5,

      "conditions": {
        "if": { "equals": ["question:auth_required", true] },
        "then": null,
        "else_hide": true
      }
    },
    {
      "question_id": "database_type",
      "type": "single_choice",
      "text": "What database does your application need?",
      "options": [
        { "value": "pg", "label": "PostgreSQL" },
        { "value": "mysql", "label": "MySQL / MariaDB" },
        { "value": "sqlite", "label": "SQLite (embedded)" },
        { "value": "mongodb", "label": "MongoDB" },
        { "value": "redis", "label": "Redis (caching)" },
        { "value": "none", "label": "No database needed" }
      ],
      "order": 4,
      "required": true
    }
  ],

  "feature_mappings": [
    {
      "question_id": "auth_required",
      "value": true,
      "modules": ["auth-module"],
      "personas": ["backend-auth", "frontend-auth"],
      "skills": ["backend", "frontend"]
    },
    {
      "question_id": "auth_type",
      "value": "sso-saml",
      "modules": ["auth-module-saml"],
      "personas": ["backend-auth"],
      "skills": ["backend"]
    },
    {
      "question_id": "auth_type",
      "value": "google",
      "modules": ["auth-module-google", "auth-module-oauth"],
      "personas": ["backend-auth"],
      "skills": ["backend"]
    },
    {
      "question_id": "database_type",
      "value": "pg",
      "modules": ["database-module-pg"],
      "personas": ["backend-data"],
      "skills": ["backend", "data-engineer"]
    }
  ],

  "meta_mappings": {
    // Cross-cutting concerns that activate based on question combinations
    "public_facing": {
      "condition": {
        "and": [
          { "equals": ["question:auth_required", true] },
          { "equals": ["question:hosting_type", "cloud"] }
        ]
      },
      "modules": ["security-module-rate-limit", "security-module-waf"],
      "personas": ["security-review"],
      "skills": ["security-architect"]
    }
  },

  "plan_stage_handoff": {
    "map_requirements": true,
    "generate_constraints": true,
    // Auto-populate fields in the Plan
    "plan_overrides": {
      "generate_name_from": ["app_name", "description"],
      "default_scope_template": "web_application_scope.md"
    }
  }
}
```

### Question Types

| Type | Description | Options | Conditions |
|---|---|---|---|
| `boolean` | Yes/No | None | `[true, false]` |
| `single_choice` | Pick one | Required | Each option |
| `multiple_choice` | Pick N | Required, min/max | Each option |
| `text` | Free text | None | Length, regex |
| `number` | Numeric input | Min/max | Range |
| `ai_assisted_text` | Free text with AI suggestions | Optional suggestions | AI fallback |

### Condition Operators

| Operator | Description | Example |
|---|---|---|
| `equals` | Value matches expected | `["question:auth_required", true]` |
| `in` | Value is in list | `["question:hosting_type", ["cloud", "edge"]]` |
| `not_equals` | Value does not match | `["question:db_type", "none"]` |
| `gt` / `gte` | Greater than | `["question:expected_users", 10000]` |
| `lt` / `lte` | Less than | `["question:budget", 50000]` |
| `exists` | Question was answered | `["question:auth_type"]` |
| `and` | All conditions must pass | Array of conditions |
| `or` | Any condition passes | Array of conditions |

### File Location

```text
catalog/intake-trees/
├── tree-static-site.json
├── tree-rest-api.json
├── tree-web-app.json
├── tree-mobile-app.json
├── tree-desktop-app.json
├── tree-cli-tool.json
├── tree-dashboard.json
├── tree-ml-pipeline.json
├── tree-distributed-system.json
└── tree-os.json       (longest, most detailed)

.realmforge/intake/observations/    ← AI observations (ignored by git)
  └── intake_ai_observations.jsonl  ← Append-only observation log
```

---

## 4. Application Type Registry

### Initial 15 Application Types

| # | Type | Decision Tree | Complexity | Questions | Modules |
|---|---|---|---|---|---|
| 1 | Static Site | `tree-static-site.json` | Low | 5-7 | Hosting, Domain, Analytics |
| 2 | REST API | `tree-rest-api.json` | Medium | 10-14 | Auth, Database, Rate Limiting, Logging |
| 3 | Web App | `tree-web-app.json` | High | 15-22 | Auth, Database, Frontend, Backend, Hosting, CI/CD |
| 4 | Mobile App | `tree-mobile-app.json` | High | 12-18 | Auth, Push, Offline, Store Deploy |
| 5 | Desktop App | `tree-desktop-app.json` | Medium | 8-12 | Auth, Update, Bundling |
| 6 | CLI Tool | `tree-cli-tool.json` | Low-Medium | 6-10 | Args, Config, Output |
| 7 | Dashboard | `tree-dashboard.json` | Medium | 8-14 | Auth, Data Source, Visualization |
| 8 | ML Pipeline | `tree-ml-pipeline.json` | High | 12-20 | Data, Training, Deploy, Monitoring |
| 9 | ETL Pipeline | `tree-etl-pipeline.json` | Medium | 10-15 | Source, Transform, Sink, Schedule |
| 10 | Game | `tree-game.json` | High | 14-22 | Engine, Multiplayer, Assets, Store |
| 11 | Distributed System | `tree-distributed-system.json` | Very High | 18-30 | Service Discovery, Messaging, Consensus, Monitoring |
| 12 | Embedded System | `tree-embedded-system.json` | Very High | 15-25 | RTOS, GPIO, Protocols, Memory |
| 13 | Operating System | `tree-os.json` | Extreme | 25-50 | Kernel, Scheduler, Drivers, FS, Network Stack |
| 14 | Database | `tree-database.json` | Very High | 18-28 | Storage Engine, Query, Replication, Indexing |
| 15 | Platform / SaaS | `tree-platform-saas.json` | Extreme | 22-40 | Multi-Tenant, Billing, RBAC, API Gateway |

### Registry Data Model

```jsonc
{
  "type_id": "web-app",
  "name": "Web Application",
  "description": "A web application with frontend, backend, and database",
  "icon": "globe",
  "complexity": "high",
  "default_tree": "tree-web-app.json",
  "tree_version": "1.0.0",
  "tags": ["web", "fullstack", "crud"],
  "alternative_names": ["web app", "website with backend", "fullstack app"],
  "ai_help_keywords": ["web", "website", "browser", "frontend", "backend"],
  "requires_modules": ["frontend-module", "backend-module"],
  "recommended_skills": ["frontend", "backend"],
  "created_at": "2026-05-06",
  "last_updated": "2026-05-06",
  "status": "active"
}
```

### Type Matching Logic (deterministic + AI fallback)

1. **Direct match**: User selects type from registry → load decision tree
2. **Keyword match**: User describes app → match against `ai_help_keywords` + `alternative_names`
3. **AI suggest**: If no match found → AI suggests top 3 types → user confirms
4. **Unknown type**: If AI can't map → full AI-driven intake → recorded as new type candidate

---

## 5. AI Assistance Layer

### When AI Triggers

| Scenario | Trigger | AI Role | Counted as |
|---|---|---|---|
| User doesn't know app type | Selects "I don't know" | Suggests top 3 types from description | AI assistance |
| Question not in decision tree | User asks something unexpected | Answers question, records observation | AI observation |
| User confused by question | Clicks "Help me answer" | Explains question in context | AI assistance |
| No tree matches type | Description doesn't match any tree | Handles full intake, records for new tree | AI new candidate |
| Complex cross-cutting concern | Combination not in `meta_mappings` | Resolves and suggests new mapping | AI observation |
| User wants to skip form | "Talk to agent" button | Full AI-driven intake | AI full intake |

### MCP Tool for AI Assistance

```jsonc
// Tool: intake_assist
{
  "name": "intake_assist",
  "description": "Handle edge cases during intake — unknown app types, questions not in decision tree, cross-cutting concerns",
  "input_schema": {
    "type": "object",
    "required": ["session_id", "trigger_reason", "context"],
    "properties": {
      "session_id": { "type": "string", "description": "Active intake session ID" },
      "trigger_reason": {
        "type": "string",
        "enum": [
          "unknown_app_type",
          "unhandled_question",
          "complex_cross_cutting",
          "user_confused",
          "new_type_candidate",
          "user_requested_agent"
        ]
      },
      "context": {
        "type": "object",
        "description": "Current intake state — answered questions, partial form state, user description"
      },
      "user_input": {
        "type": "string",
        "description": "The raw user input that triggered this assistance request"
      }
    }
  },
  "output": {
    "type": "object",
    "properties": {
      "action": {
        "type": "string",
        "enum": [
          "type_suggestion",     // Suggest application types
          "answer_question",     // Answer a user question
          "tree_proposal",       // Propose a new decision tree
          "mapping_suggestion",  // Suggest a new feature mapping
          "full_intake"          // Complete AI-driven intake
        ]
      },
      "suggestions": { "type": "array" },
      "recorded_as_observation": { "type": "boolean" },
      "observation_id": { "type": "string" }
    }
  }
}
```

### AI Integration Architecture

```text
┌─────────────────────┐
│   intake_service    │
│   (rust)            │
│                     │
│   ┌─────────────┐   │
│   │ Tree Engine  │───┼── Local eval
│   └──────┬──────┘   │
│          │           │
│          ▼           │
│   ┌─────────────┐   │
│   │ AI Boundary  │───┼──→ agent-mcp::intake_assist
│   │ Check        │   │         │
│   │ (is this     │   │         ▼
│   │  in the      │   │   ┌────────────┐
│   │  tree?)      │   │   │  AI Model  │
│   └─────────────┘   │   └────────────┘
└─────────────────────┘
          │
          ▼
┌─────────────────────┐
│  intake_observation │
│  _service           │
│                     │
│  Records:           │
│  - question text    │
│  - user intent      │
│  - context          │
│  - frequency count  │
│  - session_id       │
│  - source (AI/human)│
└─────────────────────┘
```

### AI Response Contract

The AI response must be a **structured JSON object** that can be parsed by `intake-engine`:

```jsonc
{
  "intake_response": {
    "type": "suggested_tree_update",
    "observation": "Multiple users asked about GraphQL support during REST API intake",
    "proposed_question": {
      "question_id": "api_type",
      "type": "single_choice",
      "text": "What API type does your application expose?",
      "options": [
        { "value": "rest", "label": "REST/JSON" },
        { "value": "graphql", "label": "GraphQL" },
        { "value": "grpc", "label": "gRPC" },
        { "value": "websocket", "label": "WebSocket" }
      ]
    },
    "proposed_feature_mapping": {
      "condition": { "equals": ["question:api_type", "graphql"] },
      "modules": ["api-module-graphql"],
      "personas": ["backend-api"],
      "skills": ["backend"]
    },
    "confidence": 0.85,
    "session_context": {
      "tree_id": "tree-rest-api",
      "answers": {
        "auth_required": true,
        "hosting_type": "cloud"
      }
    }
  }
}
```

---

## 6. Learning Mechanism

### Observation Collection

Every time AI handles a question not in the decision tree, the interaction is recorded:

```jsonc
{
  "observation_id": "obs_abc123",
  "session_id": "session_xyz789",
  "tree_id": "tree-web-app",
  "user_intent": "I need WebSocket support for real-time chat",
  "ai_question": "What real-time communication protocol do you need?",
  "user_response": "WebSocket for chat, SSE for notifications",
  "context": {
    "app_type": "web-app",
    "answers_so_far": { "auth_required": true, "hosting_type": "cloud", "database_type": "pg" }
  },
  "frequency_count": 1,
  "proposed_form_question": {
    "type": "single_choice",
    "text": "What real-time communication does your app need?",
    "options": [
      { "value": "websocket", "label": "WebSocket (bidirectional)" },
      { "value": "sse", "label": "Server-Sent Events (server→client)" },
      { "value": "polling", "label": "Polling" },
      { "value": "none", "label": "No real-time needed" }
    ]
  },
  "recorded_at": "2026-05-06T12:00:00Z",
  "source": "ai",
  "status": "observation"  // promotion states: observation → candidate → proposed → canonical
}
```

### Promotion Threshold

```text
Observation Count   →   Action
──────────────────────────────────────────
1                    →   Logged as observation
3                    →   Flagged as "recurring — review for promotion"
5                    →   Admin notification: "Consider adding to tree <name>"
10                   →   Auto-promote to candidate (requires admin approval to go canonical)
```

### Promotion States

```text
observation  →  candidate  →  proposed  →  canonical
    ↓               ↓            ↓            ↓
  Raw AI      Reviewed by    Draft form    Active in
  record      operator      ready for     decision tree
                             approval
```

### AI-Proposed New Tree

When a user describes an app that doesn't match any existing type (e.g., first request is for a distributed system when only 3 simple types exist):

1. AI identifies "no matching tree" → records as `new_type_candidate`
2. AI creates a **proposed decision tree** in `.realmforge/intake/proposed/`
3. Admin receives notification: "New application type proposed: distributed-system"
4. Admin can: approve → promote to canonical | edit → modify tree | reject → archive
5. Next user requesting "distributed system" sees the new tree

### Observation Storage

```text
DB table: intake_observations
├── postgres (canonical observations)
└── .realmforge/intake/observations/intake_ai_observations.jsonl (local log)

File format (JSONL — one JSON object per line):
{"observation_id":"obs_001","session_id":"ses_abc",...}
{"observation_id":"obs_002","session_id":"ses_def",...}
```

---

## 7. Admin Portal

### View: Intake Overview

```text
┌──────────────────────────────────────────────────────────────────────┐
│  ⚔️ RealmForge Admin — Intake System                                │
├──────────────────────────────────────────────────────────────────────┤
│  [Dashboard] [Intake] [Trees] [Observations] [Application Types]    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  📊 Intake Overview                                                  │
│  ┌─────────┬─────────┬──────────┬───────────┬────────────┐         │
│  │ Today   │ This Wk │ This Mo  │ Avg Time  │ AI Assist  │         │
│  │ 12      │ 47      │ 203      │ 4m 32s    │ 8 (3.9%)  │         │
│  ├─────────┼─────────┼──────────┼───────────┼─────────────────────┤ │
│  │ Web App │ API     │ Mobile   │ Desktop   │ Unknown → AI        │ │
│  │ 5       │ 3       │ 2        │ 1         │ 1                   │ │
│  └─────────┴─────────┴──────────┴───────────┴─────────────────────┘ │
│                                                                      │
│  🔔 Pending Observations (3)                                        │
│  ┌──────────────────────────────────────────────────────┬──────────┐ │
│  │ Question                                           │ Count    │ │
│  ├──────────────────────────────────────────────────────┼──────────┤ │
│  │ "Do you need WebSocket support?" (tree: web-app)    │ 7 ⭐     │ │
│  │ "What CI/CD provider?" (tree: rest-api)             │ 4 ⭐     │ │
│  │ "Need rate limiting?" (tree: platform-saas)         │ 2        │ │
│  └──────────────────────────────────────────────────────┴──────────┘ │
│                                                                      │
│  🆕 Proposed New Type (1)                                           │
│  ┌──────────────────────────────────────────────────────────────────┐│
│  │ Distributed System — Proposed by AI on 2026-05-05              ││
│  │ "Service mesh, consensus protocol, distributed DB, circuit    ││
│  │  breaker..."                                                    ││
│  │ [Review] [Approve] [Edit] [Reject]                             ││
│  └──────────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────────┘
```

### View: Tree Editor

```text
┌──────────────────────────────────────────────────────────────────────┐
│  Decision Tree Editor — tree-web-app v1.2.0                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Questions (15)                            Canvas Preview            │
│  ┌────────────────────────────────────┐  ┌────────────────────────┐ │
│  │ 📌 1. What is your app name?      │  │                        │ │
│  │ 📝 2. Describe your app...        │  │  [App Name]            │ │
│  │ 📌 3. Hosting type?               │  │    ↓                   │ │
│  │     ├─ ☐ Cloud                    │  │  [Description]         │ │
│  │     ├─ ☐ On-Prem                  │  │    ↓                   │ │
│  │     └─ ☐ Edge                     │  │  [Hosting Type]        │ │
│  │ 📌 4. Auth required?  ──if yes──►│  │    ↓                   │ │
│  │     └─ 5. Auth type?              │  │  [Auth Required?]      │ │
│  │         ├─ Email/PW               │  │    ├─yes→ [Auth Type]  │ │
│  │         ├─ SSO                    │  │    └─no→  [DB Type]    │ │
│  │         └─ Google                 │  │    ↓                   │ │
│  │ 📌 6. DB Type?                    │  │  [DB Type]             │ │
│  │ ...                               │  │                        │ │
│  └────────────────────────────────────┘  └────────────────────────┘ │
│                                                                      │
│  [Add Question] [Edit] [Reorder] [Test Tree] [Save] [Publish]      │
└──────────────────────────────────────────────────────────────────────┘
```

### Admin API Routes

| Method | Path | Description |
|---|---|---|
| `GET` | `/v1/admin/intake/trees` | List all decision trees |
| `GET` | `/v1/admin/intake/trees/:id` | Get tree with version |
| `PUT` | `/v1/admin/intake/trees/:id` | Update tree |
| `POST` | `/v1/admin/intake/trees` | Create new tree |
| `POST` | `/v1/admin/intake/trees/:id/publish` | Publish new version |
| `DELETE` | `/v1/admin/intake/trees/:id` | Deprecate tree |
| `GET` | `/v1/admin/intake/observations` | List observations (paginated, sort by count) |
| `GET` | `/v1/admin/intake/observations/:id` | Get observation with context |
| `POST` | `/v1/admin/intake/observations/:id/promote` | Promote observation to canonical |
| `POST` | `/v1/admin/intake/observations/:id/dismiss` | Dismiss observation |
| `GET` | `/v1/admin/intake/types` | List application types |
| `POST` | `/v1/admin/intake/types` | Create new app type |
| `PUT` | `/v1/admin/intake/types/:id` | Update app type |
| `GET` | `/v1/admin/intake/proposed` | List AI-proposed tree candidates |
| `POST` | `/v1/admin/intake/proposed/:id/approve` | Approve → promote to canonical |
| `POST` | `/v1/admin/intake/proposed/:id/edit` | Edit before approval |

---

## 8. Integrations

### Integration with control-service::intake_service

The existing `IntakeService` gets extended with a **new intake flow**:

```rust
// Current (Stage 1):
pub async fn create_plan(name, goal, scope, owner) -> Result<Plan>

// New structured flow:
pub async fn create_intake_session(app_type, raw_input) -> Result<Session>
pub async fn intake_next_question(session_id) -> Result<Question>
pub async fn intake_answer(session_id, question_id, answer) -> Result<AnswerResult>
pub async fn intake_complete(session_id) -> Result<Plan>
pub async fn intake_ai_assist(session_id, reason, context) -> Result<AssistResult>
```

### Integration with Plan Pipeline

```text
New Intake Flow:                      Old Intake Flow:
┌────────────────────┐                ┌────────────────────┐
│ Session Created     │                │ create_plan(name,  │
│ (type unknown)      │                │   goal, scope)     │
└────────┬───────────┘                └────────┬───────────┘
         │                                      │
         ▼                                      ▼
┌────────────────────┐                ┌────────────────────┐
│ Walk Decision Tree  │               │ 4 other stages     │
│ (conditional)       │               │ (refinement, arch, │
└────────┬───────────┘               │  decom, packets)   │
         │                             └────────────────────┘
         ▼
┌────────────────────┐
│ Generate Plan with │
│ pre-populated:     │
│ - name, goal       │
│ - constraints      │
│ - core_areas       │
│ - modules          │
│ - personas/skills  │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Enter Stage 2      │
│ (Refinement)       │
│ with structured    │
│ starting state     │
└────────────────────┘
```

### Integration with Work Path System

When the intake completes, the feature mappings generate:

```rust
pub async fn intake_to_work_paths(plan_id, feature_mappings) -> Result<Vec<WorkPathNode>>
```

This creates nodes in the work path graph for each selected module/persona.

### Integration with MCP

New MCP tool definitions (to be added in Phase 9 or alongside):

| Tool | Description |
|---|---|
| `intake_create_session` | Start a new intake session |
| `intake_get_question` | Get the next question for a session |
| `intake_submit_answer` | Submit an answer |
| `intake_get_summary` | Get current intake summary |
| `intake_complete` | Complete intake and generate plan |
| `intake_assist` | AI assistance for edge cases |

---

## 9. Crate & Module Layout

### New `intake-engine` Crate

```
crates/intake-engine/
├── Cargo.toml
└── src/
    ├── lib.rs                // Module declarations, re-exports
    ├── error.rs              // IntakeEngineError enum
    ├── tree.rs               // Decision tree definition types
    ├── engine.rs             // Decision tree execution engine
    │     evaluate_node()
    │     walk_tree()
    │     apply_conditions()
    │     resolve_path()
    ├── condition.rs          // Condition operators (equals, in, and, or, etc.)
    ├── mapper.rs             // Feature → module/persona/skill mapping
    └── validate.rs           // Tree schema validation
```

**Dependencies:** `serde`, `serde_json`, `thiserror` — **NO** RealmForge crate dependencies. Pure engine.

### Extensions to Existing Crates

**control-store** (new module):
```
crates/control-store/src/
  └── intake/
      ├── mod.rs              // Module declarations
      ├── templates.rs        // Decision tree template persistence
      ├── sessions.rs         // Intake session persistence
      ├── observations.rs     // AI observation persistence
      └── models.rs           // Intake DB types (tables, queries)
```

**control-service** (extend):
```
crates/control-service/src/
  ├── intake_service.rs       // EXTEND — Add structured intake methods
  └── intake_observation_service.rs  // NEW — Observation recording + promotion logic
```

**control-api** (new routes):
```
crates/control-api/src/
  └── routes/
      ├── mod.rs              // EXTEND — Add intake routes
      └── intake.rs           // NEW — Intake + admin endpoints
```

**agent-mcp** (new tools):
```
crates/agent-mcp/src/
  └── tools/
      └── intake.rs           // NEW — MCP intake tools
```

**operator-cli** (new commands):
```
operator-cli/src/
  └── commands/
      └── intake.rs           // NEW — Intake CLI commands
```

### Dependency Graph

```text
operator-cli
  └── control-service
        ├── authority-domain        (pure types)
        ├── intake-engine  ← NEW    (pure decision tree engine)
        ├── control-store
        │     └── intake-store      (persistence module)
        └── ... (existing)

control-api
  └── control-service

agent-mcp
  └── control-service

intake-engine
  └── (no RealmForge deps — serde, serde_json)
```

---

## 10. Database Schema

### Table: `intake_trees`

```sql
CREATE TABLE intake_trees (
    tree_id        VARCHAR(64) PRIMARY KEY,
    name           VARCHAR(255) NOT NULL,
    version        VARCHAR(16) NOT NULL DEFAULT '1.0.0',
    definition     JSONB NOT NULL,           -- Full decision tree JSON
    applies_to     TEXT[] NOT NULL,           -- Array of app type IDs
    status         VARCHAR(16) NOT NULL DEFAULT 'active',
                   -- active, draft, deprecated
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at   TIMESTAMPTZ,
    published_by   VARCHAR(128)
);

CREATE INDEX idx_intake_trees_applies ON intake_trees USING GIN (applies_to);
```

### Table: `intake_sessions`

```sql
CREATE TABLE intake_sessions (
    session_id     VARCHAR(64) PRIMARY KEY,
    tree_id        VARCHAR(64) REFERENCES intake_trees(tree_id),
    app_type_id    VARCHAR(64),
    status         VARCHAR(16) NOT NULL DEFAULT 'in_progress',
                   -- in_progress, completed, abandoned, ai_assisted
    answers        JSONB NOT NULL DEFAULT '{}',  -- {question_id: answer_value, ...}
    plan_id        VARCHAR(64),                   -- Reference to intake_plans
    owner          VARCHAR(128) NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at   TIMESTAMPTZ,
    duration_seconds INTEGER,
    ai_assist_count INTEGER NOT NULL DEFAULT 0,
    ai_full_intake  BOOLEAN NOT NULL DEFAULT false,
    source         VARCHAR(16) NOT NULL DEFAULT 'api',
                   -- api, mcp, cli, admin, ai
    metadata       JSONB DEFAULT '{}'
);

CREATE INDEX idx_intake_sessions_status ON intake_sessions(status);
CREATE INDEX idx_intake_sessions_owner ON intake_sessions(owner);
CREATE INDEX idx_intake_sessions_created ON intake_sessions(created_at DESC);
```

### Table: `intake_observations`

```sql
CREATE TABLE intake_observations (
    observation_id     VARCHAR(64) PRIMARY KEY,
    session_id         VARCHAR(64) REFERENCES intake_sessions(session_id),
    tree_id            VARCHAR(64) REFERENCES intake_trees(tree_id),

    -- The observation data
    question_text      TEXT NOT NULL,
    user_intent        TEXT,
    context            JSONB NOT NULL DEFAULT '{}',   -- Session state at time of observation
    proposed_question  JSONB,                          -- AI-suggested form question
    proposed_mapping   JSONB,                          -- AI-suggested feature mapping

    -- Frequency tracking
    frequency_count    INTEGER NOT NULL DEFAULT 1,
    first_seen         TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Promotion state
    status             VARCHAR(16) NOT NULL DEFAULT 'observation',
                       -- observation, candidate, proposed, canonical, dismissed

    -- Canonical linkage (if promoted)
    target_tree_id     VARCHAR(64),
    target_form_id     VARCHAR(64),

    -- Metadata
    source             VARCHAR(16) NOT NULL DEFAULT 'ai',
                       -- ai, manual_admin, imported
    recorded_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at        TIMESTAMPTZ,
    resolved_by        VARCHAR(128),
    resolution_note    TEXT
);

CREATE INDEX idx_intake_obs_status ON intake_observations(status);
CREATE INDEX idx_intake_obs_freq ON intake_observations(frequency_count DESC);
CREATE INDEX idx_intake_obs_tree ON intake_observations(tree_id);
CREATE INDEX idx_intake_obs_source ON intake_observations(source);
```

### Table: `intake_app_types`

```sql
CREATE TABLE intake_app_types (
    type_id          VARCHAR(64) PRIMARY KEY,
    name             VARCHAR(255) NOT NULL,
    description      TEXT NOT NULL,
    icon             VARCHAR(64),
    complexity       VARCHAR(16) NOT NULL DEFAULT 'medium',
                     -- low, medium, high, very_high, extreme
    default_tree_id  VARCHAR(64) REFERENCES intake_trees(tree_id),

    -- Metadata
    tags             TEXT[] DEFAULT '{}',
    alternative_names TEXT[] DEFAULT '{}',
    ai_help_keywords  TEXT[] DEFAULT '{}',
    requires_modules  TEXT[] DEFAULT '{}',
    recommended_skills TEXT[] DEFAULT '{}',

    status           VARCHAR(16) NOT NULL DEFAULT 'active',
                     -- active, draft, deprecated
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_intake_types_status ON intake_app_types(status);
CREATE INDEX idx_intake_types_tags ON intake_app_types USING GIN (tags);
CREATE INDEX idx_intake_types_alt ON intake_app_types USING GIN (alternative_names);
CREATE INDEX idx_intake_types_keywords ON intake_app_types USING GIN (ai_help_keywords);
```

### Migration

New migration: `013_intake_system.sql` — Creates all 4 tables above.

---

## 11. API / MCP / CLI Contracts

### Public REST API

| Method | Path | Description |
|---|---|---|
| `POST` | `/v1/intake/session` | Start new intake session (with optional app_type hint) |
| `GET` | `/v1/intake/session/:id` | Get session state and current question |
| `POST` | `/v1/intake/session/:id/answer` | Submit answer to current question |
| `GET` | `/v1/intake/session/:id/summary` | Get full summary of answers so far |
| `POST` | `/v1/intake/session/:id/complete` | Complete intake, generate Plan |
| `POST` | `/v1/intake/session/:id/abandon` | Abandon session |
| `GET` | `/v1/intake/types` | List available application types |
| `GET` | `/v1/intake/types/:id` | Get type details |
| `POST` | `/v1/intake/ai-assist` | Request AI assistance for a session |

### Example Flow (CLI)

```bash
# Start a web app intake
realmforge intake start --type web-app
→ Session ID: ses_abc123

# Get next question
realmforge intake question ses_abc123
→ Q1: What is your app name?

# Answer
realmforge intake answer ses_abc123 --question app_name --value "MyApp"
→ ✅ Recorded. Next question: Describe your application in one sentence.

# AI assist when stuck
realmforge intake assist ses_abc123 "I need authentication but I don't know SSO"
→ AI: SSO (Single Sign-On) allows users to log in once and access multiple apps.
  Based on your description, Email+Password with Google login is recommended.

# Complete
realmforge intake complete ses_abc123
→ ✅ Intake complete! Plan "plan_abc123" created.
  → 15 questions answered
  → 6 modules identified: auth, db, hosting, rate-limit, ci/cd, monitoring
  → 4 personas: backend, frontend, security-architect, devops
  → AI assistance used: 1 time
  → Plan entering 'Refinement' stage
```

### Example Flow (MCP — Agent-driven)

```
Agent detects user needs a web app with login.

mcp_intake_create_session({ app_type: "web-app" })
→ { session_id: "ses_001", first_question: {...} }

mcp_intake_submit_answer({ session: "ses_001", q: "hosting_type", a: "cloud" })
→ { next_question: { q: "auth_required", type: "boolean" } }

mcp_intake_submit_answer({ session: "ses_001", q: "auth_required", a: true })
→ { next_question: { q: "auth_type", type: "multiple_choice", options: [...] } }

mcp_intake_submit_answer({ session: "ses_001", q: "auth_type", a: ["sso-saml", "google"] })
→ { next_question: { q: "database_type", type: "single_choice", options: [...] } }

... (continues through tree)

mcp_intake_complete({ session: "ses_001" })
→ { plan_id: "plan_abc", modules: ["auth-module", "auth-module-saml", 
     "auth-module-google", "database-module-pg", "security-module-rate-limit"],
    personas: ["backend-auth", "backend-data", "security-review"],
    skills: ["backend", "data-engineer", "security-architect"] }
```

### Request/Response Types (Rust)

```rust
// ── Session ──
pub struct CreateSessionRequest {
    pub app_type: Option<String>,  // If known; if None, AI helps determine
    pub raw_description: Option<String>,
    pub owner: String,
}

pub struct SessionState {
    pub session_id: String,
    pub tree_id: Option<String>,
    pub app_type: Option<String>,
    pub status: SessionStatus,
    pub current_question: Option<Question>,
    pub questions_answered: u32,
    pub total_questions: u32,
    pub progress_pct: f32,
    pub ai_assist_count: u32,
}

// ── Questions ──
pub struct Question {
    pub question_id: String,
    pub question_type: QuestionType,
    pub text: String,
    pub options: Vec<QuestionOption>,
    pub required: bool,
    pub ai_assist_prompt: Option<String>,
}

pub enum QuestionType {
    Boolean,
    SingleChoice,
    MultipleChoice { min: u32, max: u32 },
    Text { max_length: u32, regex: Option<String> },
    Number { min: Option<f64>, max: Option<f64> },
    AiAssistedText,
}

pub struct QuestionOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

// ── Answers ──
pub struct AnswerSubmission {
    pub session_id: String,
    pub question_id: String,
    pub answer: AnswerValue,
}

pub enum AnswerValue {
    Boolean(bool),
    SingleChoice(String),
    MultipleChoice(Vec<String>),
    Text(String),
    Number(f64),
}

// ── Answer Result ──
pub struct AnswerResult {
    pub next_question: Option<Question>,
    pub session_complete: bool,
    pub affect_module_count: u32,
    pub affect_persona_count: u32,
    pub ai_assist_offered: bool,
}

// ── Completion ──
pub struct IntakeCompleteResult {
    pub session_id: String,
    pub plan_id: String,
    pub modules_identified: Vec<String>,
    pub personas_required: Vec<String>,
    pub skills_required: Vec<String>,
    pub ai_assist_count: u32,
    pub total_questions: u32,
    pub duration_seconds: u64,
}

// ── AI Assist ──
pub struct AiAssistRequest {
    pub session_id: String,
    pub trigger: AiAssistTrigger,
    pub user_input: String,
    pub context: serde_json::Value,
}

pub enum AiAssistTrigger {
    UnknownAppType,
    UnhandledQuestion,
    ComplexCrossCutting,
    UserConfused,
    NewTypeCandidate,
    UserRequestedAgent,
}

pub struct AiAssistResponse {
    pub action: AiAssistAction,
    pub suggestions: Vec<serde_json::Value>,
    pub observation_recorded: bool,
    pub observation_id: Option<String>,
}

pub enum AiAssistAction {
    TypeSuggestion,
    AnswerQuestion,
    TreeProposal,
    MappingSuggestion,
    FullIntake,
}
```

---

## 12. Phased Rollout Plan

### Phase 1: Foundation (Weeks 1-4)

**Goal:** Engine core + 3 application types + API + CLI

| Week | Deliverable | Files |
|---|---|---|
| 1 | Create `intake-engine` crate, decision tree schema, validation | `crates/intake-engine/src/**` |
| 2 | Implement tree engine (parse, walk, condition eval, mapping) | `engine.rs`, `condition.rs`, `mapper.rs` |
| 2 | DB migration + `intake-store` module | `013_intake_system.sql`, `crates/control-store/src/intake/**` |
| 3 | 3 decision trees: static-site, rest-api, web-app | `catalog/intake-trees/tree-*` |
| 3 | Extend `intake_service.rs` with structured flow | `crates/control-service/src/intake_service.rs` |
| 3 | API routes + CLI commands | `routes/intake.rs`, `commands/intake.rs` |
| 4 | Integration tests for all 3 types | `tests/intake_flow.rs` |
| 4 | Validation gate: engine compiles, 3 types work end-to-end | ✅ |

### Phase 2: AI Assistance + Observations (Weeks 5-7)

**Goal:** AI fallback, observation collection, learning mechanism

| Week | Deliverable | Files |
|---|---|---|
| 5 | `intake_observation_service.rs` + observation persistence | `control-service/src/intake_observation_service.rs` |
| 5 | MCP tools: `intake_create_session`, `intake_get_question`, `intake_submit_answer` | `crates/agent-mcp/src/tools/intake.rs` |
| 6 | MCP tool: `intake_assist` — AI fallback for edge cases | `crates/agent-mcp/src/tools/intake.rs` |
| 6 | Observation promotion logic (thresholds, alerts) | `intake_observation_service.rs` |
| 6 | 5 more trees: mobile-app, desktop-app, cli-tool, dashboard, ml-pipeline | `catalog/intake-trees/tree-*` |
| 7 | Integration tests: AI assist flow, observation recording | `tests/intake_ai_flow.rs` |
| 7 | Validation gate: 8 types work, AI fallback operational | ✅ |

### Phase 3: Admin Portal + Full Tree Set (Weeks 8-10)

**Goal:** Admin surface, full 15-type registry, production readiness

| Week | Deliverable | Files |
|---|---|---|
| 8 | Admin API routes (tree CRUD, observation management, type management) | `crates/control-api/src/routes/admin/intake.rs` |
| 8 | Tree editor visualization (read-only at first) | `frontend/src/pages/admin/intake/**` |
| 8 | Remaining 7 trees: etl-pipeline, game, distributed-system, embedded-system, os, database, platform-saas | `catalog/intake-trees/tree-*` |
| 9 | Tree editor (CRUD operations from admin UI) | Full frontend |
| 9 | AI-proposed tree approval workflow | `intake_observation_service.rs` |
| 10 | Full integration test suite (all 15 types, AI edge cases, admin operations) | `tests/intake_full_suite.rs` |
| 10 | Performance: <5 min intake time, <1% AI failure rate | ✅ |
| 10 | Security audit: threat model review, penetration tests | Security sign-off |

---

## 13. Security & Threat Model

### Identified Threats (Fatima Al-Hassan)

| Threat | Vector | Severity | Mitigation |
|---|---|---|---|
| **T1: Malicious Decision Tree** | Attacker uploads tree with privileged module mappings | 🔴 Critical | Tree validated against schema; module mappings have allowed-list; admin approval required for publish |
| **T2: AI Manipulation** | User tricks AI into suggesting unauthorized modules | 🟡 High | AI suggestions are non-binding; require explicit admin confirmation + policy check before becoming canonical |
| **T3: Session Hijacking** | Attacker steals session ID, modifies responses | 🟡 High | Session tokens with short TTL (15 min); session_id bound to actor identity |
| **T4: Observation Poisoning** | Attacker submits fake observations to trigger auto-promotion | 🟡 Medium | Observations rate-limited per session; auto-promotion threshold is 10 with admin review gate |
| **T5: Enumeration** | Attacker iterates questions to map system capabilities | 🟢 Low | Rate limiting; no sensitive info in question text; audit log for anomalous patterns |

### Required Security Controls

1. **Input validation**: All user responses sanitized (XSS, SQL injection)
2. **Permission checks**: Verify user can request specific application types
3. **Audit trail**: Log every intake session (who, what, when, AI-assisted?)
4. **Rate limiting**: AI assistance calls limited to 5 per session
5. **Approval workflow**: AI-proposed paths must be approved by admin before canonical
6. **Boundary enforcement**: AI can only suggest, not create, module/persona mappings

### Threat Model Diagram

```text
┌─────────────────┐     ┌──────────────────────┐
│   User / Agent   │────▶│  Intake Session API   │
└────────┬────────┘     └──────────┬───────────┘
         │                         │
         │ T3: Session Hijacking   │
         │ Mitigation: Short TTL   │
         │ + Identity Binding      │
         │                         │
         ▼                         ▼
┌─────────────────┐     ┌──────────────────────┐
│  Decision Tree   │     │  AI Assistance Layer  │
│  Engine          │     │                       │
│  (pure logic)    │     │  T2: AI Manipulation  │
│                  │     │  Mitigation:          │
│  T1: Tree Inj.   │     │  Non-binding suggest. │
│  Mitigation:     │     │  + Policy check       │
│  Schema validate │     └──────────┬───────────┘
│  + Allowed-list  │                │
└────────┬─────────┘                │
         │                          │
         ▼                          ▼
┌─────────────────┐     ┌──────────────────────┐
│   PostgreSQL     │     │  Observation Service │
│   (trees,        │     │                       │
│    sessions,     │     │  T4: Observation      │
│    observations) │     │  Poisoning            │
│                  │     │  Mitigation:          │
│                  │     │  Rate limit + Admin   │
└─────────────────┘     │  approval gate        │
                        └──────────────────────┘
```

---

## 14. Testing Strategy

### Unit Tests (intake-engine)

| Test | Description |
|---|---|
| `tree_parse_valid_json` | Valid tree JSON parses correctly |
| `tree_parse_invalid_json` | Invalid tree JSON returns typed error |
| `tree_validation_passes` | Tree with all required fields passes schema |
| `tree_validation_fails` | Tree with missing required fields fails |
| `condition_equals_true` | `equals` operator returns true when matching |
| `condition_equals_false` | `equals` operator returns false when not matching |
| `condition_in` | `in` operator works for array values |
| `condition_and_or` | Logical operators compose correctly |
| `walk_tree_simple_path` | Simple path through boolean questions works |
| `walk_tree_conditional_path` | Conditional questions show/hide correctly |
| `walk_tree_max_depth` | Tree with depth 5 resolves fully |
| `walk_tree_exceeds_depth` | Tree exceeding max depth returns error |
| `feature_mapping_single` | Single answer maps to correct module |
| `feature_mapping_multiple` | Multiple answers map to multiple modules |
| `feature_mapping_cross_cutting` | `meta_mappings` activate on combinations |
| `tree_version_compatibility` | Old tree version still parses with current schema |

### Integration Tests

| Test | Description |
|---|---|
| `intake_full_flow_web_app` | Full intake: start → 15 questions → complete → Plan |
| `intake_conditional_branching` | Auth=yes shows auth questions; auth=no hides them |
| `intake_ai_assist_unknown_type` | Unknown type triggers AI suggestion |
| `intake_ai_assist_question_not_in_tree` | User question not in tree → AI handles → observation recorded |
| `intake_session_persistence` | Save/resume works across restarts |
| `intake_concurrent_sessions` | Multiple sessions don't interfere |
| `intake_observation_promotion` | Observation with count 10 triggers notification |
| `intake_admin_tree_crud` | Create, read, update, deprecate tree |
| `intake_admin_observation_review` | List, promote, dismiss observations |
| `intake_admin_type_management` | Create, update, deprecate app type |
| `intake_ai_new_tree_proposal` | AI proposes new tree → admin approves → canonical |

### Performance Tests

| Test | Threshold |
|---|---|
| Tree evaluation time | <10ms per node |
| Full 15-question walk | <150ms |
| AI assist round-trip | <5s |
| Concurrent 100 sessions | <1s avg response |

### Acceptance Criteria

| ID | Criterion | Verification |
|---|---|---|
| `AC-INTAKE-001` | 100% of intakes produce valid Plan configurations | Integration tests |
| `AC-INTAKE-002` | No intakes succeed with missing required info | Integration tests |
| `AC-INTAKE-003` | Intake completes in <5 minutes (user time) | Performance tests |
| `AC-INTAKE-004` | <1% of intakes require AI assistance | Integration tests |
| `AC-INTAKE-005` | 100% of intakes logged with full trace | Audit log tests |
| `AC-INTAKE-006` | Observations with count ≥10 trigger notification | Integration tests |
| `AC-INTAKE-007` | Admin promotion gate blocks unauthorized changes | Security tests |
| `AC-INTAKE-008` | AI-proposed trees require admin approval before activation | Integration tests |
| `AC-INTAKE-009` | Tree validation rejects malformed definitions | Unit tests |
| `AC-INTAKE-010` | Sessions expire after TTL (15 min idle) | Integration tests |

---

## 15. Open Questions

The following questions remain open and should be resolved by Council or user approval before Phase 2 of rollout:

| Question | Type | Proposed Resolution |
|---|---|---|
| Should observations be deletable or only dismissible? | Design | Dismissible (soft delete with audit trail) — prevents data loss |
| What's the TTL for idle intake sessions? | Design | 15 minutes — aligns with session management |
| Should AI suggest module/persona mappings automatically, or only question text? | Security | Only question text — mapping changes require admin with security review |
| How do we handle tree versioning when a user is mid-session during a tree update? | Design | Session snapshot the tree at start; updates apply to new sessions |
| Should the admin tree editor be a visual node-based editor or a JSON/text editor? | UX | Phase 1: JSON editor. Phase 2: Visual editor (simplified). |
| Should AI-proposed trees be auto-tested before promotion? | Quality | Yes — run validation and at least 3 test sessions before promotion |

---

## Appendix A: Ubiquitous Language (Intake Domain)

| Term | Definition | ID Prefix |
|---|---|---|
| Intake Session | An active questioning session that collects requirements | `ses_` |
| Decision Tree | A structured set of conditional questions that guides intake | `tree_` |
| Application Type | A category of software application (e.g., "Web App", "OS") | `apptype_` |
| Question Node | A single question in the decision tree | `qn_` |
| Answer | A user's response to a question | — |
| Feature Mapping | The relationship between an answer and required modules/personas | — |
| Observation | An AI-recorded question not in the canonical decision tree | `obs_` |
| Promotion | The process of moving an observation to canonical form | — |
| AI Assist | An interaction where AI handles an edge case during intake | — |

## Appendix B: File Registry

| File ID | Path | Artifact Class | Module | Risk | Work Path | Allowed Grants | Required Tests |
|---|---|---|---|---|---|---|---|
| `FILE-CRATE-INTAKE-LIB` | `crates/intake-engine/src/lib.rs` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-LIB-001` |
| `FILE-CRATE-INTAKE-TREE` | `crates/intake-engine/src/tree.rs` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-TREE-001` |
| `FILE-CRATE-INTAKE-ENGINE` | `crates/intake-engine/src/engine.rs` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-ENGINE-001` |
| `FILE-CRATE-INTAKE-CONDITION` | `crates/intake-engine/src/condition.rs` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-CONDITION-001` |
| `FILE-CRATE-INTAKE-MAPPER` | `crates/intake-engine/src/mapper.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-MAPPER-001` |
| `FILE-CRATE-INTAKE-VALIDATE` | `crates/intake-engine/src/validate.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-VALIDATE-001` |
| `FILE-CRATE-INTAKE-ERROR` | `crates/intake-engine/src/error.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-ERROR-001` |
| `FILE-CRATE-STORE-INTAKE` | `crates/control-store/src/intake.rs` | `compiled_source` | Data Contracts | critical | `WP-CORE-001` | `SGL-BACKEND-STORE` | `TEST-STORE-INTAKE-001` |
| `FILE-INTAKE-OBSERVATION-SERVICE` | `crates/control-service/src/intake_observation_service.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-SERVICE` | `TEST-TBD` |
| `FILE-INTAKE-API-ROUTES` | `crates/control-api/src/routes/intake.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-API` | `TEST-TBD` |
| `FILE-INTAKE-MCP-TOOLS` | `crates/agent-mcp/src/tools/intake.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-MCP` | `TEST-TBD` |
| `FILE-INTAKE-CLI-CMDS` | `crates/operator-cli/src/commands/intake.rs` | `compiled_source` | Intake | low | `WP-CORE-001` | `SGL-BACKEND-CLI` | `TEST-TBD` |
| `FILE-DB-013` | `db/migrations/013_intake_decision_trees.sql` | `migration` | Data Contracts | critical | `WP-CORE-001` | `SGL-DATA-POSTGRES` | `TEST-MIGRATION-013` |
| `FILE-TREE-STATIC-SITE` | `catalog/intake-trees/tree-static-site.json` | `runtime_definition` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-TREE-001` |
| `FILE-TREE-WEB-APP` | `catalog/intake-trees/tree-web-app.json` | `runtime_definition` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-TREE-001` |
| `FILE-TREE-API-SERVICE` | `catalog/intake-trees/tree-api-service.json` | `runtime_definition` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-TREE-001` |

---

*"Intake is not a form — it's the first governance event. Every question asked, every answer given, every AI observation — these are the raw ore from which the entire build plan is smelted. Get the intake right, and everything downstream has a foundation."*
