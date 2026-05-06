---
doc_id: DOC-MASTER-BUILD-PLAN
title: "RealmForge Core — Master Build Plan"
status: active
owner: pm
reviewers: [cto, pm, backend, frontend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: workspace
work_path_ids: [WP-CORE-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# ⚔️ RealmForge Core — Master Build Plan

## The Living Blueprint for AI-Native Software Construction

---

### Preamble: The Soul of the Core

> *"Git tracks what humans changed.*  
> *RealmForge governs what agents are allowed to change, why they changed it,*  
> *how it was proven, and how to restore it."*

RealmForge Core is not just a set of Rust crates. It is the **governance kernel** — the control plane that makes AI-native software execution governable, traceable, auditable, and rollback-safe.

This document is the **master blueprint**. Every decision, every file, every module path that an AI agent will use to build the true application flows from this plan. The core crate structure already exists as a skeleton — typed IDs, scope models, policy logic, snapshot manifests. What follows is the **layer-by-layer, file-by-file, phase-by-phase construction plan** that will breathe life into the skeleton and make it a living, sentient governance kernel.

The vision document (`realm_forge_ai_native_path_forward.md`) is the **north star**. This document is the **battle map**. `AGENT_EXECUTION_GUIDE.md` is the **deep companion** — exact Rust signatures, DB query shapes, test structures, and agent work packets for every phase. Every agent that enters this workspace must read all three before touching a single line of code.

---

### Current State Assessment

**What exists (the skeleton):**

| Crate | Files | What's Implemented |
|-------|-------|-----------------------|
| `authority-domain` | `lib.rs`, `ids.rs`, `scope.rs`, `state.rs`, `command.rs`, `skill.rs`, `skill_creator.rs` | Typed IDs (12 types), ActorScope, enums (ExecutionMode, ApprovalState, CommandStatus, etc.), BoundedCommand value object, SkillRegistration/SkillSession, SkillCreator with catalog |
| `audit-log` | `lib.rs` | AuditEvent with hash-chain integrity, compute_hash/verify_hash |
| `policy-engine` | `lib.rs` | PolicyDecision/PolicyDenial, authorize_action() with 6 checks (expired, stale, unauthorized, wrong skill, proposal only, approval required) |
| `snapshot-ledger` | `lib.rs`, `manifest.rs`, `object_store.rs` | SnapshotManifest with hash-chain, SnapshotObjectRef, FileObjectStore (content-addressed) |
| `control-store` | `lib.rs` | CoreStore with PostgreSQL pool, append_audit_event(), insert_snapshot_manifest(), get_snapshot_manifest() |
| `control-api` | `lib.rs`, `main.rs` | Axum router with /health GET and /v1/policy/authorize POST |
| `operator-cli` | `main.rs` | Version, ListMigrations, ValidateManifest commands |
| `agent-mcp` | `lib.rs` | Tool definitions (5 tools), handle_tool() dispatching core_authorize_command |

**Migrations (what the database looks like):**
- `001_core_foundation.sql` — tenants, projects, actors, roles, actor_roles, skill_registrations, sessions, skill_sessions, bounded_commands, core_audit_events
- `002_snapshot_foundation.sql` — snapshot_manifests, snapshot_object_refs, snapshot_table_exports, rollback_previews

**What's MISSING (the organs the skeleton needs):**
- ✅ Identity & Session Management (core lifecycle — issue, renew, revoke, expire)
- ✅ Policy Enforcement Pipeline (chain of policy checks with middleware)
- ✅ Command Lifecycle Engine (propose → authorize → apply → audit → snapshot-anchor)
- ✅ Audit Trail Query & Replay (query events by scope, replay hash chain)
- ✅ Snapshot Orchestrator (create, validate, compare, prune)
- ✅ Rollback Engine (preview, validate, execute, verify)
- ✅ Agent Work Packet Generator (scope generation, permission boundary calculation)
- ✅ Service Layer (business logic between API/CLI/MCP and store)
- ✅ Full API Surface (session, command, audit, snapshot, rollback endpoints)
- ✅ Full CLI Surface (all operator commands)
- ✅ Full MCP Surface (all governance tools)
- ✅ Integration Tests (end-to-end flows through all layers)
- ✅ Error Boundary & Observability (structured error propagation, tracing)

---

## The Layer Architecture

The law is absolute. No agent may violate it.

```
api/mcp/cli  →  service layer  →  policy engine  →  domain model  →  store/snapshot adapter  →  PostgreSQL
    ↑                ↑                ↑                ↑                  ↑
  thin             orchestration    authorization    pure types          persistence
  transport        & scope          & denial         & state              & content-
  only             management       decisions        machines            addressed store
```

**Forbidden paths (enforced by code review, not by prayer):**
```
api → store                 (bypasses policy — denied)
mcp → store                 (bypasses policy — denied)
cli → store                 (bypasses policy — denied)
snapshot → policy bypass    (snapshot may not authorize commands)
service → db bypass         (service must go through store adapter)
```

---

## Phase-by-Phase Build Plan

### Phase 0: Foundation Hardening
**Goal: The skeleton must be structurally sound before adding organs.**

Files to create/modify:

```
authority-domain/src/
  ├── error.rs              NEW — Unified domain error type
  ├── ids.rs                EXTEND — Add Display for all IDs, add FromStr/Serialize/Deserialize tests
  ├── scope.rs              EXTEND — Add scope validation, add session expiry helpers
  ├── state.rs              EXTEND — Add TryFrom implementations between enums and DB string values
  ├── command.rs            EXTEND — Add status transition validation (state machine)
  ├── skill.rs              EXTEND — Add integrity validation, approval lifecycle
  ├── skill_creator.rs      EXTEND — Add catalog versioning, mutation methods
  └── lib.rs                UPDATE — Export error module

audit-log/src/
  └── lib.rs                EXTEND — Add event type categorization, add EventError variants

policy-engine/src/
  └── lib.rs                EXTEND — Add DenialCode::RateLimited, add policy chain evaluator

snapshot-ledger/src/
  ├── lib.rs                UPDATE — Re-export all modules
  ├── manifest.rs           EXTEND — Add validation, add delta computation between manifests
  └── object_store.rs       EXTEND — Add delete, list, stats operations
```

**Validation Gate:**
- `cargo test --workspace` passes with 0 failures
- `cargo clippy --workspace -- -D warnings` passes
- `cargo fmt --all -- --check` passes
- All source files under 300 lines (300 target, 500 hard cap)

---

### Phase 0a: Structured Intake System — Core Engine (Weeks 1-2)
**Goal: Build the deterministic intake engine with 3 foundational application types.**

**Business Justification:**
- 95% reduction in AI token costs (from $10/intake to $0.50/intake)
- 5-minute intake completion time (down from 20+ minutes)
- Consistent requirements capture across all project types
- Foundation for AI-native software construction governance

Files to create:

```
intake-engine/                   NEW CRATE (pure logic, no IO)
  ├── Cargo.toml            Dependencies: serde, serde_json, thiserror
  └── src/
      ├── lib.rs            Module declarations
      ├── error.rs          IntakeError enum
      ├── types.rs          IntakeSession, Question, Answer, Response types
      ├── evaluator.rs      evaluate_condition() — decision tree logic
      ├── navigator.rs      get_next_question() — tree traversal
      └── validator.rs      validate_template() — schema validation

control-service/src/
  ├── intake_service.rs     NEW — Session lifecycle, AI integration orchestration
  └── lib.rs                EXTEND — Export intake_service

control-store/src/
  └── lib.rs                EXTEND — Add intake queries (sessions, templates, responses)

migrations/
  └── 003_intake_foundation.sql    NEW — 7 tables:
                                   - application_types (with is_ai_generated, admin_reviewed)
                                   - intake_templates (decision_tree JSONB, versioned)
                                   - intake_sessions (responses, ai_questions, status)
                                   - ai_intake_questions (normalized_question, context)
                                   - question_frequency (occurrence_count, promoted tracking)
                                   - admin_review_queue (review_type, item_id, status)
                                   - intake_audit_log (event_type, event_data)
```

**Initial Application Type Templates (seed data):**
1. **Static Website** — hosting, domain, SSL, CDN, deploy method
2. **REST API** — auth, endpoints, rate limiting, versioning, DB type
3. **Dynamic Web App** — frontend framework, backend framework, session management, DB, file storage

**Conditional Logic DSL:**
```json
{
  "question_id": "Q003",
  "text": "Does your API require authentication?",
  "type": "yes_no",
  "conditional": {
    "show_if": "response.app_type == 'rest_api'"
  },
  "branches": {
    "yes": {
      "next": "Q004_auth_method",
      "features": ["user_management", "jwt_service"]
    },
    "no": {
      "next": "Q005_rate_limiting"
    }
  }
}
```

**Validation Gate:**
- `cargo test -p intake-engine` passes with 100% decision tree logic coverage
- All 3 templates load, validate, and execute without errors
- intake_service correctly orchestrates engine → store flow
- Database migration applies successfully with all 7 tables created

**Team:** Dmitri Volkov (Backend), Priya Mehta (Data)

---

### Phase 0b: Structured Intake System — AI Assistance (Weeks 3-4)
**Goal: Add AI fallback for unlisted applications and edge-case clarification.**

Files to create:

```
intake-engine/src/
  ├── ai_provider.rs        NEW — AiProvider trait
  │     ask_clarifying_question() → String
  │     normalize_question() → String
  │     generate_form() → IntakeTemplate (Phase 0d)
  ├── claude_provider.rs    NEW — ClaudeProvider implementation
  └── lib.rs                EXTEND — Export ai_provider module

control-service/src/
  └── intake_service.rs     EXTEND — Add AI fallback orchestration:
                               - detect_unlisted_app_type()
                               - ai_clarify_response()
                               - store_ai_question()
```

**AI Integration Points:**
```rust
pub trait AiProvider {
    async fn ask_clarifying_question(
        &self,
        context: &IntakeContext,
        ambiguous_response: &str,
    ) -> Result<String, IntakeError>;

    async fn normalize_question(
        &self,
        question: &str,
    ) -> Result<String, IntakeError>;
}
```

**Edge Case Handling:**
- User answers "maybe" or "it depends" → AI asks targeted follow-up
- User describes unlisted application type → AI asks 3-5 clarifying questions
- Ambiguous responses captured and normalized for learning engine (Phase 0c)

**Validation Gate:**
- AI clarification triggered correctly for ambiguous responses
- Normalized questions stored with pg_trgm GIN index for similarity search
- End-to-end test: unlisted app type → AI questions → manual completion
- AI provider abstraction allows swapping Claude for OpenAI/Mock

**Team:** Dmitri Volkov (Backend), Priya Mehta (Data)

---

### Phase 0c: Structured Intake System — Smart Learning Engine (Week 5)
**Goal: Auto-promote frequently-asked AI questions to canonical forms.**

Files to create:

```
intake-engine/src/
  ├── learning.rs           NEW — Auto-promotion logic
  │     check_promotion_threshold()
  │     generate_canonical_question()
  │     suggest_template_update()
  └── lib.rs                EXTEND — Export learning module

control-service/src/
  └── intake_learning_service.rs  NEW — Background job orchestration:
                                    - detect_promotion_candidates()
                                    - create_admin_review_task()
                                    - apply_approved_promotion()
```

**Auto-Promotion Logic:**
```
1. AI asks clarifying question Q_ai
2. Store normalized version with pg_trgm similarity index
3. Background job runs daily:
   - Query questions with similarity > 0.6 threshold
   - Count occurrences per normalized form
   - If count ≥ 5 → queue for admin review
4. Admin reviews → approves → adds to template decision tree
```

**Database Queries (PostgreSQL with pg_trgm):**
```sql
-- Find similar questions
SELECT normalized_question, COUNT(*) as occurrence_count
FROM ai_intake_questions
WHERE similarity(normalized_question, 'target question') > 0.6
  AND promoted_to_template = FALSE
GROUP BY normalized_question
HAVING COUNT(*) >= 5
ORDER BY occurrence_count DESC;
```

**Validation Gate:**
- Background job detects 5+ similar questions and creates review task
- Admin review UI displays candidate question with context
- Approved promotion correctly updates template decision tree
- Migration from Phase 2+ embeddings: performance degradation triggers upgrade

**Threshold Configuration (See DEC-PM-PENDING-auto-promotion-threshold.md):**
- Initial: Fixed threshold of 5 occurrences
- Review: 3-month evaluation of precision/recall
- Upgrade path: Variable thresholds by app type if precision < 60%

**Team:** Dmitri Volkov (Backend), Priya Mehta (Data)

---

### Phase 0d: Structured Intake System — AI Form Generation (Weeks 6-7)
**Goal: Generate draft intake forms for unlisted application types on-demand.**

Files to extend:

```
intake-engine/src/
  ├── ai_provider.rs        EXTEND — Add generate_form() method:
  │     async fn generate_form(
  │         &self,
  │         app_description: &str,
  │         ai_qa_context: Vec<(String, String)>,
  │     ) -> Result<IntakeTemplate, IntakeError>;
  │
  └── form_validator.rs     NEW — Draft form validation:
        validate_generated_form() → checks structure, detects PII

control-service/src/
  └── intake_service.rs     EXTEND — Add form generation flow:
                               - generate_draft_form()
                               - validate_draft_safety()
                               - store_for_review()
```

**AI Form Generation Prompt Structure:**
```
You are generating a structured intake form for: {app_description}

Based on these Q&A exchanges:
{ai_qa_context}

Generate a decision tree with:
1. 5-15 questions covering architecture, deployment, integrations
2. Conditional logic (show_if, branches)
3. Feature derivations (features array per answer)
4. Module mappings (which RealmForge modules are needed)

Return valid JSON matching IntakeTemplate schema.
```

**Safety Validations:**
- **PII Detection:** Scan for fields requesting SSN, credit cards, passwords
- **Structure Validation:** Ensure valid JSON, required fields present
- **Complexity Limits:** Max 20 questions, max 4 nesting levels
- **Profanity Filter:** Reject forms with inappropriate content

**Draft Form Usage Policy (See DEC-SECURITY-PENDING-draft-form-usage.md):**
- **Option A (Immediate Use):** High risk — T1 malicious injection
- **Option B (Approval-First):** Low risk — delays user, admin bottleneck
- **Option C (Hybrid):** **RECOMMENDED** — One-time use with PII detection + rate limiting

**Validation Gate:**
- AI generates valid IntakeTemplate for "IoT monitoring dashboard"
- PII detection blocks forms requesting sensitive data
- Draft form stored in admin review queue with pending status
- One-time use allows user to complete intake without blocking

**Team:** Dmitri Volkov (Backend), Priya Mehta (Data)

---

### Phase 0e: Structured Intake System — Admin Review Portal (Weeks 8-9)
**Goal: Build admin UI for reviewing AI-generated content (forms, questions, promotions).**

Files to create:

```
control-api/src/routes/
  └── admin_intake.rs       NEW — Admin endpoints:
                               GET /v1/admin/intake/review-queue
                               GET /v1/admin/intake/review/:id
                               POST /v1/admin/intake/review/:id/approve
                               POST /v1/admin/intake/review/:id/reject
                               PUT /v1/admin/intake/templates/:id/edit

frontend/admin-portal/       NEW (if separate frontend exists)
  ├── ReviewQueue.tsx       Component: list pending reviews
  ├── FormEditor.tsx        Component: edit decision tree JSON
  ├── QuestionReview.tsx    Component: approve/reject question promotions
  └── TemplatePreview.tsx   Component: preview generated forms
```

**Admin Review UI Features:**
1. **Review Queue Dashboard**
   - Filter by type: AI Questions, Generated Forms, Template Updates
   - Sort by: Date, Occurrence Count, Priority
   - Bulk actions: Approve Selected, Reject Selected

2. **Form Editor**
   - Visual decision tree editor (or JSON with validation)
   - Preview mode: simulate intake flow
   - Diff view: compare AI-generated vs. manual edits

3. **Approval Workflow**
   - Single reviewer (PM or CTO) required initially
   - Comments/feedback on rejections
   - Audit trail: who approved, when, why

**Access Control (See DEC-COUNCIL-PENDING-intake-form-approval-authority.md):**
- **Phase 1 (First 5 forms):** Full Council approval required
- **Phase 2 (After 5 forms):** PM + CTO approval sufficient
- **Escalation:** Council review for high-risk forms (e.g., financial, healthcare)

**Validation Gate:**
- Admin can view all pending reviews in queue
- Form editor correctly validates decision tree JSON
- Approval updates database and makes form available
- Rejection provides feedback, allows AI to regenerate

**Team:** Kai Larson (Frontend), Dmitri Volkov (Backend)

---

### Phase 0f: Structured Intake System — Expansion to 15 App Types (Weeks 10-12)
**Goal: Author 12 additional application type templates to cover 95% of use cases.**

Templates to author:

```
4.  Mobile App (iOS/Android)       — native vs hybrid, backend API, push notifications
5.  Microservices Architecture      — service count, communication (REST/gRPC), service mesh
6.  Batch Processing Pipeline       — data source, schedule, transformations, output
7.  Real-Time Streaming App         — event source (Kafka, Kinesis), processing, sinks
8.  Machine Learning Model API      — framework, model type, input/output, scaling
9.  E-Commerce Platform             — payment gateway, inventory, cart, checkout, admin
10. Content Management System       — content types, media storage, versioning, permissions
11. Multi-Tenant SaaS               — tenant isolation, billing, admin portal, feature flags
12. IoT Data Platform               — device protocol, telemetry, time-series DB, dashboards
13. GraphQL API                     — schema, resolvers, subscriptions, caching
14. Serverless Functions            — trigger type, runtime, cold start mitigation, logging
15. Desktop Application             — framework (Electron, Qt, etc.), updater, local storage
```

**Template Authoring Process:**
1. **Research:** Interview 2-3 teams who built this app type
2. **Draft:** Create decision tree with 8-15 questions
3. **Review:** PM + Domain Expert approve
4. **Test:** Run 3 test intakes, refine based on feedback
5. **Commit:** Merge into templates table with version 1.0

**Validation Gate:**
- All 15 templates validated with `validate_template()`
- Each template tested with 3 real-world scenarios
- Feature derivations correctly map to RealmForge modules
- QA sign-off: intakes complete in < 5 minutes with clear outputs

**Coverage Target:**
- 15 templates cover 95% of intake requests
- Remaining 5% handled by AI-assisted intake + learning engine
- Unlisted app types trigger AI form generation (Phase 0d)

**Team:** Dmitri Volkov (Backend), Kai Larson (Frontend), Priya Mehta (Data), Meg Thompson (QA)

---

### Phase 1: Service Layer — The Orchestrator
**Goal: Create the service layer that orchestrates domain → policy → store flows.**

Files to create:

```
control-service/                 NEW CRATE
  ├── Cargo.toml            Dependencies: authority-domain, audit-log, policy-engine, control-store, snapshot-ledger, chrono, serde, thiserror, tracing
  └── src/
      ├── lib.rs            Module declarations
      ├── error.rs          ServiceError enum (typed error propagation)
      ├── session_service.rs
      │     issue_session()
      │     renew_session()
      │     revoke_session()
      │     validate_session()
      ├── command_service.rs
      │     propose_command() → authorize → store (bounded command)
      │     authorize_command(action, scope) → policy check → store
      │     apply_command(command_id) → state transition → audit event → snapshot anchor
      │     deny_command(command_id, denial) → audit denial event
      ├── audit_service.rs
      │     append_event() → hash chain → store
      │     query_events(project_id, filters) → paged results
      │     verify_chain(project_id) → integrity check
      ├── snapshot_service.rs
      │     create_snapshot() → collect object refs → compute hash → store
      │     validate_snapshot(id) → verify hash + verify object refs
      │     list_snapshots(project_id) → ordered by time
      │     compare_snapshots(from_id, to_id) → compute diff
      ├── rollback_service.rs
      │     preview_rollback(from, to) → compute impact
      │     execute_rollback(from, to) → restore state
      │     verify_rollback(id) → consistency checks
      ├── skill_service.rs
      │     register_skill() → create skill registration
      │     activate_skill_session() → bind skill → session
      │     validate_skill_integrity() → check skill against expected state
      └── actor_service.rs
            get_scope(actor_id, session_id) → build ActorScope
            update_scope(scope_id) → refresh context
```

**Critical design rules for control-service:**
1. **Every service method returns `Result<T, ServiceError>`** — no panics, no unwraps
2. **Every mutation path goes through policy check first** — no exceptions
3. **Every mutation emits an audit event** — no silent writes
4. **Service methods are shared** — called by API, CLI, and MCP equally

**Validation Gate:**
- Crate compiles with `cargo check -p control-service`
- Service methods have doc comments explaining preconditions/postconditions
- No direct DB access — all persistence goes through control-store

---

### Phase 2: Session Lifecycle Engine
**Goal: Complete identity/session management — issue, renew, revoke, expire, scope-build.**

Files to create/modify:

```
control-service/src/
  ├── session_service.rs    FULL IMPLEMENTATION
  └── actor_service.rs      FULL IMPLEMENTATION

authority-domain/src/
  └── scope.rs              EXTEND — Add scope factory methods, add session→scope builder

control-store/src/
  └── lib.rs                EXTEND — Add session queries, scope queries
```

**Session lifecycle:**
```
1. Actor authenticates (external — out of Core scope)
2. session_service.issue_session(actor_id, tenant_id, project_id, ttl)
   → creates Session with state='issued'
   → returns ActorScope with execution_mode='ReadOnly'
3. session_service.activate_session(session_id)
   → transitions to 'active'
4. session_service.renew_session(session_id, new_ttl)
   → extends expires_at
5. session_service.revoke_session(session_id, reason)
   → transitions to 'revoked'
   → records audit event
6. Session expires naturally (background sweeper or lazy check)
   → any subsequent policy check fails with SessionExpired
```

**Validation Gate:**
- Session issue → activate → renew → revoke lifecycle test passes
- Expired session returns SessionExpired denial
- Scope correctly reflects execution mode and permissions

---

### Phase 3: Command Lifecycle Engine
**Goal: Full bounded command lifecycle — propose, authorize, apply, deny, fail.**

Files to create/modify:

```
control-service/src/
  ├── command_service.rs    FULL IMPLEMENTATION
  └── audit_service.rs      FULL IMPLEMENTATION

authority-domain/src/
  └── command.rs            EXTEND — Add status transition validation
                               Proposed → Authorized | Denied
                               Authorized → Applied | Failed
                               Applied → terminal

policy-engine/src/
  └── lib.rs                EXTEND — Add command-specific policy checks (target_type, action)
```

**Command lifecycle:**
```
1. propose_command(scope, action, target, payload)
   → creates BoundedCommand with status='proposed'
   → returns CommandId
2. authorize_command(command_id, scope)
   → calls policy engine with scope + action + mutating flag
   → if denied: set status='denied', emit audit event, return PolicyDenial
   → if allowed: set status='authorized', emit audit event
3. apply_command(command_id)
   → validates command is 'authorized'
   → executes command effects (service-specific)
   → sets status='applied'
   → emits audit event with full payload
   → creates snapshot anchor
4. deny_command(command_id, denial)
   → sets status='denied'
   → emits audit denial event with denial reason
```

**Validation Gate:**
- Full lifecycle test: propose → authorize → apply → verify audit trail
- Denial test: propose → authorize (with scope that denies) → verify denial response
- Attempting to apply an unauthorized command returns error

---

### Phase 4: Audit Trail Engine
**Goal: Full append-only audit with hash-chain verification and query.**

Files to create/modify:

```
control-service/src/
  └── audit_service.rs      FULL IMPLEMENTATION

audit-log/src/
  └── lib.rs                EXTEND — Add event type categorization,
                               add batch append, add chain verification

control-store/src/
  └── lib.rs                EXTEND — Add audit query by project/time/event_type,
                               add chain verification query,
                               add paged results
```

**Audit capabilities:**
```
append_event(event)           → stores with hash chain link
query_events(project_id, filters) → time range, event_type, actor_id, entity_type
verify_chain(project_id)      → recompute all hashes, detect tampering
get_chain_anchors(project_id) → first, last, count, integrity status
```

**Validation Gate:**
- 5 events appended → chain integrity verified
- Tampering with one event's payload → chain verification fails
- Query returns correct paged results with filters

---

### Phase 5: Snapshot & Rollback Engine
**Goal: Complete snapshot orchestration and rollback execution.**

Files to create/modify:

```
control-service/src/
  ├── snapshot_service.rs   FULL IMPLEMENTATION
  └── rollback_service.rs   FULL IMPLEMENTATION

snapshot-ledger/src/
  ├── manifest.rs           EXTEND — Add SnapshotDelta (added, removed, changed objects)
  ├── object_store.rs       EXTEND — Add delete, list, stat operations
  └── rollback.rs           NEW — RollbackPreview, impact analysis, execution plan

control-store/src/
  └── lib.rs                EXTEND — Add rollback preview queries,
                               add snapshot manifest list/compare queries
```

**Snapshot capabilities:**
```
create_snapshot(project_id, reason, parent_id)
  → scans registered objects
  → computes content-addressed refs
  → creates SnapshotManifest with hash chain
  → stores in DB + object store

validate_snapshot(snapshot_id)
  → verifies manifest hash
  → verifies all object refs exist in object store
  → verifies all table exports exist
  → returns validation report

list_snapshots(project_id, limit, offset)
  → returns ordered list with metadata

compare_snapshots(from_id, to_id)
  → returns SnapshotDelta (added, removed, changed objects/tables)
```

**Rollback capabilities:**
```
preview_rollback(from_snapshot, to_snapshot)
  → computes what will be restored
  → computes what will be lost
  → identifies conflicts or blockers
  → returns RollbackPreview with impact report

execute_rollback(from_snapshot, to_snapshot, preview_id)
  → validates preview was computed
  → restores objects from object store
  → restores table exports
  → creates new snapshot with reason='rollback'
  → emits audit events for every restored entity

verify_rollback(snapshot_id)
  → runs consistency checks on restored state
  → verifies file hashes match expected
  → verifies entity counts match snapshot
  → returns verification report
```

**Validation Gate:**
- Full snapshot lifecycle: create → validate → compare → store → retrieve
- Full rollback lifecycle: preview → execute → verify
- Snapshot chain integrity: parent → child links are valid
- Rollback impact report correctly lists what changes

---

### Phase 6: Agent Work Packet Generator
**Goal: Generate scoped work packets from approved planning state.**

Files to create:

```
control-service/src/
  ├── work_packet_service.rs   NEW
  │     generate_work_packet(scope, node_id, objective)
  │       → compute allowed files from node→file links
  │       → compute denied files (everything not allowed)
  │       → extract required contracts from node→contract links
  │       → extract required tests from node→test links
  │       → extract required trace points from node→trace links
  │       → compute rollback anchor from latest snapshot
  │       → return AgentWorkPacket with all scoped boundaries
  │
  │     validate_packet_boundaries(packet_id)
  │       → verify all files in packet are within allowed scope
  │       → verify packet doesn't exceed cost budget
  │       → verify all required contracts have definitions
  │       → return validation result

authority-domain/src/
  ├── work_packet.rs         NEW — AgentWorkPacket, PacketScope, FilePermission, ContractRequirement
  └── lib.rs                 UPDATE — Add work_packet module

control-store/src/
  └── lib.rs                EXTEND — Add work packet persistence
```

**Agent Work Packet structure:**
```rust
pub struct AgentWorkPacket {
    pub id: PacketId,
    pub agent_id: ActorId,
    pub work_path_node_id: String,
    pub objective: String,
    pub allowed_file_paths: Vec<String>,
    pub denied_file_paths: Vec<String>,
    pub required_contracts: Vec<String>,
    pub required_tests: Vec<String>,
    pub required_trace_points: Vec<String>,
    pub rollback_anchor: Option<SnapshotId>,
    pub cost_budget: Option<CostBudget>,
    pub permission_scope: PacketPermissionScope,
    pub created_at: DateTime<Utc>,
    pub status: PacketStatus,
}
```

**Validation Gate:**
- Packet generation correctly scopes allowed/denied files
- Packet validation detects files outside allowed scope
- Packet correctly identifies rollback anchor from latest snapshot

---

### Phase 7: Full API Surface
**Goal: Complete REST API — session, command, audit, snapshot, rollback, health.**

Files to create/modify:

```
control-api/src/
  ├── lib.rs                EXTEND — Add all route groups
  ├── main.rs               EXTEND — Add middleware, CORS, tracing
  ├── error.rs              NEW — ApiError enum with status code mapping
  ├── middleware.rs          NEW — Request tracing, error handling, request ID
  ├── routes/
  │   ├── mod.rs            NEW — Route group declarations
  │   ├── health.rs         NEW — Health routes (prometheus, readiness, liveness)
  │   ├── session.rs        NEW — POST /v1/session, GET /v1/session/:id, DELETE /v1/session/:id
  │   ├── command.rs        NEW — POST /v1/commands (propose), PUT /v1/commands/:id/authorize, PUT /v1/commands/:id/apply
  │   ├── audit.rs          NEW — GET /v1/audit/events, GET /v1/audit/chain/verify
  │   ├── snapshot.rs       NEW — POST /v1/snapshots, GET /v1/snapshots, GET /v1/snapshots/:id, POST /v1/snapshots/:id/validate
  │   ├── rollback.rs       NEW — POST /v1/rollback/preview, POST /v1/rollback/execute, GET /v1/rollback/:id/verify
  │   ├── actor.rs          NEW — GET /v1/actors/:id/scope
  │   └── work_packet.rs    NEW — POST /v1/work-packets/generate, POST /v1/work-packets/:id/validate
  └── models.rs             NEW — Request/response types for all endpoints
```

**API Design Principles:**
1. Every route handler is < 30 lines (calls service, formats response)
2. Every response includes `request_id` for trace correlation
3. Error responses follow RFC 7807 Problem Details format
4. All mutation endpoints return 201 (not 200)
5. All list endpoints support pagination (limit, offset, sort_by, order)
6. All endpoints are versioned under `/v1/`

**Validation Gate:**
- `cargo run -p control-api` starts and responds to health check
- All endpoints return correct HTTP status codes
- Error responses include proper problem details format
- Postman/curl collection works against every endpoint

---

### Phase 8: Full CLI Surface
**Goal: Complete operator CLI — all governance commands available from terminal.**

Files to create/modify:

```
operator-cli/src/
  ├── main.rs               EXTEND — Add all subcommands
  └── commands/
      ├── mod.rs            NEW — Command modules
      ├── session.rs        NEW — session issue, session renew, session revoke, session list
      ├── command.rs        NEW — command propose, command authorize, command apply, command deny
      ├── audit.rs          NEW — audit events, audit verify-chain, audit export
      ├── snapshot.rs       NEW — snapshot create, snapshot validate, snapshot list, snapshot compare
      ├── rollback.rs       NEW — rollback preview, rollback execute, rollback verify
      ├── actor.rs          NEW — actor scope
      ├── work_packet.rs    NEW — work-packet generate, work-packet validate
      ├── skill.rs          NEW — skill register, skill list, skill validate
      ├── migrate.rs        NEW — migrate up, migrate down, migrate list (from existing ListMigrations)
      └── config.rs         NEW — config init, config show, config validate
```

**CLI Design Principles:**
1. Every command has `--help` that explains purpose and all flags
2. Output is machine-readable (JSON) by default, human-readable with `--pretty`
3. Tab completion generated for bash, zsh, powershell
4. Exit codes: 0=success, 1=user error, 2=data error, 3=runtime error

**Validation Gate:**
- `cargo run -p operator-cli -- --help` shows all subcommands
- Every subcommand works end-to-end against a test database
- `--pretty` and default JSON output both work

---

### Phase 9: Full MCP Surface
**Goal: Complete MCP tool contracts — all governance tools available to AI agents.**

Files to create/modify:

```
agent-mcp/src/
  ├── lib.rs                EXTEND — Add all tool definitions
  ├── error.rs              NEW — McpError variants for all error types
  ├── tools/
  │   ├── mod.rs            NEW — Tool module declarations
  │   ├── session.rs        NEW — core_issue_session, core_renew_session, core_revoke_session
  │   ├── command.rs        NEW — core_propose_command, core_authorize_command, core_apply_command
  │   ├── audit.rs          NEW — core_query_events, core_verify_chain
  │   ├── snapshot.rs       NEW — core_create_snapshot, core_validate_snapshot, core_compare_snapshots
  │   ├── rollback.rs       NEW — core_preview_rollback, core_execute_rollback, core_verify_rollback
  │   ├── actor.rs          NEW — core_get_actor_scope
  │   ├── skill.rs          NEW — core_register_skill, core_activate_skill_session
  │   ├── work_packet.rs    NEW — core_generate_work_packet, core_validate_work_packet
  │   └── intake.rs         NEW — intake_start, intake_answer, intake_complete,
  │                            intake_ai_assist, intake_generate_form,
  │                            admin_get_review_queue, admin_review_decision
  └── types.rs              NEW — Shared MCP tool input/output types
```

**MCP Design Principles:**
1. Each tool maps one-to-one with a service method
2. Tool descriptions are complete enough for an AI agent to use without examples
3. Input schemas are strict — every required field is listed, types are explicit
4. Error messages include corrective actions when possible
5. Total tool count stays under 30 (current: 5, target: ~28 with intake tools)

**New MCP Tools for Intake System (8 tools):**
```
intake_start(app_type?)           → Start session, return first question
intake_answer(session_id, answer) → Submit answer, return next question or completion
intake_complete(session_id)       → Finalize, return derived modules/personas

intake_ai_assist(session_id)      → Request AI clarification for ambiguous response
intake_ai_answer(session_id, q, a) → Submit AI Q&A, continue form-based flow
intake_generate_form(description) → Generate draft form for unlisted app type

admin_get_review_queue()          → List pending reviews (questions, forms)
admin_review_decision(id, action) → Approve/reject review item
admin_edit_template(id, tree)     → Update template decision tree
```

**Validation Gate:**
- `core_tool_definitions()` returns all ~28 tools with complete schemas
- Every tool can be called and returns valid JSON
- Unknown tool returns McpError::UnknownTool
- Intake tools correctly orchestrate form-based + AI-assisted flows

---

### Phase 10: Integration Testing
**Goal: End-to-end tests that validate the full governance loop.**

Files to create:

```
control-service/tests/
  ├── common/mod.rs         NEW — Test helpers, test database setup, test fixtures
  ├── session_lifecycle.rs  NEW — Issue → Activate → Renew → Revoke → Verify
  ├── command_lifecycle.rs  NEW — Propose → Authorize → Apply → Audit → Verify chain
  ├── policy_enforcement.rs NEW — All 6 denial paths tested with full coverage
  ├── audit_integrity.rs    NEW — Hash chain with tampering detection
  ├── snapshot_flow.rs      NEW — Create → Validate → Compare → Store → Retrieve
  ├── rollback_flow.rs      NEW — Preview → Execute → Verify → Validate
  ├── work_packet_flow.rs   NEW — Generate → Validate → Scope check → Boundary test
  └── intake_flow.rs        NEW — Form-based intake → AI fallback → learning → admin review

control-api/tests/
  ├── common/mod.rs         NEW — Test app builder, test client
  ├── health_test.rs        NEW — Health endpoint returns expected response
  ├── session_test.rs       NEW — Full session API lifecycle
  ├── command_test.rs       NEW — Full command API lifecycle
  ├── audit_test.rs         NEW — Full audit API lifecycle
  └── integration_test.rs   NEW — Cross-crate: propose command via API → verify via audit API → snapshot → rollback → verify

operator-cli/tests/
  ├── cli_integration.rs    NEW — Test CLI commands against test database
  └── output_format.rs      NEW — Verify JSON and pretty output formats
```

**Integration test design:**
1. Each test creates its own test database (via migration) for isolation
2. Tests run in parallel using unique schema/namespace
3. Cross-crate "happy path" test covers: propose → authorize → apply → audit → snapshot → rollback → verify
4. Security boundary tests verify forbidden paths return errors
5. Intake system tests: form completion → AI clarification → form generation → admin approval

**Validation Gate:**
- `cargo test --workspace` passes all ~90+ tests in under 60 seconds
- Integration tests cover all crates together
- Test coverage > 80% on service layer

---

### Phase 11: Observability & Error Boundaries
**Goal: Structured logging, tracing, metrics, and error propagation.**

Files to create/modify:

```
control-service/src/
  ├── lib.rs                EXTEND — Add tracing spans to all service methods
  └── error.rs              EXTEND — Add error codes, severity levels, user messages

control-api/src/
  ├── middleware.rs          EXTEND — Add request tracing, latency metrics, error logging
  └── main.rs               EXTEND — Add tracing subscriber setup, metrics endpoint

control-store/src/
  └── lib.rs                EXTEND — Add query timing, connection pool metrics

policy-engine/src/
  └── lib.rs                EXTEND — Add policy decision logging (structured)
```

**Observability principles:**
1. Every service method starts with a `tracing::info_span!`
2. Every policy decision emits a structured log event with scope, action, decision, reason
3. Every database query is instrumented with timing
4. API responses include `X-Request-Id` header
5. Metrics: request count, latency p50/p95/p99, error rate by code, policy deny rate

**Validation Gate:**
- Structured trace output is visible in console
- Policy decisions are logged with all relevant context
- Error propagation correctly wraps and annotates errors at each layer

---

## AI Agent Workflow

Every AI agent that implements a phase MUST follow this workflow:

### 1. Read Phase Contract
```
Read MASTER_BUILD_PLAN.md (this document)
Read the specific phase section
Read the crate's existing source files
```

### 2. Plan Verification
```
Load /skill-creator (to register any new skills needed)
Convene /cto (Rena Okafor) for architecture review
Map affected crate dependencies
Identify any new types or enums needed in authority-domain
```

### 3. Implementation Order
```
Types first (authority-domain) → values follow types
Policy next (policy-engine) → safety before execution
Service layer next → orchestration logic
Store layer next → persistence
Transport last (API/CLI/MCP) → thin adapters only
Tests at every layer → TDD where possible
```

### 4. File Size Discipline
```
Target: < 300 lines per source file
Hard cap: 500 lines (requires /cto justification)
Split large modules into submodules before they bloat
```

### 5. Code Quality Gates
```
- [ ] cargo fmt --all -- --check
- [ ] cargo clippy --workspace -- -D warnings
- [ ] cargo test --workspace
- [ ] No unwrap() without SAFETY/INVARIANT comment
- [ ] No unsafe without SAFETY: comment
- [ ] All public items have doc comments
- [ ] All errors follow the error boundary pattern
```

### 6. Handoff Protocol

When a phase is complete:
```
1. Load /qa (Meg Thompson) for verification
   - Reproduce all test results
   - Document any defects found
   - Sign off on phase completion

2. Load /tech-writer (Clara Mills) for documentation
   - Update ADRs if architecture decisions changed
   - Update crate documentation
   - Update API/CLI/MCP documentation

3. Load /release-manager (Sam Osei) for phase handoff
   - Verify validation gates
   - Update migration state
   - Record phase completion in build plan

4. Report to /orchestrator
   - Phase completed
   - Defects found and fixed
   - Open items for next phase
   - Dependencies for downstream phases
```

---

## Complete File Manifest

### Phase 0 — Foundation Hardening (7 files)
- `authority-domain/src/error.rs` (NEW)
- Extensions to: `ids.rs`, `scope.rs`, `state.rs`, `command.rs`, `skill.rs`, `skill_creator.rs`

### Phase 0a — Intake Core Engine (15 files new)
- `intake-engine/Cargo.toml` (NEW)
- `intake-engine/src/lib.rs` (NEW)
- `intake-engine/src/error.rs` (NEW)
- `intake-engine/src/types.rs` (NEW)
- `intake-engine/src/evaluator.rs` (NEW)
- `intake-engine/src/navigator.rs` (NEW)
- `intake-engine/src/validator.rs` (NEW)
- `control-service/src/intake_service.rs` (NEW)
- `control-service/src/lib.rs` (EXTEND)
- `control-store/src/lib.rs` (EXTEND)
- `migrations/003_intake_foundation.sql` (NEW)
- Seed data: 3 application type templates (Static Website, REST API, Dynamic Web App)

### Phase 0b — AI Assistance (4 files new/extended)
- `intake-engine/src/ai_provider.rs` (NEW)
- `intake-engine/src/claude_provider.rs` (NEW)
- `intake-engine/src/lib.rs` (EXTEND)
- `control-service/src/intake_service.rs` (EXTEND)

### Phase 0c — Smart Learning Engine (3 files new/extended)
- `intake-engine/src/learning.rs` (NEW)
- `intake-engine/src/lib.rs` (EXTEND)
- `control-service/src/intake_learning_service.rs` (NEW)

### Phase 0d — AI Form Generation (3 files extended/new)
- `intake-engine/src/ai_provider.rs` (EXTEND)
- `intake-engine/src/form_validator.rs` (NEW)
- `control-service/src/intake_service.rs` (EXTEND)

### Phase 0e — Admin Review Portal (5 files new)
- `control-api/src/routes/admin_intake.rs` (NEW)
- `frontend/admin-portal/ReviewQueue.tsx` (NEW)
- `frontend/admin-portal/FormEditor.tsx` (NEW)
- `frontend/admin-portal/QuestionReview.tsx` (NEW)
- `frontend/admin-portal/TemplatePreview.tsx` (NEW)

### Phase 0f — 15 App Type Templates (seed data)
- Seed data: 12 additional application type templates (total 15)

### Phase 1 — Service Layer (12 files)
- `control-service/Cargo.toml` (NEW)
- `control-service/src/lib.rs` (NEW)
- `control-service/src/error.rs` (NEW)
- `control-service/src/session_service.rs` (NEW)
- `control-service/src/command_service.rs` (NEW)
- `control-service/src/audit_service.rs` (NEW)
- `control-service/src/snapshot_service.rs` (NEW)
- `control-service/src/rollback_service.rs` (NEW)
- `control-service/src/skill_service.rs` (NEW)
- `control-service/src/actor_service.rs` (NEW)
- `Cargo.toml` (EXTEND — add `control-service` member)
- `control-service/src/work_packet_service.rs` (NEW in Phase 6)

### Phase 2 — Session Lifecycle (4 files modified)
- `control-service/src/session_service.rs` (FULL IMPL)
- `control-service/src/actor_service.rs` (FULL IMPL)
- `authority-domain/src/scope.rs` (EXTEND)
- `control-store/src/lib.rs` (EXTEND)

### Phase 3 — Command Lifecycle (4 files modified)
- `control-service/src/command_service.rs` (FULL IMPL)
- `control-service/src/audit_service.rs` (FULL IMPL)
- `authority-domain/src/command.rs` (EXTEND)
- `policy-engine/src/lib.rs` (EXTEND)

### Phase 4 — Audit Trail Engine (3 files modified)
- `control-service/src/audit_service.rs` (FULL IMPL)
- `audit-log/src/lib.rs` (EXTEND)
- `control-store/src/lib.rs` (EXTEND)

### Phase 5 — Snapshot & Rollback Engine (4 files modified + 1 new)
- `control-service/src/snapshot_service.rs` (FULL IMPL)
- `control-service/src/rollback_service.rs` (FULL IMPL)
- `snapshot-ledger/src/manifest.rs` (EXTEND)
- `snapshot-ledger/src/rollback.rs` (NEW)
- `snapshot-ledger/src/object_store.rs` (EXTEND)
- `control-store/src/lib.rs` (EXTEND)

### Phase 6 — Work Packet Generator (4 files new/modified)
- `control-service/src/work_packet_service.rs` (NEW)
- `authority-domain/src/work_packet.rs` (NEW)
- `authority-domain/src/lib.rs` (EXTEND)
- `control-store/src/lib.rs` (EXTEND)

### Phase 7 — Full API Surface (14 files new/modified)
- `control-api/src/lib.rs` (EXTEND)
- `control-api/src/main.rs` (EXTEND)
- `control-api/src/error.rs` (NEW)
- `control-api/src/middleware.rs` (NEW)
- `control-api/src/models.rs` (NEW)
- `control-api/src/routes/mod.rs` (NEW)
- `control-api/src/routes/health.rs` (NEW)
- `control-api/src/routes/session.rs` (NEW)
- `control-api/src/routes/command.rs` (NEW)
- `control-api/src/routes/audit.rs` (NEW)
- `control-api/src/routes/snapshot.rs` (NEW)
- `control-api/src/routes/rollback.rs` (NEW)
- `control-api/src/routes/actor.rs` (NEW)
- `control-api/src/routes/work_packet.rs` (NEW)

### Phase 8 — Full CLI Surface (12 files new/modified)
- `operator-cli/src/main.rs` (EXTEND)
- `operator-cli/src/commands/mod.rs` (NEW)
- `operator-cli/src/commands/session.rs` (NEW)
- `operator-cli/src/commands/command.rs` (NEW)
- `operator-cli/src/commands/audit.rs` (NEW)
- `operator-cli/src/commands/snapshot.rs` (NEW)
- `operator-cli/src/commands/rollback.rs` (NEW)
- `operator-cli/src/commands/actor.rs` (NEW)
- `operator-cli/src/commands/work_packet.rs` (NEW)
- `operator-cli/src/commands/skill.rs` (NEW)
- `operator-cli/src/commands/migrate.rs` (NEW)
- `operator-cli/src/commands/config.rs` (NEW)

### Phase 9 — Full MCP Surface (13 files new/modified)
- `agent-mcp/src/lib.rs` (EXTEND)
- `agent-mcp/src/error.rs` (NEW)
- `agent-mcp/src/types.rs` (NEW)
- `agent-mcp/src/tools/mod.rs` (NEW)
- `agent-mcp/src/tools/session.rs` (NEW)
- `agent-mcp/src/tools/command.rs` (NEW)
- `agent-mcp/src/tools/audit.rs` (NEW)
- `agent-mcp/src/tools/snapshot.rs` (NEW)
- `agent-mcp/src/tools/rollback.rs` (NEW)
- `agent-mcp/src/tools/actor.rs` (NEW)
- `agent-mcp/src/tools/skill.rs` (NEW)
- `agent-mcp/src/tools/work_packet.rs` (NEW)
- `agent-mcp/src/tools/intake.rs` (NEW — 9 intake tools)

### Phase 10 — Integration Testing (12 files new)
- `control-service/tests/common/mod.rs` (NEW)
- `control-service/tests/session_lifecycle.rs` (NEW)
- `control-service/tests/command_lifecycle.rs` (NEW)
- `control-service/tests/policy_enforcement.rs` (NEW)
- `control-service/tests/audit_integrity.rs` (NEW)
- `control-service/tests/snapshot_flow.rs` (NEW)
- `control-service/tests/rollback_flow.rs` (NEW)
- `control-service/tests/work_packet_flow.rs` (NEW)
- `control-service/tests/intake_flow.rs` (NEW)
- `control-api/tests/common/mod.rs` (NEW)
- `control-api/tests/health_test.rs` (NEW)
- `control-api/tests/session_test.rs` (NEW)
- `control-api/tests/command_test.rs` (NEW)
- `control-api/tests/audit_test.rs` (NEW)
- `control-api/tests/integration_test.rs` (NEW)
- `operator-cli/tests/cli_integration.rs` (NEW)
- `operator-cli/tests/output_format.rs` (NEW)

### Phase 11 — Observability (5 files modified)
- `control-service/src/lib.rs` (EXTEND)
- `control-service/src/error.rs` (EXTEND)
- `control-api/src/middleware.rs` (EXTEND)
- `control-api/src/main.rs` (EXTEND)
- `control-store/src/lib.rs` (EXTEND)
- `policy-engine/src/lib.rs` (EXTEND)

---

## Dependency Graph (Crate Dependencies)

```
operator-cli
  └── control-service
        ├── authority-domain       (pure types — no deps on other RealmForge crates)
        ├── audit-log       (depends on authority-domain)
        ├── policy-engine       (depends on authority-domain)
        ├── control-store        (depends on authority-domain, audit-log, snapshot-ledger)
        ├── snapshot-ledger     (depends on authority-domain)
        └── intake-engine        (pure logic — no deps on other RealmForge crates)

control-api
  └── control-service (same hierarchy)

agent-mcp
  └── control-service (same hierarchy)

intake-engine ────┐
snapshot-ledger ──┤
audit-log ────────┤──→ authority-domain (pure)
policy-engine ────┘
```

**Layer violation detection:**
- authority-domain MUST NOT import any other capability crate
- audit-log, policy-engine, snapshot-ledger, intake-engine MAY import authority-domain only
- control-store MAY import authority-domain, audit-log, snapshot-ledger
- control-service MAY import authority-domain, audit-log, policy-engine, control-store, snapshot-ledger, intake-engine
- control-api, operator-cli, agent-mcp MAY import control-service only (NOT control-store directly)

---

## The Final Promise

When all 17 phases are complete, RealmForge Core will be able to prove:

```
Who is allowed to do what?
  → Authorized through ActorScope + PolicyDecision
Which project state?
  → Governed through BoundedCommand lifecycle
Through which skill/session?
  → Tracked through SkillSession + ActorScope
Under which approval?
  → Enforced through ApprovalState checks
With what evidence?
  → Appended through AuditEvent hash chain
How is it restored?
  → SnapshotManifest + RollbackEngine
How are requirements captured?
  → Structured Intake System (95% deterministic, 5% AI-assisted)
```

This is the heartbeat. This is the soul. This is **RealmForge Core**.

---

*"No unscoped agents. No invisible work. No ungoverned runtime behavior."*
