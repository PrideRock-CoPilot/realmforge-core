---
doc_id: DOC-SPEC-002
title: Final System Map
status: draft
owner: cto
reviewers: [domain-architect, security-architect, api-architect, data-architect, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: architecture
work_path_ids: [WP-DOCS-000]
related_decision_ids: [DEC-COUNCIL-002]
related_file_ids: []
visual_node_ids: [VN-SYSTEM-ECOSYSTEM]
visual_edge_ids: []
approval_state: pending
---

# Final System Map

## Workspace Shape

The target workspace root is `E:\realmforge`.

```text
E:\realmforge
  Cargo.toml
  docs/
  crates/
  apps/
  db/
  catalog/
  .realmforge/
```

The former `realmforge-core` source has been flattened into root `crates/` under the accepted names in `DEC-COUNCIL-002`.

## Target Backend Crate Families

The capability boundaries and crate/package names are locked by `DEC-COUNCIL-002`:

| Capability | Current source | Target crate/package | Target responsibility | Visual node |
| --- | --- | --- | --- | --- |
| Domain authority | `rf-domain` | `authority-domain` | Pure IDs, entities, state machines, commands | `VN-CRATE-DOMAIN` |
| Policy engine | `rf-policy` | `policy-engine` | Authorization and denial decisions | `VN-CRATE-POLICY` |
| Audit ledger | `rf-events` | `audit-log` | Append-only audit hash-chain events | `VN-CRATE-AUDIT` |
| Snapshot ledger | `rf-snapshot` | `snapshot-ledger` | Manifests, object refs, rollback analysis | `VN-CRATE-SNAPSHOT` |
| Control store | `rf-store` | `control-store` | Postgres persistence adapters | `VN-CRATE-STORE` |
| Control service | `rf-service` | `control-service` | Business orchestration over policy/store/snapshot | `VN-CRATE-SERVICE` |
| Agent gateway | new crate | `agent-gateway` | Enforced gateway for agent commands and file operations | `VN-CRATE-GATEWAY` |
| Control API | `rf-api` | `control-api` | Thin REST adapter | `VN-CRATE-API` |
| Agent MCP | `rf-mcp` | `agent-mcp` | Thin MCP adapter | `VN-CRATE-MCP` |
| Operator CLI | `rf-cli` | `operator-cli` | Thin command-line adapter; binary name `control` | `VN-CRATE-CLI` |

## Layer Law

```text
human UI / Codex / Claude / operator
  -> Boards / CLI / API / MCP
  -> Agent Gateway
  -> Control Service
  -> Policy Engine
  -> Domain Authority
  -> Store / Snapshot Ledger
  -> Postgres / Parquet / Object Store
```

Forbidden paths:

- Agent to filesystem without gateway.
- Agent to Postgres without gateway and store adapter.
- API/MCP/CLI to Postgres directly.
- Runtime handler execution before policy check.
- Live Watch direct remediation execution in first governed release.

## System Edges

| Edge ID | From | To | Contract |
| --- | --- | --- | --- |
| `VE-BOARDS-TO-GATEWAY` | Boards | Agent Gateway | Submit approvals, command requests, packet requests |
| `VE-GATEWAY-TO-AUTHORITY` | Agent Gateway | Authority Core | Authorize command, validate grant, append audit |
| `VE-AUTHORITY-TO-POSTGRES` | Authority Core | Postgres | Persist live governance state |
| `VE-AUTHORITY-TO-PARQUET` | Authority Core | Parquet | Export immutable state snapshots |
| `VE-KNOWLEDGE-TO-CATALOGS` | Knowledge | Catalogs | Query scoped module, file, decision, evidence records |
| `VE-BUILD-WATCH-TO-EVIDENCE` | Build Watch | Evidence Ledger | Record construction events |
| `VE-RUNTIME-BUNDLE-TO-LIVE-RUNTIME` | Runtime Bundle | Live Runtime | Load signed bundle |
| `VE-LIVE-WATCH-TO-GATEWAY` | Live Watch | Agent Gateway | Propose remediation packet |
