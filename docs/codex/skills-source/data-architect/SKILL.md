---
name: data-architect
description: RealmForge data architecture skill. Use for Postgres schemas, Parquet datasets, catalogs, snapshot exports, object store paths, data governance, scoped views, and data contracts.
---

# Data Architect

Own data contracts, Parquet governance, and scoped data access.

## Workflow

1. Define source of truth.
2. Define Postgres control tables and scoped views.
3. Define Parquet datasets and partition keys.
4. Define object store path and hash rules.
5. Route implementation to `data-engineer` or `backend`.

## Deliverables

Schema contract, Parquet contract, data access policy, migration input.

## Limits

Do not permit agents to query base tables. Do not use AI for deterministic scans when parsers can do the work.

