# Snapshot Ledger — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of the Snapshot Ledger to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION  
**Created:** 2025-01-XX  
**Author:** Architecture Team  
**Reviewers:** Dmitri Volkov (Backend Engineer), Chen Wei (Data Architect), Rena Okafor (CTO)

**Important:** This document captures WHAT the Snapshot Ledger is supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

The Snapshot Ledger is RealmForge's time-machine - a point-in-time state capture system that enables rollback, comparison, and audit of system state over time. It exists to answer "what did the system look like at time T?" and "how did we get from state A to state B?" for disaster recovery, debugging, and compliance verification.

The Snapshot Ledger solves three critical problems:
1. **State Preservation**: Captures complete system state (database schema, data, metadata) at specific points in time
2. **Rollback Capability**: Enables restoration to known-good states after failed deployments or malicious changes
3. **Change Auditing**: Provides forensic trail of state evolution for compliance and debugging

The Snapshot Ledger is NOT a backup system - it is a specialized governance layer for tracking and verifying state changes with cryptographic integrity.

---

## 2. Responsibilities

The Snapshot Ledger owns:

* **Snapshot Creation**: Capture point-in-time state of PostgreSQL database, metadata, and object store references
* **Manifest Generation**: Create snapshot manifests with metadata (timestamp, creator, hash chain link)
* **Hash Chain Integrity**: Link snapshots via cryptographic hashes to detect tampering
* **Object Store Integration**: Reference content-addressed objects in RFSource without duplication
* **Snapshot Validation**: Verify snapshot integrity (hash verification, object existence)
* **Snapshot Comparison**: Compute delta between two snapshots (what changed?)
* **Rollback Preview**: Show impact of rollback without executing it
* **Rollback Execution**: Restore database and object store to snapshot state
* **Snapshot Pruning**: Delete old snapshots according to retention policy
* **Snapshot Query**: List snapshots by project, time range, creator

---

## 3. Boundaries (What This Area Does NOT Own)

The Snapshot Ledger explicitly does NOT:

* **Continuous Backup**: Does not replace continuous database backups - snapshots are governance checkpoints, not disaster recovery
* **Real-Time Replication**: Does not provide real-time state replication - snapshots are discrete, infrequent captures
* **Data Storage**: Does not store data directly - references objects in RFSource and database
* **Authorization**: Does not enforce who can create/restore snapshots - relies on policy engine
* **Application State**: Does not capture in-memory application state - only persistent data
* **File System Snapshots**: Does not capture OS-level file system state - only RealmForge data
* **User Interface**: Has no direct UI - accessed through API/CLI

---

## 4. Programming Language(s)

* **Primary:** Rust
  * Chosen for: Memory safety in state capture, type safety for rollback logic, performance for large-scale comparisons
  * Critical for snapshot layer where bugs can cause permanent data loss

* **Secondary:** SQL (PostgreSQL)
  * Used for: Snapshot manifest storage, metadata queries, database schema capture

**Why Rust?**  
Snapshots are the last line of defense against catastrophic failures - a bug in rollback logic can destroy data permanently. Rust's ownership system prevents memory corruption during state capture. The type system ensures rollback operations are correct by construction. The performance enables fast comparison of large state deltas.

---

## 5. High-Level Architecture

* **Layer:** Infrastructure / Governance Layer
* **Position:** Consumes state from PostgreSQL and RFSource, exposes snapshot/rollback services

**Key Components:**

1. **Snapshot Orchestrator**
   * Triggers snapshot creation (manual, scheduled, command-anchored)
   * Coordinates database export + object store reference collection
   * Generates SnapshotManifest with hash chain
   * Stores manifest in PostgreSQL (snapshot_manifests table)
   * Returns snapshot ID and hash

2. **Database Exporter**
   * Exports PostgreSQL schema (tables, indexes, constraints)
   * Exports table data (pg_dump or custom export)
   * Computes data hash for integrity verification
   * Stores export metadata (table_name, row_count, data_hash)

3. **Object Store Collector**
   * Enumerates objects in RFSource referenced by this snapshot
   * Creates SnapshotObjectRef records (object_hash, storage_path, size)
   * Does NOT copy objects - only records references (content-addressed)

