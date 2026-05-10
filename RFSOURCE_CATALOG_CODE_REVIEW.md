# RFSource Catalog - Code Review

**Crate:** `crates/rfsource-catalog`  
**Purpose:** PostgreSQL-backed Artifact Registry for `.rfsource` metadata  
**Review Date:** 2026-05-07  
**Reviewer:** Domain Audit (Phase 3)

---

## Executive Summary

**Status:** ⚠️ **MVP STUB** - In-memory only, PostgreSQL not implemented

**Key Findings:**
* ✅ Clean architecture (3 files, 382 lines total)
* ✅ 7 tests cover core scenarios
* ⚠️ **In-memory only** - Documentation says "PostgreSQL-backed" but implementation is HashMap
* ⚠️ **No persistence** - Data lost on restart
* ⚠️ **No concurrent access** - Single-threaded HashMap, no locks
* ⚠️ **No transactions** - Multiple registry operations not atomic
* ✅ Grant checking implemented (wildcards supported)
* ✅ Soft delete (deleted flag, not hard delete)

**Critical Gap:** Claims to be "PostgreSQL-backed" but has NO database code. This is misleading naming/documentation.

**Production Readiness:** ❌ NOT READY - In-memory only, no persistence

---

## Files Reviewed

| File | Lines | Purpose |
|------|-------|---------|
| `src/error.rs` | 24 | Error types |
| `src/models.rs` | 140 | Data models (4 types) |
| `src/registry.rs` | 218 | ArtifactRegistry implementation + tests |
| **Total** | **382** | |

---

## Documentation vs. Reality Gap

### What Documentation Says

```rust
//! # rfsource-catalog — Artifact Registry
//!
//! **PostgreSQL-backed Artifact Registry** for `.rfsource` metadata.
```

**Schema section lists:**
```rust
//! ## Database Schema
//!
//! All tables use the `artifact_registry` schema with `ar_` prefix:
//!
//! - `ar_artifacts` — artifact registry entries
//! - `ar_versions` — version metadata
//! - `ar_grants` — grant bindings per artifact
//! - `ar_tags` — artifact tags/labels
```

### What Code Actually Is

```rust
#[derive(Clone, Debug)]
pub struct ArtifactRegistry {
    artifacts: HashMap<String, ArtifactEntry>,      // ← IN-MEMORY HashMap
    versions: HashMap<String, ArtifactVersionEntry>, // ← IN-MEMORY HashMap
    grants: Vec<GrantBinding>,                       // ← IN-MEMORY Vec
    tags: Vec<ArtifactTag>,                          // ← IN-MEMORY Vec
}
```

**Reality:** Zero database code. No SQL. No connection pools. Pure in-memory.

**Comment acknowledges this:**
```rust
/// In this initial implementation, the registry is in-memory.
/// When the schema is finalized, a PostgreSQL-backed implementation
/// will be added in `control-store`.
```

**Issue:** Top-level docs say "PostgreSQL-backed" but implementation says "initial... in-memory". Contradictory.

---

## Data Models (4 types)

### 1. ArtifactEntry

