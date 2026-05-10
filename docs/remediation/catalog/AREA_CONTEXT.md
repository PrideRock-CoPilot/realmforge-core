# Catalog System — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of the Catalog System to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION  
**Created:** 2025-01-XX  
**Author:** Architecture Team  
**Reviewers:** Chen Wei (Data Architect), Yusuf Osman (Domain Architect), Rena Okafor (CTO)

**Important:** This document captures WHAT the Catalog System is supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

The Catalog System is RealmForge's metadata registry - a hierarchical namespace for discovering, organizing, and querying all governed resources (tables, files, models, endpoints, permissions). It exists to answer "what resources exist, where are they, who owns them, and what are they for?" without requiring direct access to the resources themselves.

The Catalog System solves three critical problems:
1. **Discovery**: Enables users and agents to find resources by name, type, owner, tags, or lineage
2. **Organization**: Provides hierarchical namespaces (tenant → project → resource type → resource) for logical grouping
3. **Metadata Management**: Centralized storage for resource properties, schemas, lineage, and access patterns

The Catalog System is NOT a data storage system - it stores metadata about resources, not the resources themselves.

---

## 2. Responsibilities

The Catalog System owns:

* **Resource Registration**: Add new resources to catalog (tables, files, models, endpoints)
* **Metadata Storage**: Store resource properties (name, type, owner, created_at, schema, description, tags)
* **Hierarchical Namespace**: Organize resources in tenant → project → resource type hierarchy
* **Discovery Queries**: Search resources by name, type, owner, tags, or full-text search
* **Schema Tracking**: Track table/file schemas and schema evolution over time
* **Lineage Tracking**: Record resource dependencies (which tables/files/models depend on each other)
* **Access Pattern Tracking**: Record resource usage (who accessed, when, how often)
* **Resource Tagging**: Apply tags for categorization (PII, production, deprecated, etc.)
* **Resource Versioning**: Track resource versions and changes over time

---

## 3. Boundaries (What This Area Does NOT Own)

The Catalog System explicitly does NOT:

* **Resource Storage**: Does not store actual data - only metadata
* **Authorization**: Does not enforce access control - relies on policy engine
* **Data Processing**: Does not transform or query data - only tracks metadata
* **Snapshot Capture**: Does not capture resource state - only tracks versions
* **User Interface**: Has no direct UI - accessed through API/CLI
* **Resource Lifecycle**: Does not create, modify, or delete resources - only registers metadata

---

## 4. Programming Language(s)

* **Primary:** Rust
  * Chosen for: Type safety in metadata handling, performance for large catalogs, strong schema validation
  * Critical for catalog layer where metadata correctness is essential

* **Secondary:** SQL (PostgreSQL)
  * Used for: Metadata storage, hierarchical queries, full-text search

**Why Rust?**  
Catalog metadata is the source of truth for resource discovery - bugs in metadata handling can make resources invisible or mislead users about resource properties. Rust's type system ensures metadata is validated on ingestion. The performance enables fast queries over large catalogs (10K+ resources).

---

## 5. High-Level Architecture

* **Layer:** Service Layer (consumes domain types, exposes catalog API)
* **Position:** Above domain model, below API/CLI, alongside control services

**Key Components:**

1. **Resource Registry**
   * `register_resource()`: Add new resource to catalog
   * `update_resource()`: Modify resource metadata
   * `deregister_resource()`: Mark resource as deleted (soft delete)
   * `get_resource()`: Fetch resource metadata by ID

2. **Discovery Engine**
   * `search_resources()`: Full-text search by name, description, tags
   * `list_resources()`: Paged list by type, owner, project
   * `find_by_tag()`: Query resources with specific tags
   * `find_by_lineage()`: Query upstream/downstream dependencies

3. **Schema Manager**
   * `register_schema()`: Store table/file schema
   * `track_schema_change()`: Record schema evolution
   * `get_schema()`: Fetch current schema for resource
   * `compare_schemas()`: Diff two schema versions

4. **Lineage Tracker**
   * `record_dependency()`: Link resources (table A depends on table B)
   * `get_upstream()`: Fetch all dependencies for a resource
   * `get_downstream()`: Fetch all dependents of a resource
   * `compute_impact()`: Estimate impact of resource change

5. **Tagging System**
   * `add_tag()`: Apply tag to resource (PII, production, deprecated)
   * `remove_tag()`: Remove tag from resource
   * `get_tagged()`: Find all resources with specific tag
   * `get_tags()`: List all tags for a resource

**Data Flow:**
* Service layer → register_resource() → Resource Registry → PostgreSQL
* API/CLI → search_resources() → Discovery Engine → PostgreSQL → Return Results

---

## 6. Key Concepts

* **Resource**: Governed entity with metadata (table, file, model, endpoint)
* **Metadata**: Properties of a resource (name, type, owner, schema, description, tags)
* **Hierarchical Namespace**: tenant.project.resource_type.resource_name
* **Schema**: Structure of a table or file (columns, types, constraints)
* **Lineage**: Dependency graph between resources
* **Tag**: Label for categorization (PII, production, deprecated, etc.)
* **Discovery**: Finding resources by search criteria
* **Access Pattern**: Usage metrics (who accessed, when, frequency)

