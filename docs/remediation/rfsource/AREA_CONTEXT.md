# RFSource — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of the RFSource storage layer to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION  
**Created:** 2024-01-XX  
**Author:** Architecture Team  
**Reviewers:** Yusuf Osman (Domain Architect), Dmitri Volkov (Backend Engineer), Rena Okafor (CTO)

**Important:** This document captures WHAT RFSource is supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

RFSource is RealmForge's **single-file source ledger** - a custom `.rfsource` binary format implementing Git-like version control for source code management with governance integration. It exists to provide durable, tamper-evident, policy-aware version control where the `.rfsource` file is the canonical source of truth and filesystem files are materialized projections.

RFSource solves four critical problems:
1. **Version Control**: Git-like branching, commits, and time travel for source code
2. **Governance Integration**: Policy enforcement, required tests, and compliance checks at the storage layer
3. **Single File Simplicity**: All project history in one append-only `.rfsource` file
4. **Content Addressing**: SHA-256 hashing for immutability and deduplication

RFSource is NOT a general-purpose object store - it is a specialized source code management system optimized for RealmForge's governance-first development workflow.

---

## 2. Responsibilities

RFSource owns:

* **Single-File Storage**: All project state stored in one `.rfsource` file with custom binary format
* **Git-Like Versioning**: Branches, commits, proposals (pull requests), code reviews
* **Content Addressing**: SHA-256-based content IDs for immutability and collision resistance
* **Commit Chain**: Append-only, immutable commit history with parent pointers
* **Source Artifacts**: Files, versions, chunks, and symbols with governance metadata
* **Time Travel**: Forward rollback (non-destructive) to any commit in history
* **Materialization**: Project filesystem files as read-only projections from `.rfsource`
* **Search Indexing**: Full-text and symbol indexes for code search
* **Governance Enforcement**: Policy bindings, allowed grants, required tests at artifact level
* **ACID Semantics**: Transactional writes with durability guarantees
* **Compression**: Efficient storage with Parquet-equivalent compression algorithms

---

## 3. Boundaries (What This Area Does NOT Own)

RFSource explicitly does NOT:

* **Policy Evaluation**: Does not evaluate grants or policies - stores policy bindings only
* **Authorization**: Does not enforce who can read/write - relies on control-plane authorization
* **Build/Test Execution**: Does not run tests or builds - stores test requirements only
* **IDE Integration**: Does not provide editor features - accessed through service APIs
* **Distributed Collaboration**: Single-file design, not a distributed VCS like Git
* **Large Binary Storage**: Optimized for source code, not large binary assets
* **Hot Path Operational Data**: Not for runtime application state (use PostgreSQL for that)
* **User Interface**: Has no direct UI - accessed through service layer APIs

---

## 4. Programming Language(s)

* **Primary:** Rust
  * Chosen for: Memory safety, zero-cost abstractions, performance, strong type system
  * Critical for storage layer where memory bugs can cause data corruption
  * All 9 RFSource crates written in Rust

**Why Rust?**  
Storage layers demand correctness above all else. Rust's ownership system prevents entire classes of bugs (use-after-free, data races, buffer overflows) that have historically plagued version control systems. The performance is essential, but the safety is the requirement.

---

## 5. High-Level Architecture

* **Layer:** Infrastructure / Persistence Layer
* **Position:** Foundational storage for source code management - separate from control-plane/audit/snapshot layers

**9-Crate Architecture:**

1. **rfsource-core** (Pure Types)
   * Domain model: Manifest, Commit, Branch, Artifact, Version, Symbol
   * Content-addressed typed IDs (SHA-256)
   * Zero I/O dependencies - pure data structures
   * Serde serialization for all entity types

2. **rfsource-format** (Binary Format)
   * Custom binary format parser/writer
   * Frame-based storage (manifest, commit, artifact, index frames)
   * Compression (Snappy, Zstd equivalent)
   * Schema versioning

3. **rfsource-index** (Search)
   * Full-text index (inverted index for code search)
   * Symbol index (functions, structs, types)
   * Bloom filters for fast negative lookups
   * Index rebuilding from commit history

4. **rfsource-store** (Persistence)
   * ACID transaction layer
   * Write-ahead log (WAL) for durability
   * Crash recovery
   * File locking and concurrent access control

5. **rfsource-governance** (Policy Integration)
   * Policy binding validation
   * Required test enforcement
   * Compliance check findings storage
   * Standards violation tracking

6. **rfsource-catalog** (Resource Metadata)
   * Artifact catalog (source files in project)
   * Dependency graph tracking
   * Lineage tracking (symbol references)
   * Namespace management

7. **rfsource-query** (Read Layer)
   * Query API for reading commits, branches, artifacts
   * Time-travel queries (state at any commit)
   * Diff computation (changes between commits)
   * Search query execution

