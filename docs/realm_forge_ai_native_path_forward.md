# Realm Forge Path Forward: AI-Native Software Construction, Build, Runtime, and Rollback

**Document purpose:**  
This document defines the path forward for Realm Forge as an AI-native software construction platform. It is intended to be uploaded to Codex or another coding/planning agent so it can generate deeper implementation plans, architecture documents, schemas, and phased build tasks.

**Core thesis:**  
Git made human software collaboration scalable. Realm Forge should make AI software execution governable.

Realm Forge is not just an AI coding assistant, low-code builder, code generator, or project board. It is a new software construction substrate where humans describe and inspect systems through familiar visual interfaces, while agents operate through a backend designed specifically for scoped context, permissions, traceability, rollback, runtime governance, and cost control.

---

## 1. Problem Statement

Current AI development tools are mostly trying to place AI into human-native software workflows.

Typical current workflow:

```text
Human writes prompt
AI edits repo files
AI opens or modifies Git diffs
Human reviews code changes
CI runs tests
Humans infer whether the intent was satisfied
Rollback happens through Git
```

This is useful, but it is not AI-native.

Git is excellent for human teams. It tracks text changes, branches, commits, merges, tags, and distributed history. But Git does not natively understand:

```text
intent
capability ownership
work paths
agent permissions
runtime policies
semantic rollback
test evidence
trace points
feature-level state
agent cost
approved execution scope
blast radius
```

Git knows that a line changed.

Realm Forge should know:

```text
This auth-session behavior changed because this work path was expanded by this agent,
under this approval, affecting these routes, covered by these tests, with these runtime
trace points, tied to this rollback anchor.
```

The issue today is that companies are trying to replace human processes with AI. Realm Forge should instead define processes that work for AI while delivering the same or better results for humans.

---

## 2. Realm Forge Vision

Realm Forge should provide:

```text
Human-facing familiar UI
+
AI-native backend substrate
+
Database-governed execution
+
Snapshot-based true rollback
+
Runtime-cost-aware hosting
+
Semantic project state
```

Humans should see:

```text
canvas
modules
screens
flows
mockups
work paths
file viewers
approval dashboards
runtime traces
cost reports
```

Agents should see:

```text
scoped work packets
allowed files
forbidden files
relevant contracts
required tests
trace points
rollback anchors
standards rules
permissions
cost budgets
```

Security should see:

```text
agent credentials
policy coverage
locked files
review gates
risk nodes
runtime violations
audit evidence
signed bundles
```

Runtime should see:

```text
signed snapshot
route table
policy table
contract table
handler binding table
trace profile table
rollback parent
```

Finance should see:

```text
token spend
storage spend
build spend
runtime spend
rework spend
cost per task
cost per flow
reuse savings
```

The same governed project state should render different lenses for different consumers.

---

## 3. Core Architecture Principle

The source of truth should not be only the codebase.

The source of truth should be governed project state.

Old model:

```text
Codebase is source of truth.
Docs describe code.
Agents edit code.
Tests check code.
Deploy ships code.
```

Realm Forge model:

```text
Governed project state is source of truth.
Code is rendered, managed, or linked from state.
Agents operate on scoped state.
Tests emit evidence against state.
Runtime hosts approved state.
Rollback restores state.
```

Code remains important, but it becomes one rendering of the governed state.

Other renderings include:

```text
runtime parquet tables
agent work packets
human visual maps
documentation
test plans
trace profiles
build manifests
rollback manifests
cost reports
security reports
```

---

## 4. The Living Map

Realm Forge should maintain a living map of the software system.

The map is created as the user describes what they want. It should not require the user to manually build architecture diagrams. The conversation itself becomes the design tool.

User says:

```text
I need an app where teachers create assignments, students submit work, and admins review usage.
```

Realm Forge extracts:

```text
actors
capabilities
screens
workflows
data objects
permissions
routes
tests
risks
open decisions
```

Then it creates a draft map.

If the user says:

```text
Actually, there is no frontend. It is only an API and scheduled processing.
```

Realm Forge changes visualization mode from mockup/web-app view to workflow/API architecture view.

The map should adapt to project type.

---

## 5. Map Modes

Realm Forge needs multiple map modes.

### 5.1 Web App Mode

For apps with a UI.

Shows:

```text
screens
routes
components
API calls
state
permissions
database objects
tests
files
```

Example:

```text
Login Page -> Dashboard -> Settings
     |           |          |
     v           v          v
 POST /login  GET /metrics PUT /profile
```

### 5.2 API/System Mode

For backend-only systems.

Shows:

```text
endpoints
services
workers
queues
databases
events
policies
tests
```

Example:

```text
POST /jobs
   -> Job Service
   -> Queue
   -> Worker
   -> Result Store
   -> Audit Log
```

### 5.3 Workflow Mode

For business process systems.

Shows:

```text
actors
steps
approvals
decisions
handoffs
exceptions
SLA/rules
```

Example:

```text
Request Submitted
   -> Manager Review
   -> Security Review
   -> Approval
   -> Provision Access
   -> Notify User
```

### 5.4 Document/Knowledge Mode

For knowledge systems.

Shows:

```text
sources
ingestion
chunking
indexing
retrieval
answer generation
citation
review
governance
```

### 5.5 Platform/OS Mode

For complex platforms such as Realm Forge itself.

Shows:

```text
subsystems
capabilities
services
contracts
data stores
runtime boundaries
security zones
agent workflows
```

Example:

```text
Realm Forge Platform
├── Projects
├── Work Paths
├── Planning Module
├── Module Library
├── Skills / Guilds
├── Boards / War Table
├── Council Review
├── Knowledge Base
├── Memory
├── Execution Engine
├── Artifact Registry
├── File Registry
├── Audit Log
└── Release Governance
```

---

## 6. Work Paths

A Work Path is a visual, structured, reusable feature blueprint.

A work path node can represent:

```text
feature
capability
screen
component
API route
service
database table
JSON/config file
environment variable
permission
session policy
CORS policy
test
documentation
agent task
decision
risk
release gate
runtime policy
trace point
```

A work path edge can represent:

```text
depends_on
calls
renders
reads_from
writes_to
protects
configures
tests
documents
requires_decision
modifies_file
creates_file
uses_file
blocked_by
owned_by
```

Example Login Work Path:

```text
Login Screen
  -> Login Form Component
  -> POST /auth/login
  -> Auth Service
  -> User Store
  -> Session Store
  -> Permission Context
  -> Audit Event
  -> Tests
  -> Docs
```

The user clicks Login and sees what is involved. The agent receives the exact scoped construction path.

---

## 7. Work Path Node Example

```yaml
id: node.auth.login.screen
type: screen
name: Login Screen
capability: Authentication
route: /login
purpose: Allow users to authenticate
status: planned

files:
  creates:
    - apps/web/src/routes/login/LoginPage.tsx
    - apps/web/src/features/auth/LoginForm.tsx
  modifies:
    - apps/web/src/router.tsx

contracts:
  consumes:
    - contract.auth.login.request
    - contract.auth.login.response

tests:
  required:
    - tests/web/auth/LoginForm.test.tsx

agents:
  allowed:
    - Frontend Agent
    - QA Frontend Agent
  forbidden:
    - Database Agent

risks:
  - leaking auth errors
  - broken redirect handling
```

---

## 8. File Awareness

Every file should know why it exists and who uses it.

Clicking a file should show:

```text
path
language
owner capability
owner area
risk level
lock state
generated/managed/hand-authored state
used by nodes
used by routes
required tests
trace points
allowed agents
last approved hash
last modified actor
rollback snapshots
```

Example:

```yaml
id: file.apps.api.src.services.auth.session_service
type: file
path: apps/api/src/services/auth/session_service.py
owner_capability: Authentication
risk_level: high
used_by:
  - node.auth.login.api
  - node.auth.logout.api
  - node.auth.me.api
  - node.auth.middleware.permission_guard
tests:
  - tests/api/auth/test_session_service.py
  - tests/api/auth/test_login_flow.py
allowed_modifiers:
  - Backend Agent
  - Security Agent
requires_review:
  - Security
  - Backend
```

This gives agents blast-radius awareness.

---

## 9. Planning Module

The Planning Module turns user intent into an agent-ready execution map before implementation starts.

It creates:

```text
what is being built
why it exists
which system area owns it
which routes are involved
which files are created or modified
which database objects are required
which tests must exist first
which agents/skills are allowed to act
which assumptions are locked
which open questions block execution
which reusable patterns/modules already exist
what exact execution sequence should happen
```

Planning becomes a first-class artifact, not a throwaway markdown file.

---

## 10. Planning Session Lifecycle

