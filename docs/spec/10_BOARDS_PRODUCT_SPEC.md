---
doc_id: DOC-SPEC-010
title: Boards Product Spec
status: draft
owner: pm
reviewers: [biz-user, frontend, cto, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: boards
work_path_ids: [WP-BOARDS-001]
related_decision_ids: [DEC-COUNCIL-001]
related_file_ids: []
visual_node_ids: [VN-MODULE-BOARDS]
visual_edge_ids: []
approval_state: pending
---

# Boards Product Spec

## Responsibility

Boards are the human command surface for planning, approvals, work paths, packet status, evidence, watch signals, costs, and release readiness.

## Required Views

| View ID | Name | Required data | Primary actions |
| --- | --- | --- | --- |
| `VIEW-BOARDS-INTAKE` | Intake Board | planning sessions, open questions, catalog suggestions | approve intake, request map |
| `VIEW-BOARDS-WORKPATH` | Work Path Board | nodes, edges, risks, file links, tests, traces | approve path, request packet |
| `VIEW-BOARDS-PACKET` | Packet Board | packets, assignees, grants, evidence, blockers | approve packet, reject packet |
| `VIEW-BOARDS-EVIDENCE` | Evidence Board | tests, diffs, traces, watch events, audit events | accept evidence, request fix |
| `VIEW-BOARDS-RELEASE` | Release Board | bundle status, approvals, rollback anchors | approve release, preview rollback |
| `VIEW-BOARDS-COST` | Cost Board | token, build, storage, runtime, rework costs | flag variance, request estimate |

## Board States

`intake`, `mapping`, `review`, `approved`, `execution`, `verification`, `release_ready`, `released`, `blocked`, `archived`.

## Interaction Contracts

- Board approval creates `command.board.approve`.
- Packet request creates `command.packet.request`.
- Evidence acceptance creates `command.evidence.accept`.
- Release approval creates `command.release.approve`.
- Rollback preview request creates `command.rollback.preview`.

Every interaction writes an audit event and links to a visual node.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-BOARDS-001` |
| Visual node IDs | `VN-MODULE-BOARDS` |
| Visual edge IDs | `VE-BOARDS-TO-GATEWAY`, `VE-BOARDS-TO-WORKPATHS` |
