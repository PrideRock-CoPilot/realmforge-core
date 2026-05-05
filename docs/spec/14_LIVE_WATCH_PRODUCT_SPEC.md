---
doc_id: DOC-SPEC-014
title: Live Watch Product Spec
status: draft
owner: backend
reviewers: [qa, release-manager, security-architect, pm]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: live-watch
work_path_ids: [WP-LIVE-WATCH-001]
related_decision_ids: [DEC-USER-006]
related_file_ids: []
visual_node_ids: [VN-MODULE-LIVE-WATCH]
visual_edge_ids: []
approval_state: pending
---

# Live Watch Product Spec

## Responsibility

Live Watch monitors approved running apps and creates scoped remediation proposals when it detects meaningful issues. It does not execute fixes in the first governed release.

## Signal Types

| Signal type | Example | Default action |
| --- | --- | --- |
| `latency_micro_blip` | Repeated multi-millisecond latency on one action | propose investigation packet |
| `policy_denial_spike` | Sudden increase in denied runtime policy checks | alert security and propose packet |
| `contract_validation_failure` | Response violates contract but UI handles it | propose bug packet |
| `trace_gap` | Required trace point stops emitting | propose observability packet |
| `cost_anomaly` | Runtime cost exceeds trend | propose cost review packet |
| `error_budget_risk` | Error rate approaches threshold | alert release manager |

## Remediation Proposal Fields

`live_watch_signal_id`, `app_id`, `bundle_id`, `trace_point_ids`, `observed_window`, `severity`, `evidence_record_ids`, `recommended_work_path_id`, `proposed_packet_scope`, `requires_user_approval`.

## Autonomy Rule

Live Watch can create `command.remediation.propose`. It cannot create `command.file.apply_update`, `command.bundle.switch`, or `command.rollback.execute` without a future approved autonomy decision.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-LIVE-WATCH-001` |
| Visual node IDs | `VN-MODULE-LIVE-WATCH` |
| Visual edge IDs | `VE-LIVE-WATCH-TO-GATEWAY` |
