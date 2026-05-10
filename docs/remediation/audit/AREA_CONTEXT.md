# Audit Log — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of the Audit Log to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION  
**Created:** 2025-01-XX  
**Author:** Architecture Team  
**Reviewers:** Fatima Al-Hassan (Security Architect), Rena Okafor (CTO), Nadia Kovacs (Infrastructure Architect)

**Important:** This document captures WHAT the Audit Log is supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

The Audit Log is RealmForge's immutable forensic trail - a tamper-evident, cryptographically-verified record of every significant event in the system. It exists to answer "what happened, when, by whom, and why" for compliance, security investigations, and operational debugging. The audit log is the source of truth for accountability and the last line of defense against unauthorized changes going undetected.

The Audit Log solves three critical problems:
1. **Accountability**: Creates an irrefutable record of who did what, when, and with what authorization
2. **Tamper Detection**: Hash-chained events prevent silent modification or deletion of audit history
3. **Compliance**: Provides the forensic trail required for SOC 2, HIPAA, GDPR, and other regulatory frameworks

The Audit Log is NOT a general-purpose logging system - it is a specialized compliance and security audit trail with cryptographic integrity guarantees.

---

## 2. Responsibilities

The Audit Log owns:

* **Event Ingestion**: Accept audit events from all RealmForge components (commands, authorizations, snapshots, rollbacks)
* **Hash Chain Integrity**: Compute and verify cryptographic hash chains to detect tampering
* **Event Storage**: Persist events durably in PostgreSQL with immutability guarantees
* **Event Categorization**: Classify events by type (Command, Authorization, Snapshot, Rollback, SystemEvent)
* **Query Interface**: Provide filtered, paged queries by project, actor, time range, event type
* **Integrity Verification**: Validate hash chain continuity and detect gaps or tampering
* **Retention Management**: Enforce retention policies (minimum retention, archive to cold storage)
* **Export Capability**: Support audit log export for external compliance systems (SIEM, log aggregators)
* **Event Replay**: Enable chronological replay of events for forensic analysis

---

## 3. Boundaries (What This Area Does NOT Own)

The Audit Log explicitly does NOT:

* **Operational Logging**: Does not handle application logs, debug logs, or metrics - those go to observability systems
* **Real-Time Alerting**: Does not trigger alerts or notifications - monitoring systems consume audit events for that
* **Event Generation**: Does not create events - consumes events from other components
* **Authorization**: Does not enforce who can query audit logs - relies on policy engine
* **Analytics**: Does not perform analytics or reporting - external systems consume audit data for analysis
* **Hot Path Performance**: Not optimized for low-latency writes - designed for durability over speed
* **User Interface**: Has no direct UI - accessed through API/CLI

---

## 4. Programming Language(s)

* **Primary:** Rust
  * Chosen for: Memory safety in cryptographic operations, strong type system for event integrity, performance for hash computation
  * Critical for audit layer where bugs can compromise forensic integrity

* **Secondary:** SQL (PostgreSQL)
  * Used for: Event storage, querying, retention management

**Why Rust?**  
Audit logs are permanent records - a bug that corrupts or loses audit data can invalidate years of compliance work. Rust's ownership system prevents memory corruption in hash chain computation. The type system ensures event serialization is correct by construction. The performance enables high-throughput event ingestion without blocking other operations.

---

## 5. High-Level Architecture

* **Layer:** Cross-Cutting Infrastructure (all components write to audit log)
* **Position:** Receives events from all layers, stores in PostgreSQL, exposes query interface

**Key Components:**

1. **Event Ingester**
   * Accepts AuditEvent from service layer (command service, snapshot service, etc.)
   * Validates event structure (required fields, valid types)
   * Computes hash chain: hash(current_event + previous_hash)
   * Writes event to PostgreSQL (core_audit_events table)
   * Returns event ID and hash

2. **Hash Chain Manager**
   * Maintains hash chain continuity per project
   * Computes event hash: SHA-256(event_id || timestamp || actor_id || event_type || event_data || previous_hash)
   * Validates hash chain on query (verify_chain function)
   * Detects gaps, tampering, or missing events

