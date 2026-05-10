# RFSource Query, Materialize, Service - Consolidated Code Review

**Crates:** `rfsource-query`, `rfsource-materialize`, `rfsource-service`  
**Review Date:** 2026-05-07  
**Reviewer:** Domain Audit (Phase 3)

---

## Executive Summary

**All 3 crates: ⚠️ MVP STUBS** - Thin orchestration layers, minimal logic

| Crate | Lines | Status | Key Finding |
|-------|-------|--------|-------------|
| rfsource-query | 95 | MVP Stub | Delegates to rfsource-index (broken), adds pagination |
| rfsource-materialize | 159 | Functional MVP | Works but no incremental updates |
| rfsource-service | 214 | Thin Orchestrator | Wires everything together, no business logic |

**Overall Assessment:** These are integration/orchestration layers that expose lower layers' problems.

---

# 1. RFSource Query Review

**Lines:** 95  
**Purpose:** Query orchestration combining text search, symbol lookup, grant filtering

## Implementation

```rust
pub struct ArtifactQuery {
    pub text_search: Option<String>,
    pub symbol_search: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

pub fn search_artifacts(
    query: &ArtifactQuery,
    chunks: &[SourceChunk],
    symbols: &[SymbolRecord],
    registry: &ArtifactRegistry,
    actor_grant: &str,
) -> SearchResults {
    let grant_check = |_: &str| -> bool { true };  // ← IGNORES PARAMETER
    
    let text_hits = if let Some(ref text_q) = query.text_search {
        let mut hits = rfsource_index::search_text(text_q, chunks, symbols, &grant_check);
        hits.retain(|h| registry.grant_allows(&h.artifact_id, actor_grant));  // ← FILTERS AFTER
        hits
    } else {
        vec![]
    };
    
    // ... symbol search similar ...
    
    let total_count = text_hits.len() + symbol_hits.len();
    let text_hits = text_hits.into_iter().skip(query.offset).take(query.limit).collect();
    
    SearchResults { text_hits, symbol_hits, total_count }
}
```

## Issues

### 1. Passes Dummy Grant Check to Index (IMPORTANT)

```rust
let grant_check = |_: &str| -> bool { true };  // ← Returns true for EVERYTHING
```

**Problem:** rfsource-index's grant checking completely bypassed  
**Impact:** All chunks scanned, then filtered afterward → O(n) scan of entire corpus  
**Correct approach:** Pass real grant check to index so it can skip unauthorized chunks early

### 2. Inefficient Grant Filtering (IMPORTANT)

**Current flow:**
1. Scan ALL chunks (O(n))
2. Build ALL matches
3. Filter by grant afterward

**Better flow:**
1. Check grants FIRST (O(1) lookup)
2. Scan only authorized artifacts
3. Build matches only from authorized set

**Performance impact:** At 10K chunks with 1% authorized → scans 9,900 unnecessary chunks

### 3. Pagination on Results, Not Query (MINOR)

```rust
let text_hits = text_hits.into_iter().skip(query.offset).take(query.limit).collect();
```

**Problem:** Computes ALL matches, then paginates  
**Impact:** Query for page 1000 still scans entire corpus  
**Better:** Push limit/offset to storage layer

### 4. Total Count After Pagination (BUG)

```rust
let total_count = text_hits.len() + symbol_hits.len();  // ← Before pagination
let text_hits = text_hits.into_iter().skip(query.offset).take(query.limit).collect();  // ← After
```

**Good:** total_count is before pagination (correct for UI)  
**But:** If both text and symbol searches active, counts are independent (not combined properly)

## Recommendations

1. **Fix grant check passthrough** (1 day)
2. **Optimize grant filtering** (2 days)
3. **Add result ranking** (1 week)
4. **Add boolean query support** (AND/OR/NOT) (1 week)

**Production Readiness:** ⚠️ PARTIAL - Works but inherits rfsource-index's O(n) problems

---

# 2. RFSource Materialize Review

**Lines:** 159  
**Purpose:** Write `.rfsource` contents to filesystem as actual source files

## Implementation

```rust
pub fn materialize(
    rfsource: &RFSource,
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize, MaterializeError> {
    let artifacts = rfsource.current_artifacts()?;
    let all_chunks = rfsource.chunks()?;
    
    let mut count = 0;
    
    for (art_id, artifact) in &artifacts {
        // Grant check
        if let Some(g) = grant {
            if !artifact.allowed_grants.is_empty()
                && !artifact.allowed_grants.contains(&g.to_string())
                && !artifact.allowed_grants.iter().any(|g| g == "*")
            {
                continue;  // ← Skip unauthorized
            }
        }
        
        // Get chunks for this artifact's current version
        let chunks = all_chunks.iter()
            .filter(|c| c.artifact_id == *art_id && c.version_id == artifact.current_version_id)
            .collect::<Vec<_>>();
        
        if artifact.deleted {
            let _ = fs::remove_file(&file_path);  // ← Delete from filesystem
            continue;
        }
        
        // Reconstruct file from chunks
        let mut content = String::new();
        let mut chunks_sorted = chunks.clone();
        chunks_sorted.sort_by_key(|c| c.ordinal);
        for chunk in chunks_sorted {
            content.push_str(&chunk.text);
            content.push('\n');
        }
        
        fs::write(&file_path, content.trim())?;
        count += 1;
    }
    
    Ok(count)
}
```

