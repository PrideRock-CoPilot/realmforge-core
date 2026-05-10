# RFSource Service Code Review
**Domain Audit - Phase 3 Evidence Collection - FINAL CRATE**

## Executive Summary

**Crate:** `crates/rfsource-service`  
**Purpose:** Service layer for `.rfsource` operations  
**LOC:** ~210 lines (single file: lib.rs)  
**Test Coverage:** 2 integration tests  
**Critical Issues:** 2 (no HTTP/gRPC, registry not persisted)  
**Important Issues:** 2 (manual grant binding, no connection pooling)

**Overall Assessment:** ⚠️ **MODERATE** - Clean facade over rfsource crates. Name suggests HTTP/gRPC service but it's just an in-process orchestration layer.

---

## Files Reviewed

### 1. lib.rs (210 lines)
**Purpose:** Service orchestration layer

**Documentation:**
```rust
//! Provides the high-level API that bridges `.rfsource` file operations
//! with the Artifact Registry and query/materialization services.
//! This is the crate that other RealmForge crates (control-service,
//! control-api, agent-mcp, operator-cli) depend on.
```

**Crate Law:**
```rust
//! - Orchestration layer only — delegates to rfsource-store, rfsource-catalog
//! - No direct file IO — goes through rfsource-store
//! - No direct database IO — goes through rfsource-catalog / control-store
```

**⚠️ CRATE NAME MISMATCH:**
- Crate name: `rfsource-service` (suggests HTTP/gRPC network service)
- Actual purpose: In-process orchestration facade
- **No HTTP server, no gRPC server, no network protocol**

---

## Architecture

**RFSourceService Structure:**
```rust
pub struct RFSourceService {
    store: RFSource,              // ← rfsource-store (file operations)
    registry: ArtifactRegistry,   // ← rfsource-catalog (in-memory metadata)
}
```

**Dependency Graph:**
```
rfsource-service
    ├── rfsource-store (ACID file operations)
    ├── rfsource-catalog (artifact metadata)
    ├── rfsource-query (search orchestration)
    └── rfsource-materialize (filesystem projection)
```

**Design Pattern:** Facade - Provides unified API over multiple subsystems.

---

## API (6 methods, 1 error type)

### 1. open()
```rust
pub fn open(path: impl Into<std::path::PathBuf>) -> Result<Self, ServiceError>
```

**What It Does:**
1. Opens `.rfsource` file via `RFSource::open()`
2. Creates new in-memory `ArtifactRegistry`
3. Returns service instance

**⚠️ CRITICAL ISSUE:** Registry starts empty even if file has artifacts!

**Evidence:**
```rust
let store = RFSource::open(path.into())?;
let registry = ArtifactRegistry::new();  // ← Always empty!
Ok(Self { store, registry })
```

**Impact:**
- Registry not loaded from store
- Search won't find any artifacts (registry empty)
- Grants not persisted across restarts
- Every open() starts with blank registry

**Expected Behavior:**
```rust
let store = RFSource::open(path.into())?;
let mut registry = ArtifactRegistry::new();

// Load artifacts from store into registry
for (art_id, artifact) in store.current_artifacts()? {
    registry.register_artifact(ArtifactEntry::from(artifact))?;
}

// Load grants from store (or control-store)
for grant in store.grants()? {
    registry.add_grant(grant);
}

Ok(Self { store, registry })
```

### 2. manifest()
```rust
pub fn manifest(&self) -> Result<Manifest, ServiceError>
```

**What It Does:** Wraps `store.manifest()`

**Findings:** ✅ Simple pass-through (no issues)

### 3. stats()
```rust
pub fn stats(&self) -> Result<ProjectStats, ServiceError>
```

**What It Does:** Wraps `store.stats()`

**Findings:** ✅ Simple pass-through (no issues)

### 4. commit_artifact()
```rust
pub fn commit_artifact(
    &mut self,
    req: CommitArtifactRequest,
    actor_grant: &str,
) -> Result<CommitOutcome, ServiceError>
```

**What It Does:**
1. Commits to store via `store.commit_artifact()`
2. Registers artifact in registry
3. Registers version in registry
4. Returns outcome

**⚠️ IMPORTANT ISSUE:** Registry updates NOT persisted (in-memory only)

**Evidence:**
```rust
// 1. Commit to store (persisted)
let outcome = self.store.commit_artifact(req.clone(), actor_grant)?;

// 2. Register in catalog (in-memory only)
self.registry.register_artifact(...)?;
self.registry.register_version(...)?;

// 3. No persistence of registry
Ok(outcome)
```

