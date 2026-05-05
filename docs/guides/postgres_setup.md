---
doc_id: DOC-GUIDE-POSTGRES
title: "PostgreSQL Setup for RealmForge Core"
status: active
owner: infra-architect
reviewers: [infra-architect, backend, data-architect]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: infrastructure
work_path_ids: [WP-DATA-001]
related_decision_ids: []
related_file_ids: [FILE-DB-001, FILE-DB-002]
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# PostgreSQL Setup for RealmForge Core

## Overview

RealmForge Core uses **PostgreSQL 18.3** as its operational database. This document describes 
the native Windows PostgreSQL installation, configuration, and how to connect to it from 
RealmForge crates.

## Installation

PostgreSQL 18.3 is installed natively on this Windows machine at:

```
C:\Program Files\PostgreSQL\18\
```

**Key binaries:**
- Server binary: `C:\Program Files\PostgreSQL\18\bin\postgres.exe`
- Client (psql): `C:\Program Files\PostgreSQL\18\bin\psql.exe`
- Control: `C:\Program Files\PostgreSQL\18\bin\pg_ctl.exe`

## Service

The PostgreSQL server runs as a Windows service:

```
Service Name:  postgresql-x64-18
Display Name:  PostgreSQL Server 18
Startup Type:  Automatic
Log On As:     NT AUTHORITY\NetworkService
```

### Starting/Stopping the Service

```powershell
# From an elevated (Admin) PowerShell:
Start-Service postgresql-x64-18
Stop-Service postgresql-x64-18

# Or using pg_ctl (does not require admin):
& 'C:\Program Files\PostgreSQL\18\bin\pg_ctl' start -D 'C:\Program Files\PostgreSQL\18\data'
& 'C:\Program Files\PostgreSQL\18\bin\pg_ctl' stop -D 'C:\Program Files\PostgreSQL\18\data'
```

## Configuration

### Port

PostgreSQL listens on **port 6969** (custom, not the default 5432).

Configured in `C:\Program Files\PostgreSQL\18\data\postgresql.conf`:
```
port = 6969
listen_addresses = '*'
```

### Data Directory

```
C:\Program Files\PostgreSQL\18\data\
```

### Log Files

```
C:\Program Files\PostgreSQL\18\data\log\
```

## Authentication

For local development, **trust** authentication is configured on the `localhost` 
loopback addresses. This means no password is required when connecting from the 
local machine via `127.0.0.1` or `::1`.

Configured in `C:\Program Files\PostgreSQL\18\data\pg_hba.conf`:
```
# TYPE  DATABASE  USER      ADDRESS        METHOD
local   all       all                      trust
host    all       all       127.0.0.1/32   trust
host    all       all       ::1/128        trust
host    all       all       localhost      trust
```

**Warning:** Trust authentication is acceptable for local development only.
For production, switch to `scram-sha-256` and manage credentials properly.

The `postgres` superuser password is set to `realmforge` (for the rare cases 
where password auth is needed).

## Databases

### realmforge_core

The primary RealmForge operational database. Contains all core tables:

| Table                    | Description                           |
|--------------------------|---------------------------------------|
| `tenants`                | Multi-tenant organization records     |
| `projects`               | Project records                       |
| `actors`                 | Users and service accounts            |
| `roles`                  | RBAC role definitions                 |
| `actor_roles`            | Role assignments to actors            |
| `sessions`               | User sessions                         |
| `bounded_commands`       | Command records (CQRS)                |
| `core_audit_events`      | Immutable audit event log (chained)   |
| `snapshot_manifests`     | Snapshot metadata                     |
| `snapshot_object_refs`   | Snapshot object references            |
| `snapshot_table_exports` | Snapshot table export tracking        |
| `rollback_previews`      | Rollback preview records              |
| `catalog_entries`        | Catalog/work-path entries             |
| `work_path_graphs`       | Work path dependency graphs           |
| `work_path_edges`        | Work path graph edges                 |
| `skill_grants`           | Agent skill permission grants         |
| `skill_registrations`    | Skill registration records            |
| `skill_sessions`         | Skill session tracking                |
| `knowledge_datasets`     | Dataset metadata                      |
| `board_plans`            | Decision board plans                  |
| `board_approvals`        | Board approval records                |
| `release_commands`       | Release/deployment commands           |
| `watch_events`           | Build/watch events                    |
| `cost_records`           | Cost tracking records                 |
| `violations`             | Policy violation records              |
| `bundle_manifests`       | Runtime bundle manifests              |
| `runtime_instances`      | Runtime instance tracking             |
| `watch_signals`          | Live watch signals                    |
| `remediation_proposals`  | Auto-remediation proposals            |
| `watch_profiles`         | Watch configuration profiles          |
| `login_attempts`         | Authentication attempt audit          |
| `login_blocks`           | Rate-limit block records              |
| `login_policies`         | Login policy configuration            |
| `stored_credentials`     | Credential hashes                     |
| `work_packets`           | Agent work packet tracking            |