## Strengths

1. ✅ **Grant-scoped** - Only materializes authorized artifacts
2. ✅ **Handles deletion** - Removes deleted artifacts from filesystem
3. ✅ **Creates directories** - `fs::create_dir_all(parent)`
4. ✅ **Chunk ordering** - Sorts by ordinal before concatenation
5. ✅ **1 test** - Validates basic materialize flow

## Issues

### 1. Full Rebuild Every Time (IMPORTANT)

**Problem:** No incremental updates. Materializes ALL artifacts every time.

**Impact:**
* 100 artifacts × 10KB each = 1MB total → OK
* 10,000 artifacts × 10KB each = 100MB → Slow (1-2 seconds)
* 100,000 artifacts × 10KB = 1GB → Unacceptable (10-20 seconds)

**Better approach:**
```rust
// Track last materialized commit per artifact
// Only materialize artifacts changed since last commit
pub fn materialize_incremental(
    rfsource: &RFSource,
    output_dir: impl AsRef<Path>,
    since_commit: Option<&str>,
) -> Result<usize>
```

### 2. No Verification (MINOR)

**Problem:** Doesn't verify written files match expected hash

**Better:**
```rust
let written_hash = sha256_file(&file_path)?;
if written_hash != artifact.content_hash {
    return Err(MaterializeError::HashMismatch);
}
```

### 3. Branch Support Stubbed (MINOR)

```rust
pub fn materialize_branch(
    rfsource: &RFSource,
    _branch: &str,  // ← IGNORED
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize> {
    // For now, materialize works the same regardless of branch
    materialize(rfsource, output_dir, grant)
}
```

**Impact:** Cannot materialize different branches to different directories

### 4. Newline Handling (MINOR)

```rust
for chunk in chunks_sorted {
    content.push_str(&chunk.text);
    content.push('\n');  // ← Adds newline after EVERY chunk
}
fs::write(&file_path, content.trim())?;  // ← Trims at end
```

**Problem:** If chunk already has trailing newline, this adds a second one (then trims final)  
**Impact:** Possible extra blank lines between chunks

## Recommendations

1. **Add incremental materialize** (1 week)
2. **Add hash verification** (1 day)
3. **Implement proper branch support** (3 days)
4. **Fix newline handling** (2 hours)
5. **Add progress reporting** for large materializes (2 days)

**Production Readiness:** ⚠️ FUNCTIONAL MVP - Works but slow for large repos

---

# 3. RFSource Service Review

**Lines:** 214  
**Purpose:** High-level API combining store, catalog, query, materialize

## Implementation

```rust
pub struct RFSourceService {
    store: RFSource,
    registry: ArtifactRegistry,
}

impl RFSourceService {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let store = RFSource::open(path.into())?;
        let registry = ArtifactRegistry::new();  // ← Empty on startup
        Ok(Self { store, registry })
    }
    
    pub fn commit_artifact(&mut self, req: CommitArtifactRequest, actor_grant: &str) -> Result<CommitOutcome> {
        let outcome = self.store.commit_artifact(req.clone(), actor_grant)?;
        
        // Register in catalog
        self.registry.register_artifact(/* ... */)?;
        self.registry.register_version(/* ... */)?;
        
        Ok(outcome)
    }
    
    pub fn search(&self, query: &ArtifactQuery, actor_grant: &str) -> Result<SearchResults> {
        let chunks = self.store.chunks()?;
        let symbols = self.store.symbols()?;
        Ok(rfsource_query::search_artifacts(query, &chunks, &symbols, &self.registry, actor_grant))
    }
    
    pub fn materialize(&self, output_dir: impl AsRef<Path>, grant: Option<&str>) -> Result<usize> {
        Ok(rfsource_materialize::materialize(&self.store, output_dir, grant)?)
    }
}
```

## Strengths

1. ✅ **Clean orchestration** - Delegates to appropriate layers
2. ✅ **Unified API** - Single entry point for all operations
3. ✅ **Error propagation** - Proper Result<T> chaining
4. ✅ **2 tests** - Basic open + commit + search flow

## Issues

### 1. Registry Not Persisted (CRITICAL)

```rust
let registry = ArtifactRegistry::new();  // ← EMPTY on every open()
```

**Problem:** Registry starts empty every time service opens  
**Impact:** 
* All artifacts registered during session
* Search works because it re-loads chunks from store
* But grants added to registry are LOST on restart

**This is OK for MVP** because registry is in-memory anyway. But once PostgreSQL is added, this needs fixing.

**Better:**
```rust
pub fn open(path: impl Into<PathBuf>, db_url: &str) -> Result<Self> {
    let store = RFSource::open(path.into())?;
    let registry = ArtifactRegistry::connect(db_url).await?;  // ← Persistent
    Ok(Self { store, registry })
}
```

