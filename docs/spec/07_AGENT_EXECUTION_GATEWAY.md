---
doc_id: DOC-SPEC-007
title: Agent Execution Gateway
status: draft
owner: security-architect
reviewers: [cto, backend, api-architect, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: agent-governance
work_path_ids: [WP-GATEWAY-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-AGENT-GATEWAY]
visual_edge_ids: []
approval_state: pending
---

# Agent Execution Gateway

## Responsibility

The Agent Execution Gateway is the only path for agent-visible mutation. It validates grant, packet, policy, budget, file scope, schema scope, evidence requirements, and rollback anchors before any write.

## Gateway Flow

```text
agent request
  -> authenticate actor
  -> load active skill grant
  -> load work packet
  -> validate packet state
  -> authorize bounded command
  -> verify file/schema/catalog scope
  -> create audit event
  -> execute allowed operation
  -> require evidence record
  -> anchor to snapshot
```

## Gateway Commands

| Command action | Required grant | Mutates | Evidence required | Audit event |
| --- | --- | --- | --- | --- |
| `command.packet.request` | any active grant | no | `evidence.packet.scope` | `agent.packet.requested` |
| `command.packet.accept` | packet assignee grant | yes | `evidence.packet.acceptance` | `agent.packet.accepted` |
| `command.file.propose_update` | grant with file write | yes | `evidence.file.diff` | `agent.file.update_proposed` |
| `command.file.apply_update` | approved packet grant | yes | `evidence.file.applied` | `agent.file.updated` |
| `command.test.run` | grant with test action | yes | `evidence.test.result` | `agent.test.ran` |
| `command.catalog.copy_module` | catalog grant | yes | `evidence.catalog.copy` | `catalog.module.copied` |
| `command.bundle.build` | runtime bundle grant | yes | `evidence.bundle.build` | `bundle.built` |
| `command.remediation.propose` | live-watch grant | yes | `evidence.watch.signal` | `watch.remediation.proposed` |

## Denial Codes

`GRANT_MISSING`, `GRANT_EXPIRED`, `GRANT_REVOKED`, `PACKET_MISSING`, `PACKET_NOT_ASSIGNED`, `PACKET_SCOPE_DENIED`, `ACTION_DENIED`, `FILE_SCOPE_DENIED`, `SCHEMA_SCOPE_DENIED`, `BUDGET_EXCEEDED`, `EVIDENCE_REQUIRED`, `APPROVAL_REQUIRED`, `SEPARATION_OF_DUTIES_DENIED`, `SNAPSHOT_ANCHOR_REQUIRED`.

## File Access Rules

- Deny list wins over allow list.
- Paths are app-relative.
- Absolute paths are rejected.
- Path traversal is rejected.
- Generated files require `artifact_class: generated_source`.
- High-risk files require human approval before apply.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-GATEWAY-001` |
| Visual node IDs | `VN-AGENT-GATEWAY` |
| Visual edge IDs | `VE-GATEWAY-AUTHORIZES-COMMAND`, `VE-GATEWAY-WRITES-EVIDENCE` |
