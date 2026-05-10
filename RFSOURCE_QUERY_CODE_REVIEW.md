# RFSource Query Code Review
**Domain Audit - Phase 3 Evidence Collection**

## Executive Summary

**Crate:** `crates/rfsource-query`  
**Purpose:** Search and query APIs for agents and frontend  
**LOC:** ~90 lines (single file: lib.rs)  
**Test Coverage:** 1 unit test  
**Critical Issues:** 2 (no SQL engine, misleading crate name)  
**Important Issues:** 2 (no query validation, pagination broken)

**Overall Assessment:** ⚠️ **MODERATE** - Thin orchestration layer over rfsource-index. Name suggests "SQL engine" but it's just search API glue code.

---

## Files Reviewed

### 1. lib.rs (90 lines)
**Purpose:** Query orchestration layer

**Documentation:**
```rust
//! Search and query APIs for agents and frontend.
//!
//! Provides high-level search operations that combine text search,
//! symbol lookup, and grant-scoped filtering.
```

**Crate Law:**
```rust
//! - Query orchestration only — delegates to `rfsource-index` and `rfsource-catalog`
//! - No direct file IO — reads go through `rfsource-store`
```

**⚠️ CRATE NAME MISMATCH:**
- Crate name: `rfsource-query` (suggests SQL query engine)
- Actual purpose: Search API orchestration (text + symbol search wrapper)
- **No SQL parsing, no query planner, no execution engine**

---

## API (1 function, 2 types)

### Types

#### 1. ArtifactQuery
```rust
pub struct ArtifactQuery {
    pub text_search: Option<String>,
    pub symbol_search: Option<String>,
    pub limit: usize,
    pub offset: usize,
}
```

**Purpose:** Query parameters for search

**Findings:**
- ✅ Pagination support (limit + offset)
- ⚠️ No sort order (always unsorted)
- ⚠️ No field filters (path, language, risk_level)
- ⚠️ No date range filters (created_at, updated_at)

#### 2. SearchResults
```rust
pub struct SearchResults {
    pub text_hits: Vec<SearchHit>,
    pub symbol_hits: Vec<String>,
    pub total_count: usize,
}
```

**Purpose:** Combined search results

**Findings:**
- ✅ Total count returned (for pagination UI)
- ⚠️ Symbol hits are just names (no metadata)
- ⚠️ No scoring/ranking info

### Function: search_artifacts()

```rust
pub fn search_artifacts(
    query: &ArtifactQuery,
    chunks: &[SourceChunk],
    symbols: &[SymbolRecord],
    registry: &ArtifactRegistry,
    actor_grant: &str,
) -> SearchResults
```

**What It Does:**
1. **Text Search (optional):**
   - Calls `rfsource_index::search_text()`
   - Filters results by `registry.grant_allows()`
2. **Symbol Search (optional):**
   - Calls `rfsource_index::search_symbol()`
   - Filters by grants
   - Extracts only symbol names (drops metadata)
3. **Pagination:**
   - Counts total results
   - Applies offset + limit to text_hits
   - **⚠️ Does NOT apply pagination to symbol_hits**

**Critical Bug:**
```rust
let total_count = text_hits.len() + symbol_hits.len();
let text_hits = text_hits
    .into_iter()
    .skip(query.offset)
    .take(query.limit)
    .collect();

SearchResults {
    text_hits,
    symbol_hits,  // ← NOT PAGINATED!
    total_count,
}
```

**Impact:**
- Symbol results not paginated → returns ALL symbol matches
- If query matches 10,000 symbols → returns all 10,000
- OOM risk for broad queries

---

## Test Coverage (1 test)

1. ✅ **test_empty_search()** - Empty query returns zero results

**Missing Tests:**
- ❌ Text search with matches
- ❌ Symbol search with matches
- ❌ Grant filtering (authorized vs unauthorized)
- ❌ Pagination (offset + limit behavior)
- ❌ Combined text + symbol search
- ❌ Empty corpus vs empty query

---

## Critical Findings

### CRITICAL-1: Not a SQL Engine ❌

**Issue:** Crate name `rfsource-query` strongly suggests SQL query capabilities, but:

**Missing SQL Features:**
- ❌ No SQL parsing (no `SELECT`, `WHERE`, `JOIN`)
- ❌ No query planner
- ❌ No execution engine
- ❌ No schema introspection
- ❌ No data transformation (GROUP BY, ORDER BY, aggregates)
- ❌ No materialized views integration