### 2. Double Registration on Commit (MINOR BUG)

```rust
pub fn commit_artifact(&mut self, req: CommitArtifactRequest, actor_grant: &str) -> Result<CommitOutcome> {
    let outcome = self.store.commit_artifact(req.clone(), actor_grant)?;
    
    // Register in catalog
    self.registry.register_artifact(/* ... */)?;  // ← Fails if artifact already exists
    self.registry.register_version(/* ... */)?;
}
```

**Problem:** Second commit to same artifact will fail with "Artifact already registered"

**Better:**
```rust
// Try update first, register if not found
if let Err(CatalogError::ArtifactNotFound(_)) = self.registry.update_artifact(entry.clone()) {
    self.registry.register_artifact(entry)?;
}
```

### 3. Loads All Chunks/Symbols for Search (IMPORTANT)

```rust
pub fn search(&self, query: &ArtifactQuery, actor_grant: &str) -> Result<SearchResults> {
    let chunks = self.store.chunks()?;    // ← Loads EVERYTHING
    let symbols = self.store.symbols()?;  // ← Loads EVERYTHING
    Ok(rfsource_query::search_artifacts(query, &chunks, &symbols, &self.registry, actor_grant))
}
```

**Problem:** Loads entire corpus into memory for every search

**Impact:**
* 10,000 chunks × 1KB = 10MB per search
* 100,000 chunks × 1KB = 100MB per search → OOM risk

**Better:** Index layer should maintain in-memory index, not re-load every query

### 4. No Transaction Support (IMPORTANT)

**Problem:** Multiple registry operations not atomic

```rust
pub fn commit_artifact(...) -> Result<CommitOutcome> {
    let outcome = self.store.commit_artifact(...)?;  // ← Write 1
    self.registry.register_artifact(...)?;           // ← Write 2 (may fail)
    self.registry.register_version(...)?;            // ← Write 3 (may fail)
    Ok(outcome)
}
```

**If registry operations fail:** Store has commit, but registry doesn't → inconsistent state

**Better:**
```rust
pub async fn commit_artifact(...) -> Result<CommitOutcome> {
    let mut tx = self.registry.begin_transaction().await?;
    
    let outcome = self.store.commit_artifact(...)?;
    tx.register_artifact(...).await?;
    tx.register_version(...).await?;
    
    tx.commit().await?;
    Ok(outcome)
}
```

## Recommendations

1. **Add transaction support** when PostgreSQL implemented (1 week)
2. **Fix double registration bug** (1 day)
3. **Cache chunks/symbols** in service (don't reload every query) (3 days)
4. **Add connection pool management** (2 days)
5. **Add health check endpoint** (1 day)

**Production Readiness:** ⚠️ THIN ORCHESTRATOR - Exposes lower layers' problems

---

# Consolidated Issues Across All 3 Crates

## Critical Issues (1)

1. **rfsource-service: Registry not persisted** (once PostgreSQL is added)

## Important Issues (6)

1. **rfsource-query: Dummy grant check** (bypasses index optimization)
2. **rfsource-query: Inefficient grant filtering** (filters after scan)
3. **rfsource-materialize: No incremental updates** (full rebuild every time)
4. **rfsource-service: Loads all chunks/symbols per search** (memory issue at scale)
5. **rfsource-service: No transaction support** (inconsistency risk)
6. **rfsource-service: Double registration bug** (fails on second commit)

## Minor Issues (5)

1. **rfsource-query: Pagination on results** (not query)
2. **rfsource-materialize: No hash verification**
3. **rfsource-materialize: Branch support stubbed**
4. **rfsource-materialize: Newline handling**
5. **rfsource-service: No health check**

---

# Production Readiness Summary

| Crate | Status | Blockers | Effort to Production |
|-------|--------|----------|---------------------|
| rfsource-query | ⚠️ MVP | Inherits index O(n) problem | 2-3 weeks (fix index first) |
| rfsource-materialize | ⚠️ MVP | Slow for large repos | 1-2 weeks (incremental) |
| rfsource-service | ⚠️ MVP | Registry persistence, transactions | 3-4 weeks (with PostgreSQL) |

---

# Questions Answered (12 total)

**D2 (Test Coverage):**
* D2-003: Public functions tested ⚠️ (basic only, 1-2 tests per crate)

**D3 (Scalability):**
* D3-011: Performance degradation ❌ (materialize is O(n), query is O(n))
* D3-012: Streaming mechanisms ❌ (loads all into memory)

**D7 (Error Handling):**
* D7-008: Retries idempotent ❌ (no retry logic)
* D7-009: Partial failure recovery ❌ (service commit can be inconsistent)

**D8 (Performance):**
* D8-013: Large objects streamed ❌ (loads all chunks into memory)
* D8-014: Memory usage bounded ❌ (unbounded on search)

**D9 (Operational):**
* D9-021: Health check endpoint ❌ (no health check in service)

---

**END OF CONSOLIDATED REVIEW**