---

## 7. Success Criteria

**Correctness:**
* Every registered resource is discoverable
* Metadata is accurate (no stale or incorrect properties)
* Lineage is complete (no missing dependencies)
* Schema versions are tracked correctly

**Performance:**
* Resource registration latency p99 < 10ms
* Discovery query latency p99 < 50ms (up to 1000 results)
* Lineage query latency p99 < 100ms (up to 100 dependencies)
* Full-text search latency p99 < 200ms

**Reliability:**
* Catalog is always available (catalog unavailability does not block operations)
* Metadata is durable (no loss on crash)
* Queries return consistent results (no flaky search)

**Security:**
* Catalog queries are authorized via policy engine
* Metadata does not leak sensitive information (sanitized)
* Resource discovery respects access controls

---

## 8. Dependencies

The Catalog System depends on:

* **PostgreSQL**: For metadata storage, hierarchical queries, full-text search
  * Must support: JSON columns for flexible metadata, full-text search, foreign keys for lineage

* **Authority Domain**: Provides ActorID, ProjectID, TenantID types for resource ownership

* **Control Store**: Persists catalog metadata via store adapter

---

## 9. Consumers

The Catalog System is consumed by:

* **Data Engineers**: Discover tables and files for pipelines
* **Data Scientists**: Find datasets and models for ML workflows
* **Agents**: Discover resources for autonomous operations
* **API/CLI**: Expose catalog queries to users and scripts
* **Monitoring Systems**: Track resource usage patterns
* **Compliance Tools**: Identify PII resources, track data lineage

All consumers interact through the catalog service layer - none access PostgreSQL directly.

---

## 10. Production Readiness Considerations

**Data Integrity:**
* Metadata must be accurate (validation on ingestion)
* Lineage must be complete (no broken dependencies)
* Schema versions must be immutable (no modification)

**Failure Modes:**
* **PostgreSQL Unavailable**: Catalog queries fail gracefully, operations continue
* **Metadata Inconsistency**: Periodic validation, repair procedures
* **Lineage Break**: Detection and alerting, manual repair

**Observability:**
* Metrics: resource count, query latency, search accuracy, lineage depth
* Logs: every resource registration (resource_id, type, owner), every discovery query
* Traces: distributed tracing through catalog service

**Scalability:**
* Horizontal scaling: read replicas for discovery queries
* Vertical scaling: larger PostgreSQL instance for metadata storage
* Partitioning: shard catalog by tenant or project

---

## 11. Risk Profile

**Risk 1: Metadata Inaccuracy (HIGH)**
* **Scenario**: Catalog metadata does not match actual resource state
* **Impact**: Discovery failures, incorrect lineage, operational confusion
* **Mitigation**: Metadata validation on ingestion, periodic reconciliation

**Risk 2: Catalog Unavailability (MEDIUM)**
* **Scenario**: Catalog down, discovery queries fail
* **Impact**: Reduced discoverability, operational inefficiency
* **Mitigation**: Read replicas, graceful degradation, caching

**Risk 3: Lineage Incompleteness (MEDIUM)**
* **Scenario**: Missing dependencies in lineage graph
* **Impact**: Incorrect impact analysis, unexpected failures
* **Mitigation**: Automated lineage tracking, validation checks

**Risk 4: Search Performance Degradation (MEDIUM)**
* **Scenario**: Slow full-text search on large catalogs
* **Impact**: High latency, timeout failures, poor user experience
* **Mitigation**: Search indexing, query optimization, pagination

**Risk 5: Metadata Leakage (MEDIUM)**
* **Scenario**: Catalog exposes sensitive metadata to unauthorized users
* **Impact**: Information disclosure, security risk
* **Mitigation**: Authorization checks, metadata sanitization

---

## 12. Open Questions (If Any)

1. **Full-Text Search Engine**: Should catalog use PostgreSQL full-text search or external engine (Elasticsearch)?
   * Decision needed by: Infrastructure Architect (Nadia) + Data Architect (Chen)

2. **Lineage Tracking**: Should lineage be tracked automatically (from queries) or manually (by users)?
   * Decision needed by: Data Architect (Chen) + CTO (Rena)

3. **Schema Evolution**: Should catalog track all schema versions or only current + previous?
   * Decision needed by: Data Architect (Chen) + Backend Engineer (Dmitri)

4. **Tagging System**: Should tags be free-form or constrained to predefined set?
   * Decision needed by: Data Architect (Chen) + Security Architect (Fatima)

5. **Access Pattern Tracking**: Should catalog track every access or sample for performance?
   * Decision needed by: Infrastructure Architect (Nadia) + Data Architect (Chen)

---

## Appendix: Timeline and Status

* **Document Created:** 2025-01-XX
* **Last Updated:** 2025-01-XX
* **Author:** Architecture Team
* **Reviewers:**
  * [ ] Chen Wei (Data Architect) - Metadata schema and catalog architecture
  * [ ] Yusuf Osman (Domain Architect) - Resource types and domain integration
  * [ ] Rena Okafor (CTO) - Architecture alignment with layered system

**Next Step:** CTO review and sign-off before proceeding to Session 2 (question generation)

---

**END OF CATALOG SYSTEM AREA CONTEXT**