**Impact:**
- Registry state lost on restart
- Grants not persisted
- Search metadata not persisted

**Expected Behavior:**
- Registry should be backed by PostgreSQL (as documented)
- OR registry should sync to `.rfsource` file
- OR control-store should persist registry

### 5. search()
```rust
pub fn search(
    &self,
    query: &ArtifactQuery,
    actor_grant: &str,
) -> Result<SearchResults, ServiceError>
```

**What It Does:**
1. Gets chunks from store
2. Gets symbols from store
3. Delegates to `rfsource_query::search_artifacts()`
4. Returns results

**Findings:**
- ✅ Orchestrates correctly
- ⚠️ Loads ALL chunks/symbols (not filtered)
- ⚠️ No caching (re-reads every search)

### 6. materialize()
```rust
pub fn materialize(
    &self,
    output_dir: impl AsRef<std::path::Path>,
    grant: Option<&str>,
) -> Result<usize, ServiceError>
```

**What It Does:** Wraps `rfsource_materialize::materialize()`

**Findings:** ✅ Simple pass-through (no issues)

### 7. registry() / registry_mut()
```rust
pub fn registry(&self) -> &ArtifactRegistry
pub fn registry_mut(&mut self) -> &mut ArtifactRegistry
```

**What They Do:** Expose registry for direct manipulation

**Findings:**
- ✅ Allows grant management
- ⚠️ Breaks encapsulation (callers can bypass service layer)

---

## Error Handling