8. **rfsource-materialize** (Filesystem Projection)
   * Materialize `.rfsource` → filesystem files
   * Watch for changes and rematerialize
   * Conflict detection (filesystem edits)
   * Bidirectional sync

9. **rfsource-service** (Service Layer)
   * Public API for all RFSource operations
   * Transaction coordination
   * Event emission (for audit integration)
   * Error handling and logging

**Data Flow:**

* **Write path**: Client → Service → Store (WAL + txn) → Format (serialize) → `.rfsource` file → Index update
* **Read path**: Client → Service → Query → Format (deserialize) → Store → `.rfsource` file
* **Materialize path**: `.rfsource` file → Materialize → Filesystem files (read-only projections)

---

## 6. Key Concepts

* **`.rfsource` File**: Single binary file containing all project history (manifest, commits, artifacts, indexes)
* **Content Address**: SHA-256 hash of entity content - serves as immutable identifier (e.g., `art_1a2b3c4d5e6f7890`)
* **Commit**: Immutable snapshot of project state with parent pointer (git-like)
* **Branch**: Named pointer to a commit (e.g., `br_main`)
* **Source Artifact**: File metadata with governance annotations (policy bindings, required tests)
* **Artifact Version**: Specific version of a file with content chunks
* **Source Chunk**: Piece of source code text within a version
* **Symbol**: Extracted identifier (function, struct, variable) for search and lineage
* **Proposal**: Pull request / merge request (code review workflow)
* **Time Warp**: Forward rollback to previous commit (creates new commit, preserves history)
* **Materialized Projection**: Filesystem files derived from `.rfsource` (not source of truth)
* **Manifest Invariants**: 7 design principles encoded in the manifest (e.g., "filesystem files are materialized projections")

---

## 7. Success Criteria

**Correctness:**
* Every acknowledged commit is durable (survives process crash, power loss)
* No silent data corruption - content addressing prevents tampering
* Content addresses are collision-resistant (SHA-256 strength)
* ACID semantics for all write operations

**Performance:**
* Commit latency p99 < 100ms (includes WAL fsync)
* Read latency p99 < 10ms (query commit or artifact)
* Materialization time < 500ms for typical project (100 files)
* Index query p95 < 50ms (full-text or symbol search)

**Reliability:**
* No data loss under crash scenarios (WAL guarantees recovery)
* Corrupt `.rfsource` file detected immediately on open
* Automatic index rebuild from commit history
* Graceful handling of filesystem conflicts (materialization vs. manual edits)

**Usability:**
* Git-like workflow familiarity (branches, commits, merge)
* Fast time-travel queries (any commit accessible instantly)
* Clear error messages for governance violations
* Efficient storage (compression reduces file size by 60-80%)

---

## 8. Dependencies

RFSource depends on:

* **Rust Standard Library**: File I/O, threading, error handling
  * Must be stable and cross-platform (Linux, macOS, Windows)

* **Serde + Serde JSON**: Serialization of domain model
  * Must support all domain entity types without data loss

* **SHA-2 (sha2 crate)**: SHA-256 hashing for content addressing
  * Must be cryptographically secure and fast

* **Chrono**: DateTime handling for commit timestamps
  * Must support UTC and ISO 8601 formatting

* **Thiserror**: Error handling with typed variants
  * Must compose cleanly across all 9 crates

* **Compression Library**: Snappy or Zstd equivalent
  * Must balance speed and compression ratio

---

## 9. Consumers

RFSource is consumed by:

* **Operator CLI**: Primary interface for developers to commit, branch, time-travel
* **Control Plane**: Reads policy bindings and governance metadata from artifacts
* **Catalog Service**: Imports artifact metadata for resource discovery
* **Audit Log**: Records RFSource operations (commits, rollbacks) for compliance
* **Build/Test Tools**: Read source code from materialized filesystem
* **Code Review UI**: Reads proposals, comments, diffs for review workflow
* **Search Tools**: Query full-text and symbol indexes for code search

All consumers interact through the `rfsource-service` API - none access the `.rfsource` file directly.

---

## 10. Production Readiness Considerations

**Data Integrity:**
* WAL must fsync before ACK (no data loss on crash)
* Content addressing prevents silent corruption
* Schema version validation on file open
* Atomic transaction commits (all-or-nothing)

**Failure Modes:**
* **Corrupt `.rfsource` File**: System must detect and refuse to open; recovery from backup or rebuild
* **WAL Corruption**: System must detect and refuse to start; recovery from last known good commit
* **Filesystem Conflicts**: Materialization detects manual edits; user resolves conflicts explicitly
* **Concurrent Access**: File locking prevents multiple writers; readers see consistent state

