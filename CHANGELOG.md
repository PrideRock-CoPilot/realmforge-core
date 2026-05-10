# Changelog

All notable changes to RealmForge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased] - Storage Architecture Refactor

### Major Changes

This release represents a significant architectural evolution, replacing the POC storage layer with a production-ready governance-first system.

### Added

#### RFSource Storage Suite (8 new crates)
* **`rfsource-core`** - Pure domain types and typed IDs for storage model
* **`rfsource-format`** - Binary frame format with compression (`.rfsource` files)
* **`rfsource-index`** - Bloom filters, zone maps, and statistics
* **`rfsource-store`** - ACID storage engine with git-like commits
* **`rfsource-governance`** - Policy enforcement at storage layer
* **`rfsource-catalog`** - Schema registry and metadata management
* **`rfsource-query`** - SQL query planning and execution
* **`rfsource-materialize`** - Materialized view computation
* **`rfsource-service`** - HTTP/gRPC API layer

**Key Features**:
* Git-like version control (commit, rollback, history)
* Parquet-equivalent compression ratios
* Built-in governance enforcement
* Immutable commit semantics
* Schema evolution support

**See**: `docs/architecture/rfsource-overview.md` for full architecture documentation.

#### Governance Crates (Self-Hosting Features)
* **`code-review`** - Automated code review engine with configurable standards
* **`workflow-engine`** - Stage orchestration for development lifecycle
* **`design-council`** - Multi-stakeholder decision process with state management

These crates enable RealmForge to govern its own construction process ("eating our own dog food").

#### Workspace Dependencies
* **`async-trait`** (0.1) - Better async trait support across crates
* **`serde_yaml`** (0.9) - YAML config parsing for code review standards
* **`flate2`** (1.0) - Compression for rfsource binary format

### Removed

#### Legacy Storage
* **`parquet-store`** crate - Replaced by `rfsource-*` suite
  * **Breaking**: `.parquet` files NOT compatible with `.rfsource` format
  * **Migration**: See `docs/architecture/rfsource-overview.md#migration-from-parquet-store`

#### Legacy AI Provider
* **`ai-provider`** crate - Functionality moved to intake-engine
  * **Reason**: AI interactions are part of intake domain, not standalone
  * **Migration**: Use `intake-engine` APIs directly

### Changed

#### Workspace Structure
* **Crate count**: 16 → 26 (+62.5% growth)
* **Workspace members**: Updated in `Cargo.toml` to include all rfsource crates
* **Layering**: Strict bottom-up dependencies enforced

#### Documentation
* Moved historical progress docs to `docs/archive/`
* Added `docs/architecture/` for technical architecture docs
* Simplified root `README.md` (detailed docs now in subdirectories)

---

## Breaking Changes Summary

### For Application Developers

#### Storage API Changes

**Before** (parquet-store):
```rust
use parquet_store::write;

let data = vec![...];
write("data.parquet", data)?;
```

**After** (rfsource):
```rust
use rfsource_service::client::RFSourceClient;

let client = RFSourceClient::connect("http://localhost:8080")?;
let commit_id = client.write("data-source", data).await?;
```

**Migration Steps**:
1. Export existing `.parquet` files to JSON/CSV
2. Import into rfsource via `rfsource-service` API
3. Update application code to use `RFSourceClient`
4. Remove `parquet-store` dependency from `Cargo.toml`

#### Dependency Changes

**Update `Cargo.toml`**:
```toml
# Remove these lines:
# parquet-store = { path = "crates/parquet-store" }
# ai-provider = { path = "crates/ai-provider" }

# Add these:
rfsource-core = { path = "crates/rfsource-core" }
rfsource-service = { path = "crates/rfsource-service" }
intake-engine = { path = "crates/intake-engine" }
```

### For Contributors

#### Crate Naming
* All storage crates now prefixed with `rfsource-*`
* Governance crates separate: `code-review`, `workflow-engine`, `design-council`