### realmforge_test

Identical schema to `realmforge_core` — used for integration tests.

## Connection Strings

### .env

```env
DATABASE_URL=postgres://postgres:realmforge@127.0.0.1:6969/realmforge_core
TEST_DATABASE_URL=postgres://postgres:realmforge@127.0.0.1:6969/realmforge_test
```

### Rust (sqlx)

In `crates/control-store/src/lib.rs`, `CoreStore::connect()` accepts a URI:

```rust
let store = CoreStore::connect("postgres://postgres:realmforge@127.0.0.1:6969/realmforge_core")
    .await?;
```

For lazy pooling:
```rust
let store = CoreStore::connect_lazy("postgres://postgres:realmforge@127.0.0.1:6969/realmforge_core")?;
```

## Running Migrations

All 10 migration files are in `db/migrations/`. Apply them in order:

### Using psql

```powershell
# Apply a specific migration:
& 'C:\Program Files\PostgreSQL\18\bin\psql' -U postgres -p 6969 -d realmforge_core -f db/migrations/001_core_foundation.sql

# Apply all migrations:
Get-ChildItem db/migrations/*.sql | Sort-Object Name | ForEach-Object {
    & 'C:\Program Files\PostgreSQL\18\bin\psql' -U postgres -p 6969 -d realmforge_core -f $_.FullName
}
```

### Using sqlx CLI

```bash
# With DATABASE_URL set in environment:
DATABASE_URL="postgres://postgres:realmforge@127.0.0.1:6969/realmforge_core" sqlx migrate run
```

## Quick Reference

| Item                  | Value                                            |
|-----------------------|--------------------------------------------------|
| PostgreSQL version    | 18.3                                             |
| Install path          | `C:\Program Files\PostgreSQL\18\`                |
| Data directory        | `C:\Program Files\PostgreSQL\18\data\`           |
| Port                  | 6969                                             |
| Superuser             | postgres                                         |
| Password (dev only)   | realmforge                                       |
| Primary database      | realmforge_core                                  |
| Test database         | realmforge_test                                  |
| Auth (local dev)      | trust (no password for 127.0.0.1 connections)    |
| Auth (remote/prod)    | scram-sha-256 (not yet configured)               |
| Migration files       | `db/migrations/001-009_*.sql`                    |

## Troubleshooting

### Service won't start

Check the log file:
```powershell
Get-ChildItem 'C:\Program Files\PostgreSQL\18\data\log\*.log' | Sort-Object LastWriteTime -Descending | Select-Object -First 1 | Get-Content
```

### Common issues

1. **`pg_hba.conf` corruption** — If the file was accidentally overwritten with a single line 
   (the file path itself), replace it with proper HBA rules. See the configuration section above.

2. **Partial index with `NOW()`** — PostgreSQL 18 does not allow non-IMMUTABLE functions 
   in partial index predicates. If migration 009 fails on `idx_login_blocks_active`, 
   remove the `WHERE blocked_until > NOW()` clause from the index definition.

3. **Port 6969 already in use** — Check for another PostgreSQL instance or change the port.

4. **Connection refused** — Ensure the service is running:
   ```powershell
   Get-Service postgresql-x64-18
   ```