```text
Draft
Mapped
Reviewed
Approved
In Execution
Partially Complete
Blocked
Completed
Released
Archived
```

Execution should not begin until gates are satisfied:

```yaml
gates:
  intake_complete: true
  system_scan_complete: true
  route_map_complete: true
  file_touch_map_complete: true
  test_plan_complete: true
  risk_review_complete: true
  council_review_complete: true
  user_signoff_complete: true
```

---

## 11. Core Enforcement Rules

Realm Forge should enforce:

```text
No agent may modify a file not listed in the approved file touch map.
No agent may skip a planned test.
No agent may answer a blocking open question unless authorized.
No implementation may begin before the plan is approved.
No completed step may be marked done without proof.
No route may enter runtime without policy.
No handler may run without approved binding.
No high-risk file may release without review state.
No runtime bundle may deploy unsigned state.
```

---

## 12. Agent-Native Execution

Agents should not receive the whole repo and a giant prompt.

They should receive scoped work packets.

Example:

```yaml
agent_work_packet:
  agent: backend_agent
  node: node.auth.login.api
  credential_scope: backend_auth_login_write
  objective: Implement login API handler
  can_read:
    - work_path_nodes
    - route_contracts
    - file_registry
    - test_registry
    - decisions
  can_write:
    - agent_steps
    - implementation_notes
    - evidence_records
  allowed_file_paths:
    - apps/api/src/routes/auth/login.py
    - apps/api/src/services/auth/auth_service.py
    - tests/api/auth/test_login_flow.py
  denied:
    - database/migrations/**
    - apps/web/**
    - .env
    - security/policies/**
  required_contracts:
    - contract.auth.login.request.v1
    - contract.auth.login.response.v1
  required_trace_points:
    - trace.auth.login.user_lookup
    - trace.auth.login.session_create
  rollback_anchor:
    - snapshot.auth.login.pre_agent_step_004
```

Agents should operate inside the smallest possible permissioned universe.

---

## 13. Database-Backed Permissions

Realm Forge should use database-backed permissions instead of prompt-only rules.

Each agent role should have scoped credentials and views.

Examples:

```text
frontend_agent_view
backend_auth_agent_view
qa_agent_view
security_reviewer_view
release_manager_view
```

A backend auth agent should not merely be instructed to avoid frontend files. It should not have write access to frontend files in the project-state database or through the execution layer.

---

## 14. Parquet Source of Truth and Snapshot Layer

Parquet is useful here because it supports fast, queryable, column-oriented snapshots of project state.

Realm Forge should use Parquet snapshots to store immutable time-indexed states.

Snapshot contents:

```text
work paths
nodes
edges
files
routes
tests
decisions
agent permissions
render manifests
evidence runs
trace points
standards checks
violations
lock states
file hashes
cost metadata
```

A snapshot is not just a restore point for source files. It is a restore point for the full governed project brain.

---

## 15. Active Store vs Snapshot Store

Recommended model:

```text
Active working state:
- DuckDB or SQLite
- optimized for current edits, transactions, and local queries

Snapshot/query state:
- Parquet
- immutable, time-indexed, portable, queryable

File contents:
- content-addressed object store
- sha256-addressed exact file versions
```

Directory shape:

```text
.realmforge/
  db/
    realmforge.duckdb

  snapshots/
    snapshot_20260502_183000/
      manifest.yaml
      files.parquet
      work_paths.parquet
      nodes.parquet
      edges.parquet
      routes.parquet
      tests.parquet
      decisions.parquet
      evidence.parquet
      render_manifest.parquet
      standards_report.parquet
      cost_report.parquet

  objects/
    sha256/
      ab/cd/abcd1234...
      ef/90/ef901234...
```

---

## 16. True Rollback

Realm Forge rollback should restore project state, not just source files.

Rollback can operate by:

```text
full project
capability
work path
file
agent step
runtime bundle
```

Example:

```text
Restore Authentication/Login to last known-good green state.
```

The rollback engine should restore:

```text
metadata
map
files
locks
decisions
tests
render manifests
agent task state
runtime policy state
trace expectations
```

Rollback itself should be validated.

After rollback, Realm Forge should run:

```text
metadata consistency checks
file hash verification
standards rules
affected tests
trace replay if available
map status reconciliation
```

---

## 17. Snapshot Manifest Example

