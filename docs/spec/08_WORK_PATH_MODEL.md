---
doc_id: DOC-SPEC-008
title: Work Path Model
status: draft
owner: domain-architect
reviewers: [pm, cto, qa, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: work-paths
work_path_ids: [WP-WORKPATH-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-WORKPATH-MODEL]
visual_edge_ids: []
approval_state: pending
---

# Work Path Model

## Work Path Contract

A work path is a directed graph that turns product intent into scoped implementation. Every implementation packet must come from an approved work path node.

## Node Types

`capability`, `module`, `screen`, `component`, `api_route`, `service`, `database_table`, `parquet_dataset`, `file`, `environment_variable`, `permission`, `policy`, `test`, `trace_point`, `documentation`, `agent_packet`, `decision`, `risk`, `approval_gate`, `release_gate`, `runtime_bundle`, `watch_signal`.

## Edge Types

`depends_on`, `calls`, `renders`, `reads_from`, `writes_to`, `protects`, `configures`, `tests`, `documents`, `requires_decision`, `modifies_file`, `creates_file`, `uses_file`, `blocked_by`, `owned_by`, `emits_trace`, `requires_policy`, `anchors_snapshot`.

## Required Work Path Fields

```yaml
work_path_id:
name:
tenant_id:
app_id:
status:
objective:
owner:
risk_level:
nodes: []
edges: []
required_skill_grants: []
required_approvals: []
required_tests: []
required_trace_points: []
rollback_anchor_policy:
visual_node_id:
```

## Packet Generation Rules

- One packet maps to one primary work path node.
- A packet can read ancestor context and explicitly linked sibling context.
- A packet can write only files linked by `creates_file` or `modifies_file`.
- Required tests and trace points copy from node links.
- Every packet receives the latest known-good snapshot for its rollback anchor.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-WORKPATH-001` |
| Visual node IDs | `VN-WORKPATH-MODEL` |
| Visual edge IDs | `VE-WORKPATH-GENERATES-PACKET` |