3. **Query Engine**
   * Accepts filters: project_id, actor_id, time_range, event_type
   * Returns paged results (ordered by timestamp)
   * Supports reverse chronological queries
   * Provides event count and coverage metrics

4. **Integrity Verifier**
   * Validates hash chain for a project
   * Recomputes hashes for all events and compares to stored values
   * Detects: missing events, modified events, broken chain links
   * Returns verification result with gap details

5. **Retention Manager**
   * Enforces minimum retention period (e.g., 7 years for compliance)
   * Archives old events to cold storage (S3 Glacier, Azure Archive)
   * Prevents premature deletion
   * Supports legal hold (prevent deletion for litigation)

**Data Flow:**
* Service layer → append_event(audit_event) → Hash Chain Manager → PostgreSQL → ACK to Service
* Query: API/CLI → query_events(filters) → Query Engine → PostgreSQL → Paged Results
* Verify: Operator → verify_chain(project_id) → Integrity Verifier → Recompute Hashes → Report

---

## 6. Key Concepts

* **AuditEvent**: Immutable record with (event_id, timestamp, actor_id, event_type, event_data, previous_hash, hash)
* **Hash Chain**: Cryptographic link between events - each event hash depends on previous event
* **Event Type**: Category of event (Command, Authorization, Snapshot, Rollback, SystemEvent)
* **Project Scope**: Audit events are scoped to projects - hash chains are per-project
* **Immutability**: Events cannot be modified or deleted after writing (only archived)
* **Retention Policy**: Minimum duration events must be kept (7 years for compliance)
* **Forensic Replay**: Chronological re-execution of events for investigation
* **Tamper Detection**: Hash chain verification catches any modification or deletion

---

## 7. Success Criteria

**Correctness:**
* Every event is recorded exactly once (no duplicates, no lost events)
* Hash chain is unbroken (no gaps, tampering detected immediately)
* Event ordering is consistent (timestamp monotonicity within project)
* Event data is complete (all required fields captured)

**Performance:**
* Event ingestion latency p99 < 10ms (write to PostgreSQL)
* Query latency p99 < 100ms (paged queries up to 1000 events)
* Hash chain verification latency < 5 seconds per 10K events
* Throughput: sustain 5K events/sec per node

**Reliability:**
* No event loss under crash scenarios (durable writes to PostgreSQL)
* Graceful degradation if audit log is unavailable (queue events, never block operations)
* Automatic hash chain repair if database restore breaks continuity

**Security:**
* Events are immutable once written (no UPDATE or DELETE on core_audit_events)
* Hash tampering is immediately detected on verification
* Audit queries are authorized via policy engine
* Event data does not leak sensitive information (sanitized before logging)

---

## 8. Dependencies

The Audit Log depends on:

* **PostgreSQL**: For durable event storage in core_audit_events table
  * Must support: write-heavy workload, large table scans for verification, retention enforcement

* **Authority Domain**: Provides ActorID, SessionID, ProjectID types for event metadata

* **Cryptographic Hash Library**: SHA-256 or BLAKE3 for hash chain computation
  * Must be cryptographically secure and fast

* **Chrono**: For timestamp handling and timezone-aware event ordering

---

## 9. Consumers

The Audit Log is consumed by:

* **Compliance Systems**: External SIEM, log aggregators, compliance dashboards
* **Forensic Analysts**: Operators investigating security incidents or policy violations
* **Monitoring Systems**: Real-time alerts based on audit event patterns
* **Reporting Tools**: Dashboards showing authorization trends, command usage, rollback frequency
* **Legal/Audit Teams**: Export audit logs for regulatory audits or litigation
* **API/CLI**: Operators query audit logs through REST API or CLI commands

All consumers interact through the audit service layer - none access PostgreSQL directly.

---

## 10. Production Readiness Considerations

**Data Integrity:**
* Every event must be written durably before ACK (no data loss on crash)
* Hash chain must be continuous (no gaps, no broken links)
* Event timestamps must be monotonic within project (no time travel)