```yaml
snapshot:
  id: snapshot_20260502_183000
  created_at: 2026-05-02T18:30:00-05:00
  reason: post_agent_step_green
  project_id: realmforge
  work_path_id: workpath.auth.login
  agent_step_id: step.auth.login.006
  git_commit: optional
  parent_snapshot_id: snapshot_20260502_181500
  status: known_good

contents:
  metadata:
    files: files.parquet
    nodes: nodes.parquet
    edges: edges.parquet
    routes: routes.parquet
    tests: tests.parquet
    decisions: decisions.parquet
    evidence: evidence.parquet
  file_objects:
    object_store: ../objects/sha256

validation:
  tests_passed: true
  standards_passed: true
  council_reviewed: true
  user_approved: false
```

---

## 18. Runtime and Build Model

Realm Forge should introduce a new type of build.

Not just:

```text
npm build
```

But:

```text
realmforge build
```

This build understands:

```text
what must become compiled source
what should be generated
what should be managed
what should remain parquet-backed runtime data
what should be excluded
what should be signed
what needs tests
what needs rollback metadata
what needs runtime policy
```

---

## 19. Artifact Classification

Every artifact should have a build classification.

```yaml
artifact_class:
  - compiled_source
  - generated_source
  - managed_source
  - runtime_definition
  - runtime_policy
  - runtime_contract
  - runtime_trace_profile
  - static_asset
  - documentation
  - test_only
  - excluded
```

Compile when:

```text
it requires framework compilation
it contains custom business logic
it needs static type checking
it is performance-critical
it directly handles secrets or security primitives
it talks to external systems through adapters
it uses language-specific libraries
```

Keep in Parquet when:

```text
it is declarative
it changes often
it benefits from runtime governance
it needs fast rollback
it should be queryable
it is policy/contract/workflow/map metadata
it should be used by humans and agents
```

Hybrid when:

```text
the shape is stable but behavior needs code
the runtime needs both metadata and compiled handler logic
```

---

## 20. Governed Runtime Bundle

Realm Forge build should output a governed runtime bundle.

Example:

```text
app_20260502_190000.rfbundle
```

Inside:

```text
compiled/
runtime_state/
policies/
contracts/
traces/
evidence/
rollback/
docs/
manifest/
```

Example expanded output:

```text
dist/
  app/
    compiled/
      web/
      api/
      worker/

  runtime/
    realmforge-runtime-config.yaml
    active_snapshot.yaml

    parquet/
      routes.parquet
      flows.parquet
      policies.parquet
      permissions.parquet
      contracts.parquet
      trace_points.parquet
      feature_flags.parquet
      file_registry.parquet
      work_path_nodes.parquet
      work_path_edges.parquet
      standards.parquet

  evidence/
    build_report.parquet
    standards_report.parquet
    test_report.parquet
    dependency_report.parquet
    cost_report.parquet

  docs/
    system_map.html
    capability_authentication.html
    release_notes.md

  manifest/
    build_manifest.yaml
    render_manifest.yaml
    rollback_manifest.yaml
```

---

## 21. Runtime Engine

The runtime engine should load the signed bundle and enforce the parquet-defined governance model.

Runtime responsibilities:

```text
load active snapshot manifest
validate hashes/signature
open parquet runtime tables
build hot indexes
match routes
validate contracts
enforce policies
call approved compiled handlers
emit trace evidence
support snapshot/bundle rollback
expose safe runtime status to Realm Forge UI
```

The runtime should execute approved primitives, not arbitrary instructions.

Good:

```text
flow step type: call_registered_handler
handler: auth.login
```

Bad:

```text
flow step type: execute_code
code: "whatever the parquet says"
```

Do not allow arbitrary code execution from Parquet.

---

## 22. Runtime Request Flow

Example:

```text
POST /auth/login
```

Runtime process:

```text
1. Request arrives
2. Runtime resolves route from routes.parquet
3. Runtime validates request using contract table
4. Runtime checks policy table
5. Runtime resolves approved handler binding
6. Runtime calls compiled handler
7. Runtime emits trace events
8. Runtime validates response contract
9. Runtime writes evidence
10. Runtime returns response
```

---

## 23. Handler Binding Example

```yaml
handler_binding:
  id: handler.auth.login
  runtime_module: auth
  function: login
  allowed_inputs:
    - contract.auth.login.request
  allowed_outputs:
    - contract.auth.login.response
  required_permissions:
    - runtime.auth.login.execute
```

