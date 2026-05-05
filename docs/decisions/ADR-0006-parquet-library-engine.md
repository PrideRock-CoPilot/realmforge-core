# ADR-0006: Apache Arrow + DataFusion as Parquet Library and Query Engine

**Date:** 2026-05-05
**Status:** Accepted
**Deciders:** Council (DRI: Rena Okafor, CTO)
**Consulted:** Chen Wei (Data Architect), Priya Nair (Data Engineer), Dmitri Volkov (Backend)

## Context

RealmForge requires a Parquet library and query engine to support:

- **Knowledge phase** (Phase 5): Parquet-backed datasets for agent knowledge
- **Snapshots** (Phase 2f): Parquet-based snapshot ledger for runtime state
- **Runtime datasets** (Phase 7): Live runtime data stored in Parquet format
- **Audit logs**: Long-term audit storage in Parquet

The library must be Rust-native to match the existing crate ecosystem, avoid C/FFI bindings, and integrate cleanly with the `parquet` crate already in the dependency tree.

## Decision Drivers

- Must be pure Rust — no C/FFI bindings (security and build simplicity)
- Must support SQL query capability over Parquet files
- Must be from a well-maintained, governed open-source foundation
- Must align with existing `parquet` crate ecosystem (Apache Arrow)
- Must compile cleanly into the RealmForge workspace without external native dependencies

## Options Considered

### Option A: Apache Arrow + DataFusion (Selected)

`datafusion` crate from `apache/arrow-rs` project. Pure Rust implementation. Provides full SQL query engine over Parquet data. Same Apache Arrow project as the `parquet` crate already in use.

- Pro: Pure Rust — no C/FFI dependencies
- Pro: Apache Foundation governance — same ecosystem as existing `parquet` crate
- Pro: Full SQL query capability over Parquet files for runtime queries
- Pro: Well-maintained, active community
- Pro: `datafusion` can query Parquet files directly without loading into memory
- Con: New crate dependency (`parquet-store`) needs integration wiring

### Option B: DuckDB Rust Bindings

C-backed bindings via `duckdb` or `libduckdb-sys` crate.

- Pro: Mature SQL engine with excellent Parquet support
- Con: C/FFI bindings violate the pure-Rust security boundary
- Con: Native library dependency complicates cross-compilation and CI
- Con: Does not align with existing Arrow/Parquet crate ecosystem

### Option C: Polars

Rust-native DataFrame library with Parquet read/write support.

- Pro: Rust-native, good API
- Con: Primarily a DataFrame library, less mature as a query engine
- Con: Heavier dependency surface than `datafusion`
- Con: Duplicates existing `parquet` crate capability

## Decision

Apache Arrow + DataFusion (`datafusion` crate from `apache/arrow-rs`) is the Parquet library and query engine.

## Rationale

DataFusion is the natural extension of the existing `parquet` crate dependency already in the workspace. It is pure Rust, Apache Foundation governed, and provides full SQL query capability over Parquet data. Adding DataFusion does not introduce a new ecosystem or native dependency — it extends the Arrow/Parquet stack already present. The `parquet-store` crate provides a clean abstraction boundary between query logic and the underlying DataFusion engine.

## Consequences

**Positive:**
- Single ecosystem (Apache Arrow) for all Parquet and columnar data
- Pure Rust throughout — no C/FFI security surface
- SQL query capability over Parquet files for runtime analysis
- Active community and regular releases under Apache governance

**Negative / Trade-offs:**
- New `parquet-store` crate must be created and integrated into the workspace
- DataFusion API may evolve; pin to a stable version in `Cargo.toml`

**Risks:**
- DataFusion version churn: mitigate by explicit version pinning and CI update reviews
- Query performance tuning may require DataFusion configuration expertise (Chen Wei and Priya Nair to own)

## Implementation Plan

1. Create `crates/parquet-store/` crate with DataFusion dependency
2. Define store traits for Parquet read/write/query operations
3. Integrate with existing `parquet` crate for file-level operations
4. Wire into Knowledge (Phase 5) and Snapshot Ledger (Phase 2f) as consumers

## Review Date

2027-05-05, or earlier if DataFusion version compatibility issues arise during Phase 5 implementation.