**Failure Modes:**
* **PostgreSQL Unavailable**: Queue events in-memory, persist to WAL, never block operations
* **Hash Chain Break**: Detect immediately on next event ingestion, alert operators
* **Disk Full**: Trigger retention enforcement, alert operators, fail gracefully

**Observability:**
* Metrics: event ingestion rate, query latency, hash chain verification time, storage used, retention status
* Logs: every event ingestion (event_id, type, actor, timestamp), every hash chain break, all retention actions
* Traces: distributed tracing from event generation through storage

**Scalability:**
* Horizontal scaling: partition audit events by project_id, shard PostgreSQL
* Vertical scaling: larger PostgreSQL instance, faster disks for write-heavy workload
* Archival: move old events to cold storage (S3 Glacier) to manage growth

---

## 11. Risk Profile

**Risk 1: Audit Log Data Loss (CRITICAL)**
* **Scenario**: PostgreSQL failure, disk corruption, or bug causes event loss
* **Impact**: Compliance violation, forensic gaps, regulatory penalties
* **Mitigation**: Durable writes (fsync), PostgreSQL replication, periodic integrity checks, WAL backup

**Risk 2: Hash Chain Tampering (CRITICAL)**
* **Scenario**: Attacker modifies audit events without detection
* **Impact**: Compromised forensic trail, loss of trust, compliance failure
* **Mitigation**: Cryptographically strong hash (SHA-256/BLAKE3), periodic verification, immutable storage

**Risk 3: Audit Log Unavailability Blocks Operations (HIGH)**
* **Scenario**: Audit log down causes all operations to fail (tight coupling)
* **Impact**: System-wide outage, business disruption
* **Mitigation**: Asynchronous event ingestion, in-memory queue, graceful degradation

**Risk 4: Runaway Storage Growth (MEDIUM)**
* **Scenario**: Audit log grows unbounded, exhausts disk space
* **Impact**: PostgreSQL failure, audit log unavailability, data loss
* **Mitigation**: Retention enforcement, archival to cold storage, storage monitoring, alerts

**Risk 5: Hash Chain Break After Database Restore (MEDIUM)**
* **Scenario**: PostgreSQL restore from backup breaks hash chain continuity
* **Impact**: False tamper detection, operational confusion
* **Mitigation**: Hash chain repair procedure, clear documentation, restore testing

---

## 12. Open Questions (If Any)

1. **Archival Strategy**: Should old audit events be archived to S3/Azure or kept in PostgreSQL indefinitely?
   * Decision needed by: Infrastructure Architect (Nadia) + Data Architect (Chen)

2. **Retention Period**: What is the minimum retention period for audit events (7 years for compliance)?
   * Decision needed by: Security Architect (Fatima) + Legal/Compliance

3. **Event Sanitization**: Should audit events automatically redact sensitive data (PII, secrets) before logging?
   * Decision needed by: Security Architect (Fatima) + Domain Architect (Yusuf)

4. **Real-Time Streaming**: Should audit log support real-time event streaming (Kafka, event bus) for monitoring?
   * Decision needed by: Infrastructure Architect (Nadia) + CTO (Rena)

5. **Hash Algorithm**: SHA-256 (widely supported) vs. BLAKE3 (faster) for hash chain?
   * Decision needed by: Security Architect (Fatima) + Performance benchmarks

---

## Appendix: Timeline and Status

* **Document Created:** 2025-01-XX
* **Last Updated:** 2025-01-XX
* **Author:** Architecture Team
* **Reviewers:**
  * [ ] Fatima Al-Hassan (Security Architect) - Cryptographic integrity and compliance requirements
  * [ ] Nadia Kovacs (Infrastructure Architect) - PostgreSQL performance and archival
  * [ ] Rena Okafor (CTO) - Architecture alignment with layered system

**Next Step:** CTO review and sign-off before proceeding to Session 2 (question generation)

---

**END OF AUDIT LOG AREA CONTEXT**
