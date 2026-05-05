---
doc_id: DOC-SPEC-017
title: API MCP CLI Contracts
status: draft
owner: api-architect
reviewers: [cto, backend, security-architect, qa, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: interfaces
work_path_ids: [WP-API-001, WP-MCP-001, WP-CLI-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-INTERFACE-CONTRACTS]
visual_edge_ids: []
approval_state: pending
---

# API MCP CLI Contracts

## REST Endpoints

All endpoints are under `/v1`.

| Method | Path | Action | Permission | Test ID |
| --- | --- | --- | --- | --- |
| `POST` | `/organizations` | `command.organization.create` | `organization.write` | `TEST-API-ORG-001` |
| `POST` | `/tenants` | `command.tenant.create` | `tenant.write` | `TEST-API-TENANT-001` |
| `POST` | `/apps` | `command.app.create` | `app.write` | `TEST-API-APP-001` |
| `POST` | `/catalogs/modules/copy` | `command.catalog.copy_module` | `catalog.module.copy` | `TEST-API-CATALOG-001` |
| `POST` | `/skill-grants` | `command.skill_grant.issue` | `skill_grant.issue` | `TEST-API-GRANT-001` |
| `POST` | `/work-paths` | `command.work_path.create` | `work_path.write` | `TEST-API-WORKPATH-001` |
| `POST` | `/work-packets/generate` | `command.packet.request` | `packet.generate` | `TEST-API-PACKET-001` |
| `POST` | `/gateway/commands/{id}/authorize` | `command.authorize` | `command.authorize` | `TEST-API-COMMAND-001` |
| `POST` | `/evidence` | `command.evidence.submit` | `evidence.submit` | `TEST-API-EVIDENCE-001` |
| `POST` | `/snapshots` | `command.snapshot.create` | `snapshot.create` | `TEST-API-SNAPSHOT-001` |
| `POST` | `/runtime-bundles/build` | `command.bundle.build` | `bundle.build` | `TEST-API-BUNDLE-001` |
| `POST` | `/live-watch/signals` | `command.watch.signal_record` | `watch.signal.write` | `TEST-API-LIVE-WATCH-001` |

Error shape:

```json
{
  "error_code": "FILE_SCOPE_DENIED",
  "message": "Action denied by gateway scope policy.",
  "request_id": "req_...",
  "audit_event_id": "evt_..."
}
```

## MCP Tools

Tool names:

- `core_get_scope`
- `core_request_work_packet`
- `core_propose_command`
- `core_submit_evidence`
- `core_query_knowledge`
- `core_preview_rollback`
- `core_record_watch_signal`
- `core_build_runtime_bundle`

Every MCP tool requires `actor_id`, `skill_grant_id`, and `app_id` unless the tool is `core_get_scope`.

## CLI Commands

| Command | Purpose |
| --- | --- |
| `control migrate apply` | Apply Postgres migrations |
| `control catalog copy-module` | Copy approved module into tenant/app catalog |
| `control grant issue` | Issue skill grant |
| `control packet generate` | Generate work packet |
| `control packet inspect` | Inspect packet scope |
| `control evidence submit` | Submit evidence record |
| `control snapshot create` | Create governed snapshot |
| `control bundle build` | Build signed runtime bundle |
| `control watch tail` | Tail Build Watch or Live Watch events |
| `control rollback preview` | Preview rollback |
| `control rollback execute` | Execute approved rollback |