**Observability:**
* Metrics: commit count, file size, compression ratio, WAL size, index query latency
* Logs: every commit (actor, message, timestamp), every rollback, governance violations
* Traces: distributed tracing from client request through all 9 crates
* Health checks: `.rfsource` file integrity, WAL health, index freshness

**Scalability:**
* File size limits: 10GB typical, 100GB hard cap (before splitting)
* Commit history depth: 100K commits tested, 1M commits design target
* Concurrent readers: unlimited (lock-free reads)
* Concurrent writers: 1 writer at a time (file lock)
* Compression effectiveness: 60-80% reduction for source code

---

## 11. Risk Profile

**Risk 1: Data Loss (CRITICAL)**
* **Scenario**: WAL corruption, incomplete fsync, or `.rfsource` file corruption
* **Impact**: Permanent loss of source code history - catastrophic for governance audit trail
* **Mitigation**: Rigorous WAL testing, automatic backups, redundant storage, periodic integrity checks

**Risk 2: File Size Explosion (HIGH)**
* **Scenario**: Large commits, binary files, or ineffective compression cause `.rfsource` to exceed 100GB
* **Impact**: Poor performance, difficult to backup, slow materialization
* **Mitigation**: File size monitoring, compression tuning, warn on large files, file splitting strategy

**Risk 3: Concurrent Access Bugs (HIGH)**
* **Scenario**: Race condition in file locking or transaction handling corrupts `.rfsource`
* **Impact**: Data corruption, lost commits, or deadlock
* **Mitigation**: Exhaustive concurrency testing, file lock auditing, transaction isolation verification

**Risk 4: Index Staleness (MEDIUM)**
* **Scenario**: Index not updated after commit, or index rebuild fails
* **Impact**: Search queries return stale results, developers can't find code
* **Mitigation**: Index freshness monitoring, automatic rebuild triggers, index integrity checks

**Risk 5: Materialization Conflicts (MEDIUM)**
* **Scenario**: Developer manually edits filesystem files, conflicts with materialization from `.rfsource`
* **Impact**: Developer work lost, or `.rfsource` state inconsistent with filesystem
* **Mitigation**: Clear user messaging, conflict detection, rollback mechanisms, documentation

**Risk 6: Schema Evolution Breakage (MEDIUM)**
* **Scenario**: Schema version mismatch prevents opening old `.rfsource` files
* **Impact**: Legacy projects become inaccessible, migration pain
* **Mitigation**: Schema migration tooling, version compatibility testing, forward/backward compatibility guarantees

---

## 12. Open Questions (If Any)

1. **File Splitting Strategy**: When `.rfsource` exceeds 100GB, how do we split it? By time period? By artifact?
   * Decision needed by: Data Architect (Chen) + CTO (Rena)

2. **Distributed Collaboration**: Should RFSource support multi-writer scenarios (multiple developers, different machines)?
   * Decision needed by: CTO (Rena) + Product stakeholders
   * Note: Current design is single-file, single-writer

3. **Compression Algorithm**: Snappy (fast) vs. Zstd (better ratio) vs. configurable per-project?
   * Decision needed by: Backend Engineer (Dmitri) + Performance benchmarks

4. **Governance Hook Points**: At what points should governance checks run? Pre-commit? Post-commit? Background?
   * Decision needed by: Security Architect (Fatima) + Domain Architect (Yusuf)

5. **Backup Strategy**: Automatic backups to separate storage? Frequency? Retention?
   * Decision needed by: Infrastructure Architect (Nadia) + CTO (Rena)

6. **Index Persistence**: Should indexes be stored in `.rfsource` file or separate sidecar file?
   * Decision needed by: Backend Engineer (Dmitri) + Performance testing
   * Trade-off: Single file simplicity vs. rebuild time on open

---

## Appendix: Timeline and Status

* **Document Created:** 2024-01-XX
* **Last Updated:** 2024-01-XX (Corrected after review)
* **Author:** Architecture Team
* **Reviewers:**
  * [ ] Yusuf Osman (Domain Architect) - Domain model and invariants
  * [ ] Dmitri Volkov (Backend Engineer) - Rust implementation and ACID semantics
  * [ ] Rena Okafor (CTO) - Architecture alignment with layered system

**Next Step:** CTO review and sign-off before proceeding to Session 2 (question generation)

---

**Correction Note:**  
This document was rewritten on 2024-01-XX to correct a fundamental misunderstanding. The original version incorrectly described RFSource as a "Parquet-backed, content-addressable object store for RealmForge data artifacts (events, snapshots, audit logs)". The accurate description is: **RFSource is a single-file source ledger (`.rfsource` format) implementing Git-like version control for source code management with governance integration.** Parquet is NOT used in RFSource - it uses a custom binary format with Parquet-equivalent compression algorithms.

---

**END OF RFSOURCE AREA CONTEXT**