Parquet chooses among approved handlers. It does not create new executable code.

---

## 24. Runtime Security Rules

The runtime should refuse to boot if:

```text
snapshot is unsigned
manifest hash fails
required tables are missing
approved status is false
schema version is incompatible
critical policies are missing
handler bindings do not match compiled handlers
```

Policy must run before handler execution.

Correct:

```text
request -> route -> contract -> policy -> handler
```

Wrong:

```text
request -> handler -> maybe policy later
```

---

## 25. Runtime Tracing

Every flow should define planned trace points.

For login:

```text
request_received
contract_validated
policy_checked
user_lookup_started
user_lookup_completed
password_verified
session_created
audit_written
response_returned
```

Trace events must be safe and redacted.

Example redaction policy:

```yaml
redaction:
  forbidden_fields:
    - password
    - password_hash
    - token
    - session_secret
    - api_key
  allowed_fields:
    - user_found
    - status
    - duration_ms
    - policy_result
```

The map should show live runtime behavior:

```text
Login Flow
  request_received: passed
  contract_validated: passed
  policy_checked: passed
  user_lookup: passed
  password_verified: failed
  session_created: skipped
```

---

## 26. Lightweight Build and Reduced Dependency Surface

Realm Forge should reduce dependency bloat by moving repeatable/declarative behavior into governed runtime state instead of dependency-heavy code.

Move these into runtime parquet when safe:

```text
routes
navigation
permissions
policies
feature flags
form definitions
workflow steps
validation rules
trace points
API contracts
agent permissions
file ownership
release gates
standards rules
```

Keep real code for:

```text
UI rendering
custom business logic
database adapters
auth primitives
encryption/hash functions
external integrations
performance-sensitive services
```

Rule:

```text
Do not add a dependency when a signed runtime definition can safely describe the behavior.
Do not compile what can be governed.
Do not ship what can be queried.
Do not trust what can be validated.
```

---

## 27. Runtime Update Modes

Some changes require rebuilds. Others can be parquet-only runtime updates.

Parquet-only safe changes may include:

```text
permissions
feature flags
trace verbosity
some policies
navigation
rate limits
workflow ordering if no new handler is required
```

Compiled-code changes include:

```text
new handler logic
password hashing logic
database adapter changes
custom business logic
new external service integration
UI component changes
```

Build decision matrix:

```text
Change type                       Build action
------------------------------------------------------------
React screen changed              Recompile frontend
API handler changed               Recompile backend
Contract changed                  Regenerate types + tests + maybe compile
Policy changed                    Parquet snapshot only
Permission changed                Parquet snapshot only
Workflow step changed             Parquet snapshot, unless new handler needed
New handler binding               Compile/register handler + parquet
Trace point changed               Parquet snapshot only
File ownership changed            Metadata snapshot only
Route path changed                Parquet + generated clients + tests
Database schema changed           Migration + compile impacted services
```

---

## 28. Two Product Examples

Realm Forge must support both simple known-pattern creation and massive AI-native codebase conversion.

---

# Example A: E-Commerce Website With Payment Provider

User request:

```text
I want an e-commerce website with products, cart, checkout, user accounts,
admin product management, and payments through Stripe.
```

Realm Forge should identify this as a known commodity pattern.

Project mode:

```yaml
project_mode: fast_composition
complexity: known_pattern
risk: moderate
module_strategy: reuse_approved_templates
expected_delivery: minutes_to_hours
```

Realm Forge pulls modules:

```text
ECommerce.ProductCatalog
ECommerce.Cart
ECommerce.Checkout
ECommerce.OrderManagement
ECommerce.Payment.Stripe
ECommerce.AdminProducts
Auth.Login
Auth.Session
Notifications.OrderConfirmation
Database.ProductsOrdersUsers
```

Work path:

```text
E-Commerce Site
├── Storefront
│   ├── Home Page
│   ├── Product Listing
│   ├── Product Detail
│   └── Search/Filters
│
├── Cart
│   ├── Add To Cart
│   ├── Update Quantity
│   └── Remove Item
│
├── Checkout
│   ├── Customer Info
│   ├── Payment Provider Session
│   ├── Payment Callback/Webhook
│   └── Order Confirmation
│
├── Admin
│   ├── Product CRUD
│   ├── Inventory
│   └── Order View
│
├── Data
│   ├── users
│   ├── products
│   ├── carts
│   ├── orders
│   ├── order_items
│   └── payment_events
│
├── Security
│   ├── Session Policy
│   ├── Admin Permissions
│   ├── Webhook Signature Validation
│   └── Secret Management
│
└── Tests / Traces / Docs
```

