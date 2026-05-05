---
doc_id: DOC-PLAN-P1
title: Phase 1 — Workspace And Prerequisites
parent: DOC-PLAN-INDEX
status: draft
owner: backend
reviewers: [pm, cto, infra-architect]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: []
related_decision_ids: [DEC-COUNCIL-002]
related_file_ids: [FILE-ROOT-CARGO]
visual_node_ids: [VN-ROADMAP]
approval_state: pending
---

# Phase 1: Workspace And Prerequisites

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 1 |
| Title | Workspace And Prerequisites |
| Work paths | (prerequisite to all subsequent work paths) |
| Product module | workspace |
| Owner | backend |
| Risk | medium |
| Decision blockers | none (DEC-COUNCIL-002 is closed and accepted) |

**Mandate:** Configure Cargo availability, Git safe-directory access, Postgres access for current user, and root workspace flattening under the crate/package names accepted in `DEC-COUNCIL-002`.

---

## Decision Lockout

| Decision | Status | Impact |
|----------|--------|--------|
| `DEC-COUNCIL-002` | `ACCEPTED` (closed) | Crate/package names are locked. All crate names must use capability names (authority-domain, policy-engine, audit-log, snapshot-ledger, control-store, control-service, control-api, agent-mcp, operator-cli, agent-gateway, parquet-catalog, runtime-bundle, build-watch, live-watch). |

---

## Workflow

```
1. Verify toolchain
   a. cargo --version matches rust-toolchain.toml
   b. rustup show displays correct active toolchain

2. Configure Git
   a. git config --global safe.directory for e:/realmforge
   b. Verify origin remote points to correct repository

3. Configure Postgres
   a. Verify psql is accessible
   b. Configure connection string in environment or .env file
   c. Verify user has permission to create databases

4. Verify workspace Cargo.toml
   a. All crate members are listed: authority-domain, audit-log, policy-engine,
      snapshot-ledger, control-store, control-service, control-api, agent-mcp,
      operator-cli
   b. Dependency declarations are correct
   c. Edition matches rust-toolchain.toml
   d. Workspace root has resolver = "2"

5. Verify existing crate skeletons compile
   a. cargo check --workspace passes
   b. cargo build --workspace compiles all crate skeletons

6. Verify existing migrations
   a. 001_core_foundation.sql — tenants, projects, actors, roles, actor_roles,
      skill_registrations, sessions, skill_sessions, bounded_commands, core_audit_events
   b. 002_snapshot_foundation.sql — snapshot_manifests, snapshot_object_refs,
      snapshot_table_exports, rollback_previews

7. Run initial validation
   a. cargo fmt --all -- --check
   b. cargo clippy --workspace -- -D warnings
   c. cargo test --workspace
```

---

## File Manifest

| Action | File | Note |
|--------|------|------|
| VERIFY | `Cargo.toml` (workspace root) | Verify workspace members and dependency declarations |
| VERIFY | `rust-toolchain.toml` | Verify toolchain channel and targets |
| VERIFY | `.gitignore` | Verify ignores /target, .realmforge/, .env |
| VERIFY | `.clinerules` | Verify Cline enforcement rules are present |
| VERIFY | `crates/authority-domain/Cargo.toml` | Verify crate metadata matches DEC-COUNCIL-002 |
| VERIFY | `crates/policy-engine/Cargo.toml` | Same |
| VERIFY | `crates/audit-log/Cargo.toml` | Same |
| VERIFY | `crates/snapshot-ledger/Cargo.toml` | Same |
| VERIFY | `crates/control-store/Cargo.toml` | Same |
| VERIFY | `crates/control-api/Cargo.toml` | Same |
| VERIFY | `crates/agent-mcp/Cargo.toml` | Same |
| VERIFY | `crates/operator-cli/Cargo.toml` | Same |
| VERIFY | `db/migrations/001_core_foundation.sql` | Verify applies cleanly |
| VERIFY | `db/migrations/002_snapshot_foundation.sql` | Verify applies cleanly |

---

## Completion Gates

- [x] `cargo build --workspace` compiles all crate skeletons
- [x] `cargo test --workspace` passes with 0 failures
- [x] `cargo clippy --workspace -- -D warnings` passes
- [x] `cargo fmt --all -- --check` passes
- [x] Postgres connection string is configured and accessible via `control-store`
- [x] Git safe-directory configuration is verified
- [x] DB migration `001_core_foundation.sql` applies cleanly
- [x] DB migration `002_snapshot_foundation.sql` applies cleanly

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-DOMAIN` | Verify and configure crate Cargo.toml files |
| `SGL-DATA-POSTGRES` | Verify Postgres access and migrations |

---

## Dependencies

- Phase 0 complete (documentation certified)
