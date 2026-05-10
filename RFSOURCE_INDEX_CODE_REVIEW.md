# RFSource Index - Code Review

**Crate:** `crates/rfsource-index`  
**Purpose:** Text and symbol search for `.rfsource` artifacts  
**Review Date:** 2026-05-07  
**Reviewer:** Domain Audit (Phase 3)

---

## Executive Summary

**Status:** ⚠️ **MVP STUB** - Basic search only, missing key features

**Key Findings:**
* ✅ Clean implementation (128 lines, single file)
* ❌ **NO bloom filters** despite crate name suggesting indexing
* ❌ **NO statistics tracking** (no cardinality estimates, no selectivity)
* ❌ **NO persistent index** (in-memory only, scans all chunks every query)
* ⚠️ Basic grant checking (but no actual permission enforcement)
* ✅ 3 tests cover basic scenarios
* ❌ **NOT production-ready** - Linear scan does not scale

**Critical Gap:** This is a **search** implementation masquerading as an **index**. True indexing requires data structures (inverted index, bloom filters, statistics) that enable sub-linear query performance. Current implementation scans ALL chunks for every search.

**Production Readiness:** ❌ NOT READY - Performance will degrade linearly with data size

---

## Files Reviewed

| File | Lines | Purpose |
|------|-------|---------|
| `src/lib.rs` | 128 | Text/symbol search functions |
| **Total** | **128** | |

---

## Implementation Analysis

### What It Claims to Be (from crate name/docs)

**`rfsource-index`** suggests:
* Indexing structures (inverted index, bloom filters)
* Statistics for query optimization
* Persistent index storage
* Sub-linear query performance

### What It Actually Is

**A naive in-memory search** with:
* Linear scan through all chunks
* Case-insensitive substring matching
* No persistent state
* No query optimization
* No statistics

---

## Code Review

### 1. Text Search Function

```rust
pub fn search_text(
    query: &str,
    chunks: &[SourceChunk],
    _symbols: &[SymbolRecord],  // ← NOT USED
    grant_allows: &impl Fn(&str) -> bool,
) -> Vec<SearchHit> {
    let query_lower = query.to_lowercase();
    let mut hits = Vec::new();

    for chunk in chunks {  // ← LINEAR SCAN - O(n)
        if !chunk.text.to_lowercase().contains(&query_lower) {
            continue;
        }
        // Grant check
        if !grant_allows("search") {
            continue;
        }
        
        // Build excerpt
        let excerpt_len = 120.min(chunk.text.len());
        let excerpt = &chunk.text[..excerpt_len];
        
        hits.push(SearchHit { /* ... */ });
    }
    
    hits
}
```

**Issues:**
1. **O(n) complexity** - Scans every chunk, every query
2. **No index structure** - No inverted index, no bloom filter
3. **Symbols parameter ignored** - Why pass if not used?
4. **Grant check hardcoded** - Checks "search" permission but doesn't check artifact-level grants
5. **No ranking** - All results have same score ("text_match")
6. **No pagination** - Returns all matches (unbounded)
7. **Excerpt fixed at 120 chars** - No context around match

**Performance at scale:**
* 100 artifacts × 10 chunks each = 1,000 chunks → 1,000 scans per query
* 10,000 artifacts × 10 chunks = 100,000 chunks → 100,000 scans per query
* **This will NOT scale**

### 2. Symbol Search Function

```rust
pub fn search_symbol<'a>(
    query: &str,
    symbols: &'a [SymbolRecord],
    grant_allows: &impl Fn(&str) -> bool,
) -> Vec<&'a SymbolRecord> {
    let query_lower = query.to_lowercase();
    symbols
        .iter()
        .filter(|s| s.symbol_name.to_lowercase().contains(&query_lower) 
                    && grant_allows("search"))
        .collect()
}
```

**Issues:**
1. **O(n) complexity** - Scans every symbol
2. **No trie or prefix tree** - Symbol names are ideal for trie optimization
3. **Same grant problem** - Hardcoded "search" check
4. **No ranking** - Exact matches not prioritized over substrings
5. **Substring matching on symbol names** - Usually want exact or prefix match

**Better approach for symbols:**
* Trie for prefix matching
* Hash map for exact matches
* Bloom filter for "does not exist" fast path

### 3. Excerpt Function

```rust
pub fn excerpt(text: &str, line_start: u32, line_end: u32) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = (line_start as usize).saturating_sub(1);
    let end = (line_end as usize).min(lines.len());
    lines[start..end].join("\n")
}
```

