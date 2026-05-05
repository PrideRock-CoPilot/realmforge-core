---
doc_id: DOC-SPEC-015
title: Authority Core Spec
status: draft
owner: cto
reviewers: [domain-architect, security-architect, backend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: authority-core
work_path_ids: [WP-CORE-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-MODULE-AUTHORITY-CORE]
visual_edge_ids: []
approval_state: pending
---

# Authority Core Spec

## Responsibility

Authority Core answers who may do what, to which app state, through which grant and packet, under which approval, with what evidence, and how that state can be restored.

## Required Services

| Service | Public operations |
| --- | --- |
| Identity Service | `issue_session`, `renew_session`, `revoke_session`, `validate_session` |
| Scope Service | `build_actor_scope`, `refresh_scope`, `validate_scope` |
| Skill Grant Service | `issue_grant`, `activate_grant`, `suspend_grant`, `revoke_grant`, `validate_grant` |
| Command Service | `propose_command`, `authorize_command`, `apply_command`, `deny_command` |
| Audit Service | `append_event`, `query_events`, `verify_chain` |
| Snapshot Service | `create_snapshot`, `validate_snapshot`, `compare_snapshots`, `get_latest_known_good` |
| Rollback Service | `preview_rollback`, `execute_rollback`, `verify_rollback` |
| Work Packet Service | `generate_packet`, `validate_packet`, `submit_evidence`, `complete_packet` |

## Completion Gates From Current Core

- Actor scope must load real session, roles, allowed actions, tenant, project, app, and grant state.
- Skill session activation must use caller tenant and project values.
- Session renewal must persist new expiry.
- Snapshot validation must check manifest hash, object refs, table exports, and Parquet dataset hashes.
- Rollback execution must require preview acknowledgement and create verification evidence.
- Work packet generation must persist packet scope and derive boundaries from work path/catalog data.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-CORE-001` |
| Visual node IDs | `VN-MODULE-AUTHORITY-CORE` |
| Visual edge IDs | `VE-GATEWAY-TO-AUTHORITY`, `VE-AUTHORITY-TO-POSTGRES` |