**What It Actually Does:**
```rust
// Just calls existing search functions and filters by grants
rfsource_index::search_text(...)
rfsource_index::search_symbol(...)
registry.grant_allows(...)
```

**This is orchestration, not a query engine.**

**Impact:**
- Users expect SQL querying based on crate name
- Developers waste time looking for SQL functionality
- Design mismatch creates confusion

**Recommendation:**
1. **Rename crate to `rfsource-search-api`** (honest name)
2. **OR implement actual SQL:**
   - Use `sqlparser` crate to parse SQL
   - Build execution plan
   - Map tables to RFSource entities (commits, artifacts, chunks)
   - Example queries:
     ```sql
     SELECT logical_path, line_count
     FROM artifacts
     WHERE language = 'Rust' AND risk_level = 'high'
     ORDER BY line_count DESC
     LIMIT 10;
     ```

**Audit Questions Answered:**
- **D1-001: Is crate purpose documented?** ⚠️ MISLEADING - name suggests SQL, docs say "search API"
- **D3-010: Does query engine exist?** ❌ NO

---

### CRITICAL-2: Pagination Bug (Symbol Hits Not Paginated) ❌

**Issue:** `symbol_hits` not paginated, only `text_hits`.

**Evidence:**
```rust
let total_count = text_hits.len() + symbol_hits.len();  // ← Count ALL
let text_hits = text_hits
    .into_iter()
    .skip(query.offset)
    .take(query.limit)
    .collect();  // ← Paginate text_hits

SearchResults {
    text_hits,         // ← Paginated
    symbol_hits,       // ← NOT paginated (all results returned)
    total_count,
}
```

**Failure Scenario:**
```rust
let query = ArtifactQuery {
    text_search: None,
    symbol_search: Some("get".to_string()),  // Common prefix
    limit: 10,  // Request 10 results
    offset: 0,
};

let results = search_artifacts(&query, &chunks, &symbols, &registry, &grant);
// Expect: 10 symbols
// Actual: ALL matching symbols (could be 1000+)
```

**Impact:**
- OOM risk (unbounded results)
- Network timeout (large payload)
- Client UI crash (rendering 1000s of results)

**Recommendation:**
```rust
// Option 1: Paginate symbol_hits separately
let symbol_hits: Vec<String> = if let Some(ref sym_q) = query.symbol_search {
    rfsource_index::search_symbol(sym_q, symbols, &grant_check)
        .into_iter()
        .filter(|s| registry.grant_allows(&s.artifact_id, actor_grant))
        .skip(query.offset)  // ← Add pagination
        .take(query.limit)
        .map(|s| s.symbol_name.clone())
        .collect()
} else {
    vec![]
};

// Option 2: Combined pagination (interleave text + symbol results)
let mut all_results = vec![];
all_results.extend(text_hits.into_iter().map(ResultType::Text));
all_results.extend(symbol_hits.into_iter().map(ResultType::Symbol));
all_results.sort_by_key(|r| r.score());
let page = all_results.into_iter().skip(offset).take(limit).collect();
```

**Audit Questions Answered:**
- **D7-009: Is pagination correct?** ❌ NO - symbol_hits not paginated
- **D8-007: Are unbounded queries prevented?** ❌ NO - symbol results unbounded

---

## Important Findings

### IMPORTANT-1: No Query Validation ⚠️

**Issue:** `ArtifactQuery` has no validation. Accepts invalid inputs.

**Examples:**

#### 1. Negative offset
```rust
let query = ArtifactQuery {
    offset: usize::MAX,  // ❌ Will skip all results
    limit: 10,
};
```

#### 2. Zero limit
```rust
let query = ArtifactQuery {
    limit: 0,  // ❌ No results (valid but likely user error)
    offset: 0,
};
```

#### 3. Huge limit
```rust
let query = ArtifactQuery {
    limit: usize::MAX,  // ❌ OOM risk
    offset: 0,
};
```

#### 4. Both searches None
```rust
let query = ArtifactQuery {
    text_search: None,
    symbol_search: None,  // ❌ No-op query (wastes resources)
    limit: 10,
    offset: 0,
};
```

**Recommendation:**
```rust
impl ArtifactQuery {
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.limit == 0 {
            return Err(QueryError::ZeroLimit);
        }
        if self.limit > 1000 {
            return Err(QueryError::LimitTooLarge(self.limit));
        }
        if self.text_search.is_none() && self.symbol_search.is_none() {
            return Err(QueryError::EmptyQuery);
        }
        Ok(())
    }
}
```