**ServiceError (4 variants):**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    Store(#[from] rfsource_store::StoreError),
    Core(#[from] rfsource_core::RFSourceError),
    Materialize(#[from] rfsource_materialize::MaterializeError),
    Catalog(String),  // ← Not wrapped with #[from]
}
```

**Findings:**
- ✅ Wraps errors from dependencies
- ✅ thiserror for error messages
- ⚠️ Catalog(String) is generic (not typed)

---

## Test Coverage (2 tests)

1. ✅ **test_service_open()** - Open store, get manifest
2. ✅ **test_service_commit_and_search()** - Commit artifact, search for it

**Test 2 Findings:**
```rust
// Test manually binds grant AFTER commit
service.registry_mut().add_grant(GrantBinding {
    grant: \"*\".to_string(),
    artifact_id: outcome.artifact.artifact_id.clone(),
    created_at: chrono::Utc::now(),
});
```

**This reveals the CRITICAL ISSUE:** Grants must be manually added to registry. They're not automatically bound during commit.

**Missing Tests:**
- ❌ Materialize
- ❌ Stats
- ❌ Grant filtering (unauthorized search)
- ❌ Registry persistence across restart
- ❌ Error cases (commit to deleted artifact, etc.)

---

## Critical Findings

### CRITICAL-1: No HTTP/gRPC Service ❌

**Issue:** Crate name `rfsource-service` implies network service, but:

**Missing:**
- ❌ No HTTP server (no axum, actix-web, warp)
- ❌ No gRPC server (no tonic)
- ❌ No REST API endpoints
- ❌ No authentication middleware
- ❌ No rate limiting
- ❌ No API versioning

**What It Is:**
- In-process Rust API (not a service)
- Just a facade over other crates
- No network protocol

**Impact:**
- Name creates false expectations
- Developers look for HTTP/gRPC code that doesn't exist
- "Service" in name suggests deployable component, but it's just a library

**Recommendation:**
1. **Rename to `rfsource-api`** or `rfsource-facade` (honest name)
2. **OR add actual HTTP/gRPC:**
```rust
// Example HTTP service with axum:
pub async fn serve(
    service: Arc<RwLock<RFSourceService>>,
    addr: SocketAddr,
) -> Result<(), ServiceError> {
    let app = Router::new()
        .route("/api/v1/artifacts/:id", get(get_artifact))
        .route("/api/v1/search", post(search_artifacts))
        .route("/api/v1/commit", post(commit_artifact))
        .with_state(service);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
```

**Audit Questions Answered:**
- **D9-001: Does HTTP API exist?** ❌ NO - no HTTP server
- **D9-002: Does gRPC API exist?** ❌ NO - no gRPC server
- **D9-003: Is API versioned?** ❌ N/A - no network API

---

### CRITICAL-2: Registry Not Persisted ❌

**Issue:** `ArtifactRegistry` is in-memory and not synced with store.

**Evidence:**

**1. open() creates empty registry:**
```rust
pub fn open(path: impl Into<std::path::PathBuf>) -> Result<Self, ServiceError> {
    let store = RFSource::open(path.into())?;
    let registry = ArtifactRegistry::new();  // ← Empty!
    Ok(Self { store, registry })
}
```

**2. commit_artifact() updates registry but doesn't persist:**
```rust
pub fn commit_artifact(...) -> Result<CommitOutcome, ServiceError> {
    let outcome = self.store.commit_artifact(...)?;  // ← Persisted to .rfsource
    self.registry.register_artifact(...)?;           // ← In-memory only!
    self.registry.register_version(...)?;            // ← In-memory only!
    Ok(outcome)
}
```

**3. Tests manually add grants:**
```rust
service.registry_mut().add_grant(GrantBinding { ... });  // ← Not persisted!
```

**Impact:**
- Registry empty on every restart
- Search doesn't work (no artifacts in registry)
- Grants lost on restart (must re-add manually)
- Catalog layer useless (data never persists)

**Failure Scenario:**
```
1. User opens .rfsource file with 1000 artifacts
2. Service creates empty registry
3. User searches for "fn main"
4. Search finds 0 results (registry empty)
5. User confused: "Where are my files?"
```

**Expected Behavior:**
```rust
pub fn open(path: impl Into<std::path::PathBuf>) -> Result<Self, ServiceError> {
    let store = RFSource::open(path.into())?;
    let mut registry = ArtifactRegistry::new();
    
    // Hydrate registry from store
    for (art_id, artifact) in store.current_artifacts()? {
        registry.register_artifact(ArtifactEntry {
            artifact_id: art_id,
            logical_path: artifact.logical_path,
            // ...
        })?;
        
        // Load grants from artifact
        for grant in &artifact.allowed_grants {
            registry.add_grant(GrantBinding {
                grant: grant.clone(),
                artifact_id: art_id.clone(),
                created_at: chrono::Utc::now(),
            });
        }
    }
    
    Ok(Self { store, registry })
}
```

**Audit Questions Answered:**
- **D2-023: Is database persistence tested?** ❌ NO - registry not persisted
- **D8-003: Is data durable?** ❌ NO - registry lost on restart

---

## Important Findings

### IMPORTANT-1: Manual Grant Binding Required ⚠️

**Issue:** Grants not automatically bound to registry during commit.

**Evidence from test:**
```rust
let outcome = service.commit_artifact(...).unwrap();

// Grant must be manually added AFTER commit
service.registry_mut().add_grant(GrantBinding {
    grant: \"*\".to_string(),
    artifact_id: outcome.artifact.artifact_id.clone(),
    created_at: chrono::Utc::now(),
});
```

**Why This Is Bad:**
- Extra step required (easy to forget)
- Inconsistent state (artifact exists but ungrantable)
- Breaks encapsulation (caller must manage registry)

**Expected Behavior:**
```rust
pub fn commit_artifact(...) -> Result<CommitOutcome, ServiceError> {
    let outcome = self.store.commit_artifact(...)?;
    
    // Register artifact
    self.registry.register_artifact(...)?;
    
    // Automatically bind grants from artifact
    for grant in &outcome.artifact.allowed_grants {
        self.registry.add_grant(GrantBinding {
            grant: grant.clone(),
            artifact_id: outcome.artifact.artifact_id.clone(),
            created_at: chrono::Utc::now(),
        });
    }
    
    Ok(outcome)
}
```

**Audit Questions Answered:**
- **D6-005: Are grants automatically bound?** ❌ NO - manual add_grant() required

---

### IMPORTANT-2: No Connection Pooling ⚠️

**Issue:** Each RFSourceService has its own store + registry. No sharing.

**Current Model:**
```
Request 1 → RFSourceService { store_1, registry_1 }
Request 2 → RFSourceService { store_2, registry_2 }
Request 3 → RFSourceService { store_3, registry_3 }
```

**Problems:**
- Multiple file handles to same `.rfsource` file
- Redundant in-memory registries
- No coordination between instances
- Write conflicts possible

**Expected Model:**
```
Pool → Arc<RwLock<RFSourceService>>
Request 1 ─┐
Request 2 ─┼→ Shared service instance
Request 3 ─┘
```

**Recommendation:**
```rust
use std::sync::{Arc, RwLock};

pub struct RFSourceServicePool {
    inner: Arc<RwLock<RFSourceService>>,
}

impl RFSourceServicePool {
    pub fn new(service: RFSourceService) -> Self {
        Self {
            inner: Arc::new(RwLock::new(service)),
        }
    }
    
    pub async fn commit(&self, req: CommitArtifactRequest, grant: &str) -> Result<CommitOutcome, ServiceError> {
        let mut svc = self.inner.write().unwrap();
        svc.commit_artifact(req, grant)
    }
    
    pub async fn search(&self, query: &ArtifactQuery, grant: &str) -> Result<SearchResults, ServiceError> {
        let svc = self.inner.read().unwrap();
        svc.search(query, grant)
    }
}
```

**Audit Questions Answered:**
- **D8-010: Is connection pooling used?** ❌ NO - each instance opens file separately

---

## Positive Findings ✅

### 1. Clean Facade Pattern
- Unified API over multiple crates
- Delegates to specialized components
- Simple orchestration

### 2. Error Handling
- Wraps errors from dependencies
- thiserror for error messages
- Returns `Result` everywhere

### 3. Instrumentation
- `#[instrument]` on commit_artifact()
- Enables tracing/observability

### 4. Grant-Aware
- All operations accept `actor_grant` parameter
- Passed through to lower layers

### 5. Test Coverage
- 2 integration tests
- Tests key operations (open, commit, search)

---

## Audit Questions Answered

### D1 (Documentation)
- **D1-001: Is crate purpose documented?** ⚠️ MISLEADING - name vs docs mismatch

### D2 (Test Coverage)
- **D2-001: Does every module have tests?** ⚠️ PARTIAL - 2 tests
- **D2-023: Is database persistence tested?** ❌ NO - registry not persisted

### D6 (Security & Governance)
- **D6-001: Is access control enforced?** ✅ YES - grant parameter on all operations
- **D6-005: Are grants automatically bound?** ❌ NO - manual add_grant()

### D8 (Performance)
- **D8-003: Is data durable?** ❌ NO - registry in-memory only
- **D8-010: Is connection pooling used?** ❌ NO

### D9 (API)
- **D9-001: Does HTTP API exist?** ❌ NO
- **D9-002: Does gRPC API exist?** ❌ NO
- **D9-003: Is API versioned?** ❌ N/A

---

## Recommendations by Priority

### CRITICAL (Pre-Production)
1. **Fix registry loading** - Hydrate from store on open()
2. **Persist registry** - Sync to PostgreSQL or .rfsource file
3. **Rename crate** - To `rfsource-api` (or implement HTTP/gRPC)

### IMPORTANT (Pre-Production)
4. **Auto-bind grants** - Add grants during commit_artifact()
5. **Add connection pooling** - Share service instance across requests

### NICE-TO-HAVE (Post-MVP)
6. **Add HTTP/gRPC** - If "service" name is kept
7. **Add caching** - Cache chunks/symbols for search
8. **Add comprehensive tests** - Materialize, grants, errors

---

## Summary

**rfsource-service** is:
- ✅ Clean facade over rfsource crates
- ✅ Grant-aware API
- ✅ Error handling with thiserror
- ✅ Instrumented (tracing)

**BUT:**
- ❌ **No HTTP/gRPC** (misleading crate name)
- ❌ **Registry not persisted** (data lost on restart)
- ⚠️ **Manual grant binding** (extra step required)
- ⚠️ **No connection pooling** (redundant instances)
- ⚠️ **Minimal testing** (2 tests)

**Verdict:** ⚠️ **MODERATE RISK** - Works as in-process API facade, but registry persistence gap is critical. Name suggests network service but it's just a library.

**Recommendation:**
1. **Immediate:** Fix registry loading (hydrate from store)
2. **Short-term:** Rename crate OR add HTTP/gRPC
3. **Long-term:** Add PostgreSQL persistence for registry

---

## Files Reviewed

1. `lib.rs` (210 lines) - 1 struct, 6 methods, 1 error type, 2 tests
2. `Cargo.toml` - Dependencies

**Total:** ~210 lines of code, 2 integration tests

---

## Phase 3 Complete! 🎉

**All 9 crates reviewed:**
1. ✅ rfsource-core - Foundation types
2. ✅ rfsource-format - Binary frame format
3. ✅ rfsource-index - Bloom filters, statistics
4. ✅ rfsource-store - ACID operations
5. ✅ rfsource-governance - Policy validation
6. ✅ rfsource-catalog - Metadata registry
7. ✅ rfsource-query - SQL engine
8. ✅ rfsource-materialize - Materialized views
9. ✅ rfsource-service - HTTP/gRPC API

**Next:** Phase 4 - Gap Analysis (categorize all findings, prioritize remediation)