Build classification:

```yaml
compiled_source:
  - storefront UI components
  - checkout handler
  - payment webhook handler
  - database adapter
  - admin product service

runtime_parquet:
  - route registry
  - permission matrix
  - checkout flow definition
  - payment trace points
  - feature flags
  - form schemas
  - product admin workflow
  - audit rules
  - file ownership map
  - standards rules

generated_source:
  - API clients
  - request/response contracts
  - route bindings
  - test skeletons
  - docs
```

Value:

```text
Simple known apps become fast.
The user sees familiar low-code/no-code style flow.
The backend remains governed, traceable, and rollback-safe.
```

---

# Example B: Massive Linux Distribution Upload

User request:

```text
I want to upload a Linux distribution and have AI convert the source into the Realm Forge backend model so it understands everything and can resume from there.
```

Realm Forge should identify this as massive codebase ingestion, not fast app composition.

Project mode:

```yaml
project_mode: massive_codebase_ingestion
complexity: extreme
risk: high
module_strategy: staged_discovery_and_conversion
expected_delivery: long_running_multi_phase
requires_budget_controls: true
requires_storage_controls: true
requires_human_checkpoints: true
```

Correct promise:

```text
Realm Forge converts the Linux source tree into a queryable, governed,
agent-operable project state.
```

Incorrect promise:

```text
Realm Forge instantly rewrites Linux.
```

---

## 29. Massive Ingestion Pipeline

### Phase 0: Intake and Budget Gate

Estimate:

```text
file count
total size
language mix
build systems
binary files
docs
tests
dependency depth
expected storage
expected token budget
expected indexing time
```

Output:

```yaml
ingestion_estimate:
  project_type: operating_system_distribution
  estimated_files: very_high
  estimated_symbols: very_high
  estimated_storage: high
  estimated_token_cost: high
  recommended_mode: staged_ingestion
  immediate_full_conversion_allowed: false
```

### Phase 1: Raw Inventory

Cheap deterministic scan first.

```text
walk file tree
hash files
detect languages
detect build systems
detect configs
detect licenses
detect docs
detect tests
detect generated files
detect binary artifacts
```

Creates:

```text
files.parquet
directories.parquet
file_hashes.parquet
language_inventory.parquet
build_systems.parquet
license_inventory.parquet
binary_inventory.parquet
```

### Phase 2: Build System and Dependency Map

Detect:

```text
Makefiles
Kconfig
CMake
Cargo
package manifests
kernel config dependencies
module boundaries
include paths
compile targets
```

Creates:

```text
build_targets.parquet
compile_units.parquet
include_edges.parquet
config_flags.parquet
dependency_edges.parquet
```

### Phase 3: Symbol Indexing

Extract:

```text
functions
types
structs
macros
constants
modules
exports
imports
syscalls
interfaces
```

Creates:

```text
symbols.parquet
symbol_references.parquet
call_edges.parquet
type_edges.parquet
macro_usage.parquet
exported_interfaces.parquet
```

### Phase 4: Capability/Subdomain Mapping

Group files into capabilities.

Examples:

```text
Scheduler
Memory Management
Virtual File System
Networking
Device Drivers
Process Management
Security
Architecture Layer
Boot
IPC
Filesystems
Power Management
```

Creates:

```text
capabilities.parquet
capability_file_links.parquet
capability_symbol_links.parquet
capability_dependency_edges.parquet
```

### Phase 5: Work Path Reconstruction

Create flows from actual code behavior.

Examples:

```text
Process Creation Flow
Scheduler Tick Flow
Memory Allocation Flow
System Call Entry Flow
File Open Flow
Network Packet Receive Flow
Boot Flow
Module Load Flow
```

Creates:

```text
work_paths.parquet
work_path_nodes.parquet
work_path_edges.parquet
trace_candidates.parquet
```

### Phase 6: Trace Point Candidate Generation

For each flow, propose safe debug points.

```yaml
trace_candidate:
  id: trace.syscall.entry
  flow_id: flow.kernel.syscall_entry
  symbol_id: sym.do_syscall_64
  file_path: arch/x86/entry/common.c
  safe_fields:
    - syscall_number
    - process_id
    - result_code
  forbidden_fields:
    - raw_user_memory
    - secrets
  risk_level: high
```

