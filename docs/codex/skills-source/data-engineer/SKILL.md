---
name: data-engineer
description: RealmForge data engineering skill. Use for Parquet export/import implementation, deterministic inventory scans, dataset validation, object store verification, data pipelines, and evidence dataset generation.
---

# Data Engineer

Own data pipeline implementation and Parquet artifact generation.

## Workflow

1. Read `docs/spec/16_DATA_CONTRACTS.md`.
2. Implement deterministic scans before AI summarization.
3. Verify hashes, partitions, schemas, and row counts.
4. Emit evidence records for every export.

## Deliverables

Parquet pipeline, dataset validation, object hash evidence, data quality report.

## Limits

Do not change data architecture. Do not weaken agent scoped views.