#### Layering Rules
* Lower crates cannot depend on higher crates
* See `docs/architecture/rfsource-overview.md#layering-principles`

#### Testing
* New integration tests require `rfsource-service` running
* Unit tests remain isolated per-crate

---

## Migration Guide

### Step 1: Backup Existing Data

```bash
# Export all parquet files to JSON
for file in data/*.parquet; do
  cargo run --bin export-parquet -- "$file" > "${file%.parquet}.json"
done
```

### Step 2: Start RFSource Service

```bash
# Build and run service
cargo build --release -p rfsource-service
./target/release/rfsource-service --port 8080
```

### Step 3: Import Data

```bash
# Import JSON files to rfsource
for file in data/*.json; do
  curl -X POST http://localhost:8080/sources/$(basename "$file" .json)/write \
    -H "Content-Type: application/json" \
    -d @"$file"
done
```

### Step 4: Update Application Code

See "Breaking Changes Summary" above for code examples.

### Step 5: Verify Migration

```bash
# Run integration tests
cargo test --test integration_test

# Verify data integrity
cargo run --bin verify-migration
```

### Step 6: Clean Up

```bash
# Remove old parquet files (after verification!)
# rm data/*.parquet
```

---

## Architectural Decisions

### Why Custom Storage Format?

**Decision**: Build custom `.rfsource` format instead of using Apache Parquet or Delta Lake.

**Rationale**:
* **Governance-first**: Policies enforced at storage layer, not bolted on
* **Version control**: Git-like semantics core to AI-native construction
* **Fast POC**: Validated in <2 hours vs. weeks for Delta Lake integration
* **Full control**: Enables innovation (AI-powered optimization, custom indexes)

**Tradeoffs**:
* Smaller ecosystem than Parquet
* Need to maintain format ourselves
* Migration burden for existing data

**Status**: Production-ready, actively developed

**See**: `docs/archive/README.md#storage-format-decision-resolved`

### Why 8 Separate Crates?

**Decision**: Split storage into 8 layered crates instead of monolith.

**Rationale**:
* **Clear boundaries**: Each crate has single responsibility
* **Testability**: Test layers in isolation
* **Reusability**: CLI tools can use `rfsource-core` without HTTP server
* **Team scaling**: Different engineers own different layers

**Tradeoffs**:
* More `Cargo.toml` complexity
* Requires disciplined layering

**Status**: Accepted, enforced via code review

---

## Deprecation Warnings

### None Currently

All deprecated features have been removed in this release.

---

## Upgrade Path for Major Versions

### From 0.1.x → 0.2.x (Current)

**Required Actions**:
1. Migrate from `parquet-store` to `rfsource-*` (see Migration Guide above)
2. Remove `ai-provider` dependency, use `intake-engine` instead
3. Update `Cargo.toml` workspace members list

**Estimated Time**: 2-4 hours for typical project

**Support**: Old `parquet-store` crate available in git history for reference

---

## Roadmap Preview

### Next Release (0.3.0) - Planned Q2 2026
* [ ] Distributed query execution (multi-node rfsource-query)
* [ ] Change data capture (CDC) streams
* [ ] Parquet import/export for ecosystem interop

### Future (0.4.0+)
* [ ] Time-travel queries (AS OF TIMESTAMP)
* [ ] AI-powered query optimization
* [ ] Cross-source federated queries

---

## Contributors

This release was made possible by the RealmForge core team and community contributors.

**Special Thanks**:
* Storage architecture design and POC validation
* Migration tooling and testing
* Documentation improvements

---

## Resources

* **Architecture**: `docs/architecture/rfsource-overview.md`
* **Migration**: See "Migration Guide" section above
* **Historical Context**: `docs/archive/README.md`
* **Specifications**: `docs/spec/00_INDEX.md`
* **Issues**: [GitHub Issues](https://github.com/realmforge/realmforge/issues)

---

**Last Updated**: 2026-05-07  
**Maintainer**: RealmForge Core Team