4. **Snapshot Comparator**
   * Accepts two snapshot IDs (from_snapshot, to_snapshot)
   * Computes schema delta (tables added/dropped, columns added/dropped)
   * Computes data delta (rows added/modified/deleted per table)
   * Returns SnapshotDiff with change summary

5. **Rollback Engine**
   * **Preview Mode**: Compute rollback impact (what will change?)
   * **Execute Mode**: Restore database schema and data to snapshot state
   * **Verify Mode**: Confirm restored state matches snapshot manifest
   * Validation: check object store integrity before rollback

6. **Manifest Validator**
   * Verifies snapshot hash chain (each manifest links to previous)
   * Validates object references (all objects exist in RFSource)
   * Checks data hash integrity (recompute and compare)
   * Returns validation result with gaps/issues

**Data Flow:**
* Create: Snapshot Service → Database Exporter → Object Store Collector → Manifest Generator → PostgreSQL
* Compare: Snapshot Service → Comparator → Delta Computation → Return Diff
* Rollback: Rollback Service → Preview/Validate → Execute Restore → Verify → Return Status

---

## 6. Key Concepts

* **Snapshot**: Point-in-time capture of system state (database + object store references)
* **SnapshotManifest**: Metadata record with (snapshot_id, timestamp, creator, hash, previous_hash, object_refs)
* **SnapshotObjectRef**: Reference to content-addressed object in RFSource (hash, path, size)
* **Hash Chain**: Cryptographic link between snapshots (each manifest hash depends on previous)
* **Snapshot Delta**: Difference between two snapshots (schema changes, data changes)
* **Rollback Preview**: Dry-run showing what would change during rollback
* **Rollback Execution**: Actual restoration of database and object store to snapshot state
* **Snapshot Pruning**: Deletion of old snapshots according to retention policy
* **Content Addressing**: Objects referenced by hash, not copied - enables deduplication

---

## 7. Success Criteria

**Correctness:**
* Every snapshot is complete (all tables, all referenced objects)
* Snapshot hash chain is unbroken (no gaps, tampering detected)
* Rollback restores exact state (bit-for-bit match with snapshot)
* Snapshot comparison is accurate (no missed changes)

**Performance:**
* Snapshot creation latency < 5 minutes for 1GB database
* Snapshot comparison latency < 30 seconds for two adjacent snapshots
* Rollback preview latency < 1 minute
* Rollback execution latency < 10 minutes for 1GB database

**Reliability:**
* Snapshot creation never leaves database in inconsistent state
* Rollback validates before execution (no partial rollbacks)
* Snapshot verification detects all integrity issues
* Graceful failure if object store is unavailable

**Security:**
* Snapshots are immutable once created (no modification)
* Snapshot manifests are hash-chained to detect tampering
* Rollback requires explicit authorization (policy check)
* Snapshot data does not leak sensitive information (sanitized metadata)

---

## 8. Dependencies

The Snapshot Ledger depends on:

* **PostgreSQL**: For snapshot manifest storage and database state capture
  * Must support: database export (pg_dump or equivalent), schema introspection, transaction isolation

* **RFSource**: For content-addressed object storage
  * Snapshot Ledger references objects by hash, does not copy them

* **Authority Domain**: Provides ActorID, SessionID, ProjectID types for snapshot metadata

* **Cryptographic Hash Library**: SHA-256 or BLAKE3 for manifest hash chain

* **Chrono**: For timestamp handling and snapshot ordering

---

## 9. Consumers

The Snapshot Ledger is consumed by:

* **Control Plane**: Creates snapshots after significant commands (deploy, major config change)
* **Rollback Service**: Uses snapshots to restore system state after failures
* **Compliance Systems**: Exports snapshots for regulatory audits
* **Debugging Tools**: Compares snapshots to diagnose state drift
* **API/CLI**: Operators create, list, compare, and restore snapshots through REST API or CLI
* **Monitoring Systems**: Alerts on snapshot creation failures or hash chain breaks

