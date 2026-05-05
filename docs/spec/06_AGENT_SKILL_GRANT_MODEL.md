---
doc_id: DOC-SPEC-006
title: Agent Skill Grant Model
status: draft
owner: security-architect
reviewers: [cto, domain-architect, api-architect, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: agent-governance
work_path_ids: [WP-SKILL-001]
related_decision_ids: [DEC-COUNCIL-003]
related_file_ids: []
visual_node_ids: [VN-SKILL-GRANT-MODEL]
visual_edge_ids: []
approval_state: pending
---

# Agent Skill Grant Model

## Core Rule

An agent has zero capabilities until an active skill grant gives it bounded capabilities. Skill text is descriptive; skill grants are enforceable state.

## Skill Grant Fields

| Field | Type | Rule |
| --- | --- | --- |
| `skill_grant_id` | typed ID | Prefix `sgr_` |
| `agent_id` | typed ID | Must identify active agent actor |
| `skill_id` | typed ID | Must identify approved skill template |
| `tenant_id` | typed ID | Required |
| `app_id` | typed ID | Required for implementation actions |
| `allowed_tools` | string list | Tool names exactly as gateway exposes them |
| `allowed_actions` | string list | Bounded command action names |
| `allowed_file_patterns` | string list | App-relative globs |
| `denied_file_patterns` | string list | Takes precedence over allow list |
| `allowed_postgres_views` | string list | Views only, not base tables |
| `allowed_parquet_datasets` | string list | Dataset IDs only |
| `max_tokens` | integer | Hard budget |
| `max_seconds` | integer | Hard time budget |
| `requires_human_approval` | boolean | True for high-risk grants |
| `separation_group` | string | Blocks conflicting duties |
| `expires_at` | timestamp | Required |
| `state` | enum | See `03_UBIQUITOUS_LANGUAGE.md` |

## Standard Skill Grants

| Grant ID | Skill | Owns | Denied |
| --- | --- | --- | --- |
| `SGL-BACKEND-DOMAIN` | backend | Domain model source files | frontend assets, runtime bundle signing |
| `SGL-BACKEND-POLICY` | backend | Policy engine source files | UI, catalog approval, release signoff |
| `SGL-BACKEND-GATEWAY` | backend | Agent gateway source files | Postgres direct mutation outside store |
| `SGL-DATA-POSTGRES` | data-engineer | Migration specs and store queries | API handlers, frontend |
| `SGL-DATA-PARQUET` | data-engineer | Parquet schema and exports | Policy decisions |
| `SGL-FRONTEND-SHELL` | frontend | Human-facing UI files after Council decision | backend crates, migrations |
| `SGL-QA-VERIFY` | qa | Test definitions and verification evidence | product acceptance changes |
| `SGL-RELEASE` | release-manager | Bundle verification and release gates | implementation files |
| `SGL-SECURITY-REVIEW` | security-architect | Threat model and policy review | direct code mutation |

## Separation Of Duties

- An agent cannot both implement and approve the same high-risk work packet.
- An agent cannot both write policy enforcement and certify the security review for that policy.
- An agent cannot both create a runtime bundle and mark it release approved.
- Frontend skill grant cannot edit backend crates.
- Backend skill grant cannot edit frontend implementation files.
- Data skill grant cannot change product acceptance criteria.

## Agent Action Mapping

Every agent action must produce:

| Agent action | Skill grant | Bounded command | Audit event | Evidence record | Rollback anchor |
| --- | --- | --- | --- | --- | --- |
| Request work packet | active grant | `command.packet.request` | `agent.packet.requested` | `evidence.packet.scope` | latest app snapshot |
| Read catalog slice | active grant | `command.catalog.read` | `agent.catalog.read` | `evidence.catalog.access` | current catalog snapshot |
| Propose file mutation | active grant | `command.file.propose_update` | `agent.file.update_proposed` | `evidence.file.diff` | pre-packet snapshot |
| Submit tests | active grant | `command.evidence.submit_test` | `agent.evidence.submitted` | `evidence.test.result` | packet snapshot |
| Propose remediation | live-watch grant | `command.remediation.propose` | `watch.remediation.proposed` | `evidence.watch.signal` | live bundle snapshot |

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-SKILL-001` |
| Visual node IDs | `VN-SKILL-GRANT-MODEL` |
| Visual edge IDs | `VE-GRANT-POWERS-AGENT`, `VE-GRANT-LIMITS-PACKET` |