**Issues:**
1. **Splits entire text** - O(n) where n = lines in text
2. **No caching** - Repeated calls re-split
3. **Off-by-one complexity** - `saturating_sub(1)` suggests 1-indexed lines but unclear

**Positive:** At least handles bounds correctly (no panic)

---

## Missing Index Features

### What a Real Index Should Have

**1. Inverted Index:**
```rust
HashMap<Term, Vec<(ArtifactId, ChunkId, Position)>>
// "hello" -> [(art_1, chk_1, pos 0), (art_2, chk_5, pos 42)]
```

**2. Bloom Filters:**
```rust
// Fast negative lookups
if !bloom_filter.contains(term) {
    return vec![]; // Term definitely not in corpus
}
```

**3. Statistics:**
```rust
struct TermStatistics {
    document_frequency: usize,  // How many docs contain term
    total_occurrences: usize,   // Total occurrences across corpus
    idf: f64,                   // Inverse document frequency for ranking
}
```

**4. Persistent Storage:**
* Index serialized to `.rfsource` frames
* Incremental updates on commit
* Rebuild from scratch if corrupted

**5. Query Optimization:**
* Rank by TF-IDF or BM25
* Boolean queries (AND, OR, NOT)
* Phrase queries ("exact match")
* Proximity queries (within N words)

**Current Implementation:** NONE of the above

---

## Grant Checking Issues

```rust
if !grant_allows("search") {
    continue;
}
```

**Problems:**
1. **Always passes "search"** - Not artifact-specific
2. **Function parameter, not data-driven** - Can't be tested properly
3. **No audit trail** - Denied searches not logged
4. **Inconsistent with governance layer** - Should check `allowed_grants` field from SourceArtifact

**Better approach:**
```rust
// Check artifact-level grants from metadata
let artifact = registry.get_artifact(&chunk.artifact_id)?;
if !user_grants.any(|g| artifact.allowed_grants.contains(g)) {
    audit_log.denied_search(user, artifact_id);
    continue;
}
```

---

## Testing

**Tests Found (3 total):**

```rust
#[test]
fn test_search_text_empty_chunks() {
    let hits = search_text("hello", &[], &[], &grant_allows);
    assert!(hits.is_empty());
}

#[test]
fn test_search_text_finds_match() {
    // Creates 1 chunk, searches for "hello", expects 1 hit
}

#[test]
fn test_search_symbol_finds_match() {
    // Creates 1 symbol, searches for "MyStruct", expects 1 result
}
```

**Coverage:** Basic happy path only

**Missing Tests:**
* Performance tests (10K, 100K chunks)
* Case-insensitive matching edge cases
* Unicode handling (emoji, accents, non-ASCII)
* Grant denial scenarios
* Pagination (currently unbounded results)
* Ranking verification
* Excerpt boundary conditions

---

## Dependencies

```toml
[dependencies]
rfsource-core.workspace = true
serde.workspace = true
thiserror.workspace = true
tracing.workspace = true
```

**Analysis:**
* ✅ No external index libraries (e.g., tantivy) - Keeps it simple
* ⚠️ `thiserror` imported but no error types defined
* ⚠️ `tracing` imported but no trace points
* ⚠️ `serde` imported but no custom serialization

**Unused dependencies?** Possibly - error types would make sense but aren't present

---

## Mapping to Audit Questions

### D3 (Scalability)

**D3-008: Tested with 1GB files?**
* ❌ **Gap** - Linear scan will fail at scale

**D3-011: Performance degradation with file size?**
* ❌ **Gap** - O(n) means linear degradation, unacceptable

**D3-012: Streaming mechanisms?**
* ❌ **Gap** - Loads all chunks into memory

### D8 (Performance)

**D8-001: p50/p95/p99 latency?**
* ❌ **Gap** - No benchmarks

**D8-006: Max reads per second?**
* ❌ **Gap** - Unbounded results means query time is unbounded

**D8-008: Bottlenecks identified?**
* ⚠️ **Partial** - Linear scan is obvious bottleneck, but not measured

**D8-022: Can operations be parallelized?**
* ⚠️ **Partial** - Search could be parallelized across chunks, but not implemented

### D2 (Test Coverage)

**D2-015: Tested at 10x load?**
* ❌ **Gap** - No load tests

**D2-003: All public functions tested?**
* ⚠️ **Partial** - Basic tests exist but not comprehensive