All consumers interact through the snapshot service layer - none access PostgreSQL or RFSource directly.

---

## 10. Production Readiness Considerations

**Data Integrity:**
* Snapshot manifests must be complete (no missing object references)
* Hash chain must be continuous (no gaps, no broken links)
* Rollback must be atomic (all-or-nothing, no partial state)

**Failure Modes:**
* **Snapshot Creation Fails**: Transaction rollback, no partial snapshot, clear error
* **Object Store Unavailable**: Snapshot creation blocks or fails gracefully
* **Rollback Fails Mid-Execution**: Database restore from pre-rollback backup, rollback retryable

**Observability:**
* Metrics: snapshot creation time, snapshot size, rollback latency, hash chain verification time, storage used
* Logs: every snapshot creation (snapshot_id, timestamp, creator, size), every rollback (from, to, result), all hash chain breaks
* Traces: distributed tracing from snapshot request through database export and object collection

**Scalability:**
* Horizontal scaling: snapshot creation can be parallelized (one per project)
* Vertical scaling: larger database instance for faster pg_dump, more disk for snapshot storage
* Pruning: delete old snapshots to manage growth (retention policy)

---

## 11. Risk Profile

**Risk 1: Snapshot Data Loss (CRITICAL)**
* **Scenario**: Snapshot manifest or referenced objects lost
* **Impact**: Cannot rollback, permanent loss of state history, compliance failure
* **Mitigation**: PostgreSQL replication, RFSource redundancy, periodic integrity checks

**Risk 2: Incomplete Snapshot (CRITICAL)**
* **Scenario**: Snapshot creation succeeds but misses tables or objects
* **Impact**: Rollback restores incomplete state, data loss, operational disruption
* **Mitigation**: Validation checks, completeness verification, integration tests

**Risk 3: Rollback Corruption (CRITICAL)**
* **Scenario**: Rollback execution corrupts database or object store
* **Impact**: System unusable, data loss, extended outage
* **Mitigation**: Rollback preview + validation, pre-rollback backup, rollback testing

**Risk 4: Hash Chain Tampering (HIGH)**
* **Scenario**: Attacker modifies snapshot manifests without detection
* **Impact**: Compromised state history, rollback to malicious state
* **Mitigation**: Cryptographically strong hash (SHA-256/BLAKE3), periodic verification, immutable storage

**Risk 5: Runaway Storage Growth (MEDIUM)**
* **Scenario**: Snapshots accumulate, exhaust disk space
* **Impact**: Snapshot creation fails, database unavailable
* **Mitigation**: Retention enforcement, pruning automation, storage monitoring

---

## 12. Open Questions (If Any)

1. **Snapshot Frequency**: How often should automatic snapshots be created (daily, per-command, on-demand)?
   * Decision needed by: CTO (Rena) + Infrastructure Architect (Nadia)

2. **Retention Period**: How long should snapshots be kept (30 days, 1 year, indefinite)?
   * Decision needed by: Data Architect (Chen) + Security Architect (Fatima)

3. **Rollback Scope**: Should rollback restore entire database or allow per-table rollback?
   * Decision needed by: CTO (Rena) + Domain Architect (Yusuf)

4. **Snapshot Compression**: Should database exports be compressed (reduces storage, increases CPU)?
   * Decision needed by: Infrastructure Architect (Nadia) + Performance benchmarks

5. **External Snapshot Storage**: Should snapshots be stored externally (S3, Azure Blob) or in PostgreSQL?
   * Decision needed by: Infrastructure Architect (Nadia) + Data Architect (Chen)

---

## Appendix: Timeline and Status

* **Document Created:** 2025-01-XX
* **Last Updated:** 2025-01-XX
* **Author:** Architecture Team
* **Reviewers:**
  * [ ] Dmitri Volkov (Backend Engineer) - Snapshot and rollback implementation
  * [ ] Chen Wei (Data Architect) - Database export and state capture
  * [ ] Rena Okafor (CTO) - Architecture alignment with layered system

**Next Step:** CTO review and sign-off before proceeding to Session 2 (question generation)

---

**END OF SNAPSHOT LEDGER AREA CONTEXT**