**Audit Questions Answered:**
- **D7-008: Are invalid inputs rejected early?** ❌ NO - no query validation

---

### IMPORTANT-2: No Error Handling ⚠️

**Issue:** `search_artifacts()` returns `SearchResults`, not `Result<SearchResults, QueryError>`.

**Problems:**
1. **No failure reporting:**
   - Index lookup fails → ???
   - Registry fails → ???
   - Grant check fails → ???

2. **Silent failures:**
   - Grants denied → empty results (indistinguishable from no matches)
   - No way to differentiate:
     - "No results found" (valid)
     - "Access denied" (auth error)
     - "Index unavailable" (system error)

**Recommendation:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Index unavailable: {0}")]
    IndexUnavailable(String),
    
    #[error("Access denied to artifact: {0}")]
    AccessDenied(String),
    
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

pub fn search_artifacts(...) -> Result<SearchResults, QueryError> {
    // Can now report failures
}
```

**Audit Questions Answered:**
- **D7-001: Are all errors typed?** ❌ NO - no error enum
- **D7-003: Are errors actionable?** ❌ NO - silent failures

---

## Positive Findings ✅

### 1. Clean Orchestration
- Delegates to specialized crates (index, catalog)
- No IO dependencies (follows crate law)
- Simple API surface (1 function)

### 2. Grant Filtering
- Applies `registry.grant_allows()` after search
- Respects access control
- Actor-scoped results

### 3. Pagination Support (Partial)
- `limit` + `offset` fields exist
- Total count returned
- (But symbol_hits not actually paginated)

### 4. Optional Searches
- Text and symbol searches are both `Option<String>`
- Can search text only, symbols only, or both

---

## Audit Questions Answered

### D1 (Documentation)
- **D1-001: Is crate purpose documented?** ⚠️ MISLEADING - name vs docs mismatch

### D2 (Test Coverage)
- **D2-001: Does every module have tests?** ⚠️ MINIMAL - 1 test (empty query only)
- **D2-002: Are unit tests comprehensive?** ❌ NO - no positive test cases

### D3 (Domain Model)
- **D3-010: Does query engine exist?** ❌ NO - just search orchestration

### D6 (Security & Governance)
- **D6-001: Is access control enforced?** ✅ YES - grant filtering applied

### D7 (Error Handling)
- **D7-001: Are all errors typed?** ❌ NO - no error type
- **D7-003: Are errors actionable?** ❌ NO - silent failures
- **D7-008: Are invalid inputs rejected early?** ❌ NO - no validation
- **D7-009: Is pagination correct?** ❌ NO - symbol_hits bug

### D8 (Performance)
- **D8-007: Are unbounded queries prevented?** ❌ NO - symbol results unbounded

---

## Recommendations by Priority

### CRITICAL (Pre-Production)
1. **Fix pagination bug** - Apply offset/limit to symbol_hits
2. **Add error handling** - Return `Result<SearchResults, QueryError>`
3. **Rename crate** - To `rfsource-search-api` (or implement real SQL)

### IMPORTANT (Pre-Production)
4. **Add query validation** - Check limit, offset, empty query
5. **Add comprehensive tests** - Text search, symbol search, grants, pagination

### NICE-TO-HAVE (Post-MVP)
6. **Add combined pagination** - Interleave text + symbol results
7. **Add field filters** - language, path, risk_level
8. **Add sort options** - by score, date, path

---

## Summary

**rfsource-query** is:
- ✅ Simple orchestration layer
- ✅ Grant-aware
- ✅ Clean API

**BUT:**
- ❌ **Not a SQL engine** (misleading crate name)
- ❌ **Pagination bug** (symbol_hits unbounded)
- ❌ **No error handling** (silent failures)
- ❌ **No query validation** (accepts invalid inputs)
- ❌ **Minimal testing** (1 test)

**Verdict:** ⚠️ **MODERATE RISK** - Thin glue layer with critical bugs. Simple to fix, but name creates false expectations.

**Recommendation:**
1. **Immediate:** Fix pagination bug (5-line change)
2. **Short-term:** Add error handling + validation
3. **Long-term:** Rename crate OR build actual SQL engine

---

## Files Reviewed

1. `lib.rs` (90 lines) - 1 function, 2 types, 1 test
2. `Cargo.toml` - Dependencies

**Total:** ~90 lines of code, 1 unit test

---

## Next Steps

Continue Phase 3 evidence collection with:
- **rfsource-materialize** - Materialized views
- **rfsource-service** - HTTP/gRPC API

Then proceed to Phase 4: Gap Analysis
