---
doc_id: DOC-SPEC-012
title: Build Watch Product Spec
status: draft
owner: backend
reviewers: [qa, security-architect, pm, release-manager]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: build-watch
work_path_ids: [WP-BUILD-WATCH-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-MODULE-BUILD-WATCH]
visual_edge_ids: []
approval_state: pending
---

# Build Watch Product Spec

## Responsibility

Build Watch monitors construction-time behavior: packets, file changes, command approvals, tests, policy denials, cost, snapshots, evidence, and unauthorized attempts.

## Event Types

| Event type | Severity | Required fields |
| --- | --- | --- |
| `build.packet.started` | info | `packet_id`, `agent_id`, `skill_grant_id` |
| `build.file.changed` | info | `file_id`, `packet_id`, `diff_hash` |
| `build.file.scope_denied` | critical | `file_path`, `packet_id`, `denial_code` |
| `build.test.completed` | info | `test_id`, `status`, `duration_ms` |
| `build.policy.denied` | high | `command_id`, `denial_code`, `actor_id` |
| `build.cost.threshold_crossed` | medium | `packet_id`, `cost_type`, `budget`, `actual` |
| `build.snapshot.created` | info | `snapshot_id`, `reason`, `parent_snapshot_id` |
| `build.evidence.missing` | high | `packet_id`, `required_evidence_type` |

## Required Alerts

- Unauthorized file change attempt.
- Packet executing without active skill grant.
- Required test missing.
- Required trace point missing.
- Cost threshold crossed.
- Snapshot anchor missing.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-BUILD-WATCH-001` |
| Visual node IDs | `VN-MODULE-BUILD-WATCH` |
| Visual edge IDs | `VE-BUILD-WATCH-TO-EVIDENCE` |