```rust
pub struct ArtifactEntry {
    pub artifact_id: String,
    pub logical_path: String,
    pub language: String,
    pub current_version_id: String,
    pub risk_level: String,
    pub allowed_grants: Vec<String>,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Good:**
* Soft delete (deleted flag)
* Timestamps for audit
* Grant list for access control

**Issues:**
* bare `String` IDs (not typed IDs from rfsource-core)
* `risk_level` and `language` are strings (should be enums)

### 2. ArtifactVersionEntry

```rust
pub struct ArtifactVersionEntry {
    pub version_id: String,
    pub artifact_id: String,
    pub commit_id: String,
    pub rfsource_path: Option<String>,  // ← Which .rfsource file
    pub content_hash: String,
    pub line_count: u32,
    pub created_at: DateTime<Utc>,
}
```

**Good:**
* Tracks which `.rfsource` file contains this version
* Content hash for integrity

**Issues:**
* No parent_version_id (can't build version DAG)
* No size_bytes (only line_count)

### 3. GrantBinding

```rust
pub struct GrantBinding {
    pub grant: String,           // e.g. "SGL-BACKEND-READ"
    pub artifact_id: String,
    pub created_at: DateTime<Utc>,
}
```

**Good:**
* Simple model
* Supports wildcard "*"

**Issues:**
* No actor/subject (who has the grant?)
* No expiration
* No revocation tracking

### 4. ArtifactTag

```rust
pub struct ArtifactTag {
    pub artifact_id: String,
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}
```

**Good:** Simple key-value tagging

**Issues:** No uniqueness constraints (can add duplicate tags)

---

## API Operations

### Artifact Operations (5)

1. `register_artifact()` - Add new artifact
2. `get_artifact()` - Fetch by ID
3. `list_artifacts()` - List all (with deleted filter)
4. `update_artifact()` - Update metadata
5. `delete_artifact()` - Soft delete (sets flag)

**Quality:** Clean, straightforward

### Version Operations (3)

1. `register_version()` - Add new version
2. `get_version()` - Fetch by ID
3. `list_versions()` - Get all versions for artifact

**Quality:** Clean

**Missing:** Version history query (parent chain), version diff

### Grant Operations (3)

1. `add_grant()` - Bind grant to artifact
2. `list_grants()` - List grants for artifact
3. `grant_allows()` - Check if grant permits access

**Quality:** Basic but functional

**Issues:**
* `grant_allows()` only checks artifact grants, not actor grants
* No grant revocation
* No grant expiration
* No audit trail for grant checks

### Tag Operations (2)

1. `tag_artifact()` - Add tag
2. `list_tags()` - List tags for artifact

**Quality:** Minimal

**Missing:** Remove tag, update tag, tag search

---

## Grant Checking Logic

```rust
pub fn grant_allows(&self, artifact_id: &str, grant: &str) -> bool {
    self.grants
        .iter()
        .any(|g| g.artifact_id == artifact_id && (g.grant == "*" || g.grant == grant))
}
```

**Good:**
* Supports wildcard "*"
* Simple boolean check

**Problems:**
1. **Inverted model** - Checks if artifact HAS grant, not if ACTOR has grant
2. **No actor parameter** - Who is asking? Not checked
3. **No audit** - Denied checks not logged
4. **Linear scan** - O(n) where n = total grants (should be indexed)

**Correct model should be:**
```rust
pub fn actor_can_access(&self, actor: &str, artifact_id: &str) -> bool {
    // 1. Get actor's grants
    // 2. Get artifact's required grants
    // 3. Check intersection
    // 4. Log result
}
```

---

## Testing (7 tests)

```rust
#[test]
fn test_register_and_get()           // Basic CRUD
fn test_duplicate_register_fails()   // Duplicate prevention
fn test_soft_delete()                // Soft delete behavior
fn test_grant_check()                // Grant matching
fn test_wildcard_grant()             // "*" grant
fn test_register_version()           // Version registration
```

**Coverage:** Good for in-memory implementation

**Missing:**
* Concurrent access (not applicable for single-threaded HashMap)
* Large-scale operations (10K artifacts)
* Grant denial scenarios
* Tag operations
* Update operations

---

## Concurrency Issues

**Current design:**
```rust
pub struct ArtifactRegistry {
    artifacts: HashMap<String, ArtifactEntry>,  // ← NOT thread-safe
    // ...
}
```

**Problems:**
1. **No Sync/Send** - Cannot share across threads safely
2. **No locks** - Concurrent reads/writes will race
3. **No transactions** - Multiple operations not atomic

**When PostgreSQL is added:**
* Need connection pool (e.g., deadpool-postgres)
* Need transaction support
* Need row-level locking for concurrent updates
* Need SERIALIZABLE isolation for consistency

---

## Missing PostgreSQL Implementation

**What needs to be built:**

### 1. Database Schema (SQL)

```sql
CREATE SCHEMA artifact_registry;