### Phase 7: Realm Forge Backend Conversion

Create:

```text
file registry
symbol registry
capability graph
work path graph
build graph
test graph
risk map
trace map
ownership map
agent permission map
rollback snapshot
```

The source is not necessarily rewritten. It is made AI-operable.

---

## 30. Token and Cost Control

For massive systems, Realm Forge must not use LLM calls for everything.

Use deterministic scanning first:

```text
file inventory
hashing
language detection
AST parsing
symbol extraction
dependency detection
build graph parsing
```

Use AI where it adds value:

```text
capability naming
flow explanation
risk interpretation
module grouping
documentation synthesis
agent task planning
complex conversion decisions
```

Track:

```text
cost per file
cost per symbol index
cost per capability map
cost per agent task
cost per successful flow
cost per rollback
reuse savings
storage growth
snapshot retention cost
```

AI ROI should be measurable, not hand-waved.

---

## 31. Project Modes

Realm Forge should expose project modes:

```yaml
project_modes:
  fast_composition:
    description: Build from approved reusable modules
    examples:
      - e-commerce site
      - landing page
      - CRUD admin app
      - dashboard
      - approval workflow

  guided_build:
    description: Build custom app through planning/work paths
    examples:
      - internal business app
      - SaaS MVP
      - data portal

  codebase_ingestion:
    description: Load existing codebase into Realm Forge map
    examples:
      - existing monolith
      - legacy app
      - internal platform

  massive_system_ingestion:
    description: Large-scale semantic indexing and AI-operability conversion
    examples:
      - OS source tree
      - enterprise platform
      - large multi-language repo

  runtime_bundle:
    description: Build governed runtime bundle from approved state
```

---

## 32. V1 MVP Recommendation

Do not build everything first.

Build one vertical slice that proves the whole loop.

Recommended first flow:

```text
Authentication/Login
```

Why login?

```text
It touches frontend, backend, database, security, sessions, policies, traces, tests, docs, and runtime.
```

MVP should demonstrate:

```text
User adds Login module
System creates work path map
System creates parquet-backed project state
System classifies compiled vs runtime artifacts
Agent receives scoped work packet
Agent implements allowed files
Tests emit trace evidence
Build creates signed bundle
Runtime serves login using compiled handler + parquet policies/contracts/traces
User clicks Login flow and sees runtime evidence
Rollback restores last known-good login state
```

---

## 33. V1 Components

### 33.1 Project State Store

```text
DuckDB or SQLite active state
Parquet snapshot export
content-addressed object store
snapshot manifest
```

### 33.2 Work Path Graph

```text
work_paths
work_path_nodes
work_path_edges
node_files
node_routes
node_tests
node_trace_points
```

### 33.3 File Registry

```text
files
file_hashes
file_ownership
file_lock_state
file_build_classification
file_agent_permissions
```

### 33.4 Agent Work Packet Generator

```text
scope generation
allowed files
denied files
required contracts
required tests
required trace points
rollback anchor
cost budget
```

### 33.5 Build Classifier

```text
compiled_source
generated_source
managed_source
runtime_parquet
documentation
test_only
excluded
```

### 33.6 Runtime Bundle Builder

```text
compiled app outputs
runtime parquet tables
build manifest
render manifest
rollback manifest
signatures/hashes
evidence reports
```

### 33.7 Runtime Engine

```text
load signed bundle
query parquet route/policy/contract/trace tables
call approved compiled handlers
emit evidence
support rollback by switching bundle/snapshot
```

### 33.8 Human UI

```text
map canvas
flow inspector
file viewer
runtime evidence view
agent task view
rollback view
cost view
```

---

## 34. Suggested Initial Schema Tables

```text
projects
planning_sessions
capabilities
work_paths
work_path_nodes
work_path_edges
files
file_snapshots
file_node_links
routes
route_node_links
contracts
policies
permissions
handlers
handler_bindings
tests
test_node_links
trace_points
trace_profiles
evidence_runs
evidence_events
agent_roles
agent_permissions
agent_work_packets
standards
standard_violations
build_artifacts
build_manifests
runtime_bundles
snapshots
rollback_events
cost_events
```

---

## 35. Suggested Commands

