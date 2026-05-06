---
doc_id: DOC-SPEC-021
title: Acceptance Test Plan
status: draft
owner: qa
reviewers: [pm, cto, security-architect, release-manager]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: quality
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-ACCEPTANCE-TESTS]
visual_edge_ids: []
approval_state: pending
---

# Acceptance Test Plan

## Documentation Tests

| Test ID | Assertion |
| --- | --- |
| `TEST-DOCS-METADATA-001` | Every spec doc has required front matter. |
| `TEST-DOCS-OPEN-DECISIONS-001` | All unresolved choices appear in `22_OPEN_DECISIONS.md`. |
| `TEST-DOCS-FILE-REGISTRY-001` | Every implementation file referenced by a spec has a registry entry. |
| `TEST-DOCS-VISUAL-MAP-001` | Every product module has visual node and work path IDs. |

## Governance Tests

| Test ID | Assertion |
| --- | --- |
| `TEST-SKILL-GRANT-001` | Agent without active grant has zero capabilities. |
| `TEST-SOD-001` | Implementer cannot approve own high-risk packet. |
| `TEST-GATEWAY-001` | Gateway denies file outside packet scope. |
| `TEST-AUDIT-CHAIN-001` | Tampered audit chain fails verification. |
| `TEST-SNAPSHOT-001` | Snapshot validates manifest, objects, table exports, and Parquet hashes. |

## Product Tests

| Test ID | Assertion |
| --- | --- |
| `TEST-CATALOG-001` | Tenant copies approved global module with provenance. |
| `TEST-WORKPATH-001` | Work path node generates scoped packet. |
| `TEST-KNOWLEDGE-001` | Knowledge answer cites governed records and respects grant scope. |
| `TEST-BUILD-WATCH-001` | Build Watch records unauthorized file attempt. |
| `TEST-BUNDLE-001` | Runtime bundle refuses invalid signature. |
| `TEST-LIVE-RUNTIME-001` | Live Runtime execution passes policy check, writes an audit event, and anchors a snapshot before reporting success. |
| `TEST-LIVE-WATCH-001` | Live Watch proposes remediation packet for repeated latency signal. |
| `TEST-LOGIN-E2E-001` | Login vertical completes catalog to rollback loop. |

## Verification Evidence

| Test ID | Evidence | Status |
| --- | --- | --- |
| `TEST-CATALOG-001` | `cargo test -p control-service --test catalog_integration --target-dir target-quality` passed on 2026-05-05. | passed |
| `TEST-WORKPATH-001` | `cargo test -p control-service --test work_path_integration --target-dir target-quality` passed on 2026-05-05. | passed |
| `TEST-BUNDLE-001` | Runtime-bundle wrong-key and live-runtime invalid-signature tests passed under `cargo test --workspace --target-dir target-quality` on 2026-05-05. | passed |
| `TEST-LIVE-WATCH-001` | `cargo test -p live-watch --target-dir target-quality` passed on 2026-05-05; engine now proposes from current-cycle anomalies only. | passed |
| `TEST-LOGIN-E2E-001` | `cargo test -p control-service --test login_vertical --target-dir target-quality` passed on 2026-05-05 for handler/policy/session/audit/snapshot-anchor flow; catalog-to-rollback E2E remains open. | partial |
| `TEST-INTAKE-LIB-001` | `cargo test --workspace` — intake-engine lib tests pass (28 intake tests included in 244 total). 2026-05-06. | passed |
| `TEST-INTAKE-TREE-001` | All 3 starter trees load and validate: `tree-static-site.json`, `tree-web-app.json`, `tree-api-service.json`. Parsed via `cargo test`. 2026-05-06. | passed |
| `TEST-INTAKE-CONDITION-001` | Condition module tests pass under `cargo test --workspace`. 2026-05-06. | passed |
| `TEST-INTAKE-ENGINE-001` | Engine evaluation tests pass under `cargo test --workspace`. 2026-05-06. | passed |
| `TEST-INTAKE-MAPPER-001` | Feature mapping tests pass under `cargo test --workspace`. 2026-05-06. | passed |
| `TEST-INTAKE-VALIDATE-001` | Tree validation tests pass under `cargo test --workspace`. 2026-05-06. | passed |
| `TEST-INTAKE-ERROR-001` | Error type tests pass under `cargo test --workspace`. 2026-05-06. | passed |
| `TEST-STORE-INTAKE-001` | `cargo test -p control-store --target-dir target-quality` — store compiles with intake module. 2026-05-06. | passed |
 | `TEST-MIGRATION-013` | Migration `013_intake_decision_trees.sql` applies cleanly; seeds 3 trees. `cargo test` covered migration ref. 2026-05-06. | passed |
 | `TEST-CODE-REVIEW-001` | Check engine loads and validates a standards YAML file. | not_yet_run |
 | `TEST-CODE-REVIEW-002` | ExecutableCheck runs an external command and captures exit code. | not_yet_run |
 | `TEST-CODE-REVIEW-003` | RegexCheck finds violations in source files. | not_yet_run |
 | `TEST-CODE-REVIEW-004` | FilePropertyCheck detects oversized files. | not_yet_run |
 | `TEST-CODE-REVIEW-005` | Framework overlay rules supersede base rules on same check ID. | not_yet_run |
 | `TEST-CODE-REVIEW-006` | ReviewReport groups findings by file and sorts by severity. | not_yet_run |
 | `TEST-CODE-REVIEW-007` | Missing `.code-review.yaml` falls back to base language rules. | not_yet_run |
 | `TEST-CODE-REVIEW-008` | Explicit `.code-review.yaml` with framework ID activates overlay. | not_yet_run |
 | `TEST-CODE-REVIEW-009` | ExecutableCheck timeout kills hung processes. | not_yet_run |
 | `TEST-CODE-REVIEW-010` | Security: command whitelist rejects unregistered executables. | not_yet_run |
