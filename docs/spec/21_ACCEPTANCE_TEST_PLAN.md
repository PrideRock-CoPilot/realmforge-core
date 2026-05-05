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