```text
realmforge init
realmforge intake
realmforge plan
realmforge map
realmforge snapshot
realmforge packet create
realmforge validate
realmforge build
realmforge runtime boot
realmforge runtime switch
realmforge rollback
realmforge inspect flow
realmforge inspect file
realmforge ingest
realmforge ingest estimate
realmforge ingest inventory
realmforge ingest symbols
realmforge ingest capabilities
```

---

## 36. Codex Planning Tasks

Codex should expand this document into implementation tasks in this order.

### Phase 1: Core State and Snapshot Foundation

Deliver:

```text
.realmforge directory structure
DuckDB/SQLite schema
Parquet snapshot writer
snapshot manifest
content-addressed object store
file inventory scanner
basic CLI
```

### Phase 2: Work Path and File Registry

Deliver:

```text
work path schema
node/edge schema
file registry
file-to-node links
route/test/trace links
human-readable exports
```

### Phase 3: Agent Work Packets

Deliver:

```text
agent role model
permission scopes
work packet generator
allowed/denied file manifests
required test/trace/contract lists
rollback anchors
```

### Phase 4: Build Classifier

Deliver:

```text
artifact classification rules
compiled/generated/runtime_parquet split
build manifest
render manifest
validation rules
standards engine
```

### Phase 5: Runtime Bundle

Deliver:

```text
runtime bundle directory format
runtime parquet export
manifest signing/hashing
boot validation report
rollback manifest
```

### Phase 6: Runtime Engine V1

Deliver:

```text
one-language runtime target
route lookup
contract validation
policy validation
approved handler registry
trace emission
evidence writing
snapshot/bundle switch
```

Recommended first runtime language:

```text
Rust
```

Reason:

```text
strong typing
memory safety
good async support
good service performance
good CLI/server deployment
good fit for security-sensitive runtime
```

### Phase 7: Human UI

Deliver:

```text
map canvas
flow inspector
file viewer
trace viewer
build report viewer
rollback control
cost dashboard
```

### Phase 8: Fast Composition Demo

Deliver:

```text
e-commerce module templates
login module
cart module
checkout module
payment-provider module
admin module
build/runtime demo
```

### Phase 9: Massive Ingestion Demo

Deliver:

```text
large repo ingestion estimator
raw file inventory
build graph extractor
symbol indexing
capability mapper
flow reconstruction
agent packet from ingested codebase
```

---

## 37. Non-Negotiable Guardrails

```text
Do not rely on prompt instructions for security boundaries.
Do not allow arbitrary code execution from Parquet.
Do not let agents modify files outside approved work packets.
Do not deploy unsigned runtime bundles.
Do not store secrets in trace events.
Do not use LLM calls for deterministic scans that normal parsers can handle.
Do not let runtime policy checks happen after handler execution.
Do not treat Git rollback as true rollback.
Do not make one giant markdown file the source of truth.
Do not make all source files generated-only; support generated, managed, hand-authored, hybrid, and locked files.
```

---

## 38. Product Positioning

Realm Forge is not:

```text
just an AI coding assistant
just low-code
just no-code
just a code generator
just a repo scanner
just a Git alternative
just a deployment tool
```

Realm Forge is:

```text
an AI-native software construction and runtime governance substrate
```

Possible positioning statement:

```text
Git made human software collaboration scalable.
Realm Forge makes AI software execution governable.
```

Sharper:

```text
Git tracks what humans changed.
Realm Forge governs what agents are allowed to change, why they changed it,
how it was proven, and how to restore it.
```

---

## 39. Success Criteria

Realm Forge is successful if it can prove:

```text
Simple known apps become fast to create.
Massive systems become navigable.
Agents stop guessing.
Humans stop drowning.
Security stops relying on prompt obedience.
Rollback restores complete governed state.
Runtime behavior is traceable.
Costs are measurable per flow, task, and capability.
Dependency bloat is reduced by moving declarative behavior into governed runtime definitions.
```

---

## 40. Final North Star

No unscoped agents.  
No invisible work.  
No ungoverned runtime behavior.  
No mystery builds.  
No rollback by prayer.  
No dependency bloat by default.  
No AI spend without measurable value.

Realm Forge should let:

```text
Humans design visually.
Agents execute through scoped data.
Security enforce policy through state.
Builds produce governed runtime bundles.
Runtime behavior emit trace evidence.
Rollback restore the whole known-good universe.
Costs become visible.
```

That is the path forward.