---

## Critical Issues

### 1. Linear Scan Performance (CRITICAL)

**Issue:** O(n) search means query time grows linearly with data size.

**Evidence:**
```rust
for chunk in chunks {  // Scans EVERY chunk
    if !chunk.text.to_lowercase().contains(&query_lower) {
```

**Impact:** At 100K chunks:
* Assuming 1KB per chunk = 100MB to scan per query
* With case-insensitive comparison = significant CPU
* With grant checks = additional overhead
* **Estimated query time:** 100-500ms for small corpus, 1-10s for large

**Recommendation:** Implement inverted index (1-2 weeks effort)

### 2. No Persistent State (IMPORTANT)

**Issue:** Index must be rebuilt from scratch on every startup.

**Evidence:** No serialization, no frame storage, pure in-memory

**Impact:**
* Slow startup (must scan all chunks to build in-memory state)
* No cross-session optimization
* Cannot pre-build index offline

**Recommendation:** Serialize index to `.rfsource` frames (3 days effort)

### 3. Grant Checking Broken (IMPORTANT)

**Issue:** Grant checks don't actually check artifact grants.

**Evidence:**
```rust
if !grant_allows("search") {  // Always "search", not artifact-specific
```

**Impact:** All users with "search" grant can see ALL artifacts (no artifact-level access control)

**Recommendation:** Integrate with artifact registry to check `allowed_grants` (2 days effort)

### 4. Missing Bloom Filters (IMPORTANT)

**Issue:** Despite crate name suggesting indexing, no bloom filters.

**Evidence:** No bloom filter imports, no probabilistic data structures

**Impact:** Cannot quickly eliminate non-matching queries

**Recommendation:** Add bloom filter per artifact (1 week effort)

---

## Recommendations

### Priority 1: Acknowledge MVP Status
* Update crate docs to say "MVP: linear scan, no persistent index"
* Document performance limitations (not suitable for >10K chunks)
* Note that this is a placeholder for future index implementation

### Priority 2: Add Performance Tests
* Benchmark with 1K, 10K, 100K chunks
* Measure latency (p50/p95/p99)
* Document breaking points
* Add test that fails if query time >1s for 10K chunks

### Priority 3: Implement True Index (Post-MVP)
* **Inverted index** for text search (2 weeks)
* **Trie** for symbol prefix matching (1 week)
* **Bloom filters** for fast negatives (3 days)
* **Persistent storage** in `.rfsource` frames (3 days)
* **Incremental updates** on commit (1 week)

### Priority 4: Fix Grant Checking
* Integrate with `rfsource-catalog` to get artifact metadata
* Check `allowed_grants` field per artifact
* Add audit logging for denied searches
* Write tests for grant scenarios

### Priority 5: Add Query Features
* **Pagination:** Limit + offset for results
* **Ranking:** TF-IDF or BM25 for relevance
* **Boolean queries:** AND, OR, NOT operators
* **Phrase queries:** Exact multi-word matches

---

## Positive Findings

1. ✅ **Clean code** - Simple, readable, no magic
2. ✅ **Type safe** - Uses rfsource-core types correctly
3. ✅ **Basic tests** - 3 tests cover happy paths
4. ✅ **No unwrap()** - No panics in production code
5. ✅ **Honest** - Doesn't claim features it doesn't have (unlike name)

---

## Production Readiness: ❌ NOT READY

**Blockers:**
* Linear scan performance (unacceptable at scale)
* No persistent state (slow startup)
* Grant checking broken (security risk)
* No load testing (performance unknowns)

**Estimated Effort to Production:**
* MVP acknowledgment: 1 day
* Performance tests: 2 days
* True index implementation: 4-6 weeks
* Grant fixing: 2 days
* **Total:** 5-7 weeks

---

## Questions Answered (8 total)

**D2 (Test Coverage): 1 partial**
* D2-003: Public functions tested ⚠️ (basic only)

**D3 (Scalability): 3 gaps**
* D3-008: Tested with 1GB files ❌
* D3-011: Performance degradation ❌
* D3-012: Streaming mechanisms ❌

**D8 (Performance): 4 gaps**
* D8-001: Latency benchmarks ❌
* D8-006: Max reads per second ❌
* D8-008: Bottlenecks identified ⚠️ (obvious but unmeasured)
* D8-022: Parallelization ⚠️ (possible but not implemented)

---

**END OF RFSOURCE-INDEX CODE REVIEW**