CREATE TABLE artifact_registry.ar_artifacts (
    artifact_id TEXT PRIMARY KEY,
    logical_path TEXT NOT NULL,
    language TEXT NOT NULL,
    current_version_id TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    allowed_grants TEXT[] NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE artifact_registry.ar_versions (
    version_id TEXT PRIMARY KEY,
    artifact_id TEXT NOT NULL REFERENCES artifact_registry.ar_artifacts(artifact_id),
    commit_id TEXT NOT NULL,
    rfsource_path TEXT,
    content_hash TEXT NOT NULL,
    line_count INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE artifact_registry.ar_grants (
    grant_id SERIAL PRIMARY KEY,
    grant_token TEXT NOT NULL,
    artifact_id TEXT NOT NULL REFERENCES artifact_registry.ar_artifacts(artifact_id),
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (grant_token, artifact_id)
);

CREATE TABLE artifact_registry.ar_tags (
    tag_id SERIAL PRIMARY KEY,
    artifact_id TEXT NOT NULL REFERENCES artifact_registry.ar_artifacts(artifact_id),
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (artifact_id, key, value)
);
```

### 2. Connection Pool Setup

```rust
use deadpool_postgres::{Config, Pool, Runtime};

pub struct ArtifactRegistry {
    pool: Pool,
}

impl ArtifactRegistry {
    pub async fn new(database_url: &str) -> Result<Self> {
        let config = database_url.parse::<Config>()?;
        let pool = config.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;
        Ok(Self { pool })
    }
}
```

### 3. Async Operations

```rust
pub async fn register_artifact(&self, entry: ArtifactEntry) -> Result<()> {
    let client = self.pool.get().await?;
    client.execute(
        "INSERT INTO artifact_registry.ar_artifacts (...) VALUES (...)",
        &[/* params */]
    ).await?;
    Ok(())
}
```

**Effort:** 2-3 weeks for full PostgreSQL implementation

---

## Mapping to Audit Questions

### D2 (Test Coverage)

**D2-010: Database interactions tested?**
* ❌ **Gap** - No database, so no database tests

**D2-012: Race conditions tested?**
* ❌ **Gap** - No concurrency tests (single-threaded HashMap)

### D4 (Versioning)

**D4-013: Schema changes versioned?**
* ❌ **Gap** - Schema not finalized, no migrations

**D4-014: Schema migrations tested?**
* ❌ **Gap** - No migrations exist

**D4-015: Schema changes rolled back?**
* ❌ **Gap** - No migration rollback

### D6 (Security)

**D6-002: Permissions checked at every layer?**
* ⚠️ **Partial** - Grant checking exists but inverted logic

**D6-004: Permissions audited?**
* ❌ **Gap** - No audit logging for grant checks

**D6-008: All operations logged?**
* ⚠️ **Partial** - `#[instrument]` on some methods but no persistent audit

### D9 (Operational)

**D9-019: Database migrations automated?**
* ❌ **Gap** - No migrations, no automation

**D9-022: Backup while running?**
* ❌ **Gap** - No database, no backup strategy

---

## Critical Issues

### 1. False Documentation (MINOR but MISLEADING)

**Issue:** Top-level docs claim "PostgreSQL-backed" but implementation is in-memory

**Recommendation:** Update docs to say "In-memory (PostgreSQL planned)"

### 2. No Persistence (CRITICAL for production)

**Issue:** All metadata lost on restart

**Impact:** Cannot deploy in production, cannot recover from crashes

**Recommendation:** Implement PostgreSQL backend (2-3 weeks)

### 3. Inverted Grant Model (IMPORTANT)

**Issue:** Checks if artifact HAS grant, not if ACTOR can access

**Impact:** Cannot enforce actor-level permissions correctly

**Recommendation:** Redesign grant_allows() to take actor parameter (3 days)

### 4. No Concurrency Support (IMPORTANT)

**Issue:** HashMap not thread-safe, no locks

**Impact:** Cannot use in multi-threaded service

**Recommendation:** Switch to Arc<RwLock<HashMap>> for MVP, or PostgreSQL for production (1 day for MVP, 3 weeks for production)

---

## Recommendations

### Priority 1: Fix Documentation
* Update top-level docs to acknowledge in-memory MVP
* Remove "PostgreSQL-backed" until actually implemented
* Add "Limitations" section listing no-persistence, no-concurrency

**Effort:** 1 hour

### Priority 2: Add Thread Safety (MVP)
```rust
use std::sync::{Arc, RwLock};

pub struct ArtifactRegistry {
    artifacts: Arc<RwLock<HashMap<String, ArtifactEntry>>>,
    // ...
}
```

**Effort:** 1 day

### Priority 3: Implement PostgreSQL Backend (Production)
* Define schema (SQL migrations)
* Setup connection pool
* Convert all operations to async
* Add transaction support
* Write integration tests

**Effort:** 2-3 weeks

### Priority 4: Fix Grant Model
* Add actor parameter to grant_allows()
* Query actor grants from authority system
* Check intersection with artifact grants
* Add audit logging

**Effort:** 3 days

---

## Positive Findings

1. ✅ **Clean API** - Operations are simple and intuitive
2. ✅ **Good tests** - 7 tests cover core scenarios
3. ✅ **Soft delete** - Preserves history
4. ✅ **Wildcard grants** - "*" supported
5. ✅ **Type safety** - No unwrap(), proper Result<T>
6. ✅ **Tracing** - #[instrument] on key operations

---

## Production Readiness: ❌ NOT READY

**Blockers:**
* No persistence (data lost on restart)
* No concurrency support (single-threaded)
* No PostgreSQL implementation (claimed but missing)
* Grant model inverted (cannot enforce actor permissions)

**Estimated Effort to Production:**
* Fix docs: 1 hour
* Thread safety (MVP): 1 day
* PostgreSQL backend: 2-3 weeks
* Fix grant model: 3 days
* **Total:** 3-4 weeks

---

## Questions Answered (9 total)

**D2 (Test Coverage): 2 gaps**
* D2-010: Database interactions ❌
* D2-012: Race conditions ❌

**D4 (Versioning): 3 gaps**
* D4-013: Schema versioning ❌
* D4-014: Schema migrations ❌
* D4-015: Schema rollback ❌

**D6 (Security): 2 partial, 1 gap**
* D6-002: Permissions checked ⚠️ (inverted logic)
* D6-004: Audit logging ❌
* D6-008: Operations logged ⚠️ (tracing only)

**D9 (Operational): 2 gaps**
* D9-019: Migrations automated ❌
* D9-022: Backup strategy ❌

---

**END OF RFSOURCE-CATALOG CODE REVIEW**
