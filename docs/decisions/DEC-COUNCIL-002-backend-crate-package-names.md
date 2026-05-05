---
decision_id: DEC-COUNCIL-002
title: Backend Crate And Package Names
status: accepted
owner: council
participants: [cto, domain-architect, backend, tech-writer]
decided_at: 2026-05-04
supersedes: []
related_spec_ids: [DOC-SPEC-001, DOC-SPEC-002, DOC-SPEC-004, DOC-SPEC-020, DOC-SPEC-022]
---

# DEC-COUNCIL-002: Backend Crate And Package Names

## Decision Question

What exact crate and package names replace `realmforge-core` and `rf-*` after workspace flattening?

## Council Decision

RealmForge will flatten the current `realmforge-core` Rust implementation into the root workspace at `E:\realmforge` and replace `rf-*` crate/package names with capability names.

The root workspace has no product-branded Rust package. Product branding remains in human-facing docs and product surfaces.

## Accepted Name Map

| Current source | Accepted crate/package | Responsibility |
| --- | --- | --- |
| `realmforge-core` workspace | root `E:\realmforge` workspace | Workspace container only, no package name |
| `rf-domain` | `authority-domain` | Pure IDs, entities, state machines, commands |
| `rf-policy` | `policy-engine` | Authorization, policy decisions, denial codes |
| `rf-events` | `audit-log` | Append-only audit hash-chain events |
| `rf-snapshot` | `snapshot-ledger` | Snapshot manifests, object refs, rollback analysis |
| `rf-store` | `control-store` | Postgres persistence adapters |
| `rf-service` | `control-service` | Shared orchestration over domain, policy, store, audit, and snapshots |
| `rf-api` | `control-api` | Thin REST adapter |
| `rf-mcp` | `agent-mcp` | Thin MCP adapter for agent tools |
| `rf-cli` | `operator-cli` | Thin operator CLI package |
| `realmforge` CLI binary | `control` | Operator command surface |

The first new gateway crate will be named `agent-gateway`.

Future approved target crate names already listed in the file registry remain accepted: `parquet-catalog`, `runtime-bundle`, `build-watch`, and `live-watch`.

## Rationale

The product definition locks the rule that backend internals use capability names rather than product branding or `rf-*` prefixes. The accepted names preserve the existing bounded-context responsibilities while making each crate readable outside the product brand.

The names also match the planned file registry, which already defines target paths such as `crates/authority-domain`, `crates/policy-engine`, `crates/control-service`, `crates/agent-mcp`, and `crates/operator-cli`.

## Rejected Options

| Option | Reason rejected |
| --- | --- |
| Keep `rf-*` names until later | Blocks the active Rust cleanup and keeps product shorthand in backend internals. |
| Use `realmforge-*` names | Violates the locked product decision that backend internals should avoid product branding. |
| Use generic names such as `domain`, `policy`, `store` | Too broad for a multi-crate workspace and unclear in dependency graphs. |

## Consequences

- The `DEC-COUNCIL-002` lockout is closed for workspace flattening and `rf-*` crate/package replacement.
- Implementation may rename Cargo packages, crate directories, dependency aliases, imports, tests, docs, and health/service labels that depend on these exact names.
- The rename must preserve the layer law and thin API/MCP/CLI adapter boundaries.
- Other open decisions remain unresolved until they block active work.

## Follow-Up

- Backend owns the mechanical Rust rename and validation.
- CTO owns architecture review if the rename exposes layer violations.
- Tech Writer owns spec/docs alignment after the implementation rename.
- QA owns validation evidence for `cargo fmt`, `cargo clippy`, and `cargo test`.
