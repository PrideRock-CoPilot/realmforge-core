# RFSource Materialize Code Review
**Domain Audit - Phase 3 Evidence Collection**

## Executive Summary

**Crate:** `crates/rfsource-materialize`  
**Purpose:** Materialize logical source artifacts from `.rfsource` to filesystem  
**LOC:** ~170 lines (single file: lib.rs)  
**Test Coverage:** 1 integration test  
**Critical Issues:** 1 (branch materialization not implemented)  
**Important Issues:** 3 (no atomic writes, no file permissions, no incremental updates)

**Overall Assessment:** ⚠️ **MODERATE** - Simple filesystem projection with grant filtering. Works for MVP but lacks atomicity and optimization.

---

## Files Reviewed

### 1. lib.rs (170 lines)
**Purpose:** Filesystem materialization

**Documentation:**
```rust
//! Reads the current state from a `.rfsource` file and writes the rendered
//! source files to the target directory. This is how `.rs`, `.ts`, `.py`,
//! `.md` and other source files are produced from the canonical `.rfsource`
//! storage.
```

**Crate Law:**
```rust
//! - Filesystem projection only — no business logic
//! - Output files are "rendered projections", not truth
//! - Grant-scoped: only materialize artifacts the caller has access to
```

**Key Principle:** `.rfsource` is canonical source of truth; filesystem files are derived projections.

---

## API (2 functions, 1 error type)

### 1. materialize()
```rust
pub fn materialize(
    rfsource: &RFSource,
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize, MaterializeError>
```

**What It Does:**
1. Get all current artifacts from store
2. Get all chunks from store
3. Group chunks by version_id
4. For each artifact:
   - Check grant access
   - Reconstruct content from chunks (sorted by ordinal)
   - Write to `output_dir/logical_path`
   - Handle deleted artifacts (remove file if exists)
5. Return count of materialized files

**Performance:**
- Materializes ALL artifacts (no filtering by path)
- Reads ALL chunks into memory
- No incremental updates (full rebuild every time)

**Example:**
```rust
let store = RFSource::open(".rfsource")?;
let count = materialize(&store, "./src", Some("SGL-BACKEND-READ"))?;
println!("Materialized {} files", count);
```

**Findings:**
- ✅ Grant filtering (respects `allowed_grants`)
- ✅ Creates parent directories
- ✅ Handles deleted artifacts (removes files)
- ⚠️ Non-atomic writes (crash mid-materialize → partial state)
- ⚠️ No file permissions/metadata preserved
- ⚠️ No incremental updates (rebuilds everything)

### 2. materialize_branch()
```rust
pub fn materialize_branch(
    rfsource: &RFSource,
    _branch: &str,  // ← Unused!
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize, MaterializeError>
```

**CRITICAL FINDING:** This function **does NOT implement branch-specific materialization**. It's just a wrapper around `materialize()` that ignores the `branch` parameter!

**Evidence:**
```rust
pub fn materialize_branch(..., _branch: &str, ...) {
    // For now, materialize works the same regardless of branch
    // since the store's current_artifacts() reflects the main branch state.
    // Branch-specific materialization will be a future enhancement.
    materialize(rfsource, output_dir, grant)
}
```

**Impact:**
- Cannot materialize feature branches
- Cannot compare branch states
- Cannot test code from non-main branches
- Function signature promises capability it doesn't deliver

### 3. MaterializeError (2 variants)
```rust
#[derive(Debug, thiserror::Error)]
pub enum MaterializeError {
    Io(#[from] std::io::Error),
    Store(String),
}
```

**Findings:**
- ✅ thiserror for error messages
- ✅ Wraps std::io::Error
- ⚠️ Store error is generic String (not typed)

---

## Implementation Details

### Content Reconstruction
```rust
let mut content = String::new();
let mut chunks_sorted = chunks.clone();
chunks_sorted.sort_by_key(|c| c.ordinal);  // ← Sort by chunk order
for chunk in chunks_sorted {
    content.push_str(&chunk.text);
    content.push('\n');  // ← Add newline between chunks
}
fs::write(&file_path, content.trim())?;  // ← Write to file
```

**Process:**
1. Clone chunks (allocates)
2. Sort by ordinal (deterministic order)
3. Concatenate with newlines
4. Trim trailing whitespace
5. Write atomically (at OS level)

**Issues:**
- ⚠️ `trim()` may alter file content (strips trailing newlines)
- ⚠️ Adds newline between chunks (may create extra blank lines)
- ⚠️ Clones chunks unnecessarily (could use indices)

### Grant Filtering
```rust
if let Some(g) = grant {
    if !artifact.allowed_grants.is_empty()
        && !artifact.allowed_grants.contains(&g.to_string())
        && !artifact.allowed_grants.iter().any(|g| g == \"*\")
    {
        continue;  // ← Skip artifact (no access)
    }
}
```

**Logic:**
1. If no grant specified → materialize all
2. If artifact has no grant requirements → materialize (public)
3. If artifact requires grants:
   - Check exact match
   - Check wildcard "*"
   - Else skip

**Issues:**
- ✅ Correct grant logic
- ⚠️ No audit trail (doesn't log skipped artifacts)

### Deleted Artifact Handling
```rust
if artifact.deleted {
    // Skip deleted artifacts (or remove if they exist)
    let _ = fs::remove_file(&file_path);  // ← Ignore errors
    continue;
}
```

**Findings:**
- ✅ Removes deleted files from filesystem
- ⚠️ Silently ignores remove errors (file may not exist yet)
- ⚠️ No confirmation logged (user doesn't know what was removed)

---

## Test Coverage (1 test)

1. ✅ **test_materialize_single_file()** - Creates store, commits file, materializes, checks file exists

**Good Coverage:**
- File creation
- Directory creation (src/ subdirectory)
- Content verification (file exists)

**Missing Tests:**
- ❌ Grant filtering (authorized vs unauthorized)
- ❌ Deleted artifact removal
- ❌ Multi-file materialization
- ❌ Nested directories (src/foo/bar/baz.rs)
- ❌ Path traversal prevention (../../../etc/passwd)
- ❌ Branch materialization (feature vs main)
- ❌ Overwrite behavior (materialize twice)
- ❌ Chunk ordering (ordinal correctness)

---

## Critical Findings

### CRITICAL-1: Branch Materialization Not Implemented ❌

**Issue:** `materialize_branch()` ignores the `branch` parameter and always materializes `main`.

**Evidence:**
```rust
pub fn materialize_branch(
    rfsource: &RFSource,
    _branch: &str,  // ← Underscore prefix means "intentionally unused"
    ...
) -> Result<usize, MaterializeError> {
    // Comment admits it's not implemented:
    // "For now, materialize works the same regardless of branch"
    materialize(rfsource, output_dir, grant)  // ← Just calls materialize()
}
```

**Impact:**
- Cannot materialize feature branches
- Cannot test code from non-main branches
- Cannot compare branch states on filesystem
- Function API is misleading (promises capability it lacks)

**Use Cases Broken:**
- Developer: "Materialize my `feature/new-api` branch to test it"
- CI/CD: "Build and test branch X before merging"
- Code review: "Show me the filesystem diff between main and PR branch"

**Remediation:**
```rust
pub fn materialize_branch(
    rfsource: &RFSource,
    branch: &str,
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize, MaterializeError> {
    // 1. Get branch head commit
    let branch_state = rfsource.get_branch(branch)?;
    let head_commit_id = branch_state.head_commit_id
        .ok_or_else(|| MaterializeError::Store("Branch has no head commit".into()))?;
    
    // 2. Get artifacts at that commit
    let artifacts = rfsource.artifacts_at_commit(&head_commit_id)?;
    
    // 3. Get chunks for those artifact versions
    let chunks = rfsource.chunks_at_commit(&head_commit_id)?;
    
    // 4. Materialize (rest of logic same)
    // ...
}
```

**Audit Questions Answered:**
- **D4-001: Can you materialize any branch?** ❌ NO - only main branch
- **D4-002: Does branch switching work?** ❌ NO - not implemented

---

## Important Findings

### IMPORTANT-1: Non-Atomic Materialization ⚠️

**Issue:** Files written one-by-one. Crash mid-materialize → partial state.

**Failure Scenario:**
```
materialize() starts
  ✅ Write src/main.rs
  ✅ Write src/lib.rs
  ✅ Write src/foo/mod.rs
  💥 CRASH (OOM, SIGKILL, power loss)
  ❌ src/bar/baz.rs NOT written
  ❌ src/qux/quux.rs NOT written
Result: Partial checkout (inconsistent state)
```

**Impact:**
- User sees half-updated codebase
- CI/CD builds may fail on partial state
- No rollback mechanism
- No way to detect partial materialization

**Recommendation:**
```rust
pub fn materialize_atomic(
    rfsource: &RFSource,
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize, MaterializeError> {
    let tmp_dir = output_dir.as_ref().with_extension("tmp");
    
    // 1. Materialize to temp directory
    let count = materialize(rfsource, &tmp_dir, grant)?;
    
    // 2. Atomically rename (on same filesystem)
    fs::rename(&tmp_dir, output_dir)?;
    
    Ok(count)
}
```

**Trade-off:** 2x disk space during materialization, but atomic cutover.

**Audit Questions Answered:**
- **D7-011: Are writes atomic?** ❌ NO - file-by-file writes
- **D7-013: Is partial state detectable?** ❌ NO

---

### IMPORTANT-2: No File Permissions/Metadata ⚠️

**Issue:** Materialized files have default permissions (typically 644). Executable scripts lose +x bit.

**Problem:**
```rust
// In .rfsource:
artifact: "scripts/deploy.sh"
metadata: { executable: true }

// After materialize:
ls -l out/scripts/deploy.sh
-rw-r--r-- 1 user staff ... deploy.sh  // ← Not executable!
```

**Impact:**
- Shell scripts not executable (user must `chmod +x` manually)
- No ownership preservation (all files owned by materializing process)
- No timestamp preservation (all files have materialize time)

**Recommendation:**
```rust
// 1. Add executable flag to ArtifactEntry
pub struct ArtifactEntry {
    // ...
    pub executable: bool,
}

// 2. Set permissions during materialize
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    if artifact.executable {
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&file_path, perms)?;
    }
}
```

**Audit Questions Answered:**
- **D3-011: Are file permissions preserved?** ❌ NO

---

### IMPORTANT-3: No Incremental Updates ⚠️

**Issue:** Every materialize is a full rebuild. No detection of unchanged files.

**Performance:**
```
Repo: 1000 files
Change: Edit 1 file
materialize():
  - Reads 1000 artifacts
  - Reads 10,000 chunks
  - Writes 1000 files (all of them)
  - Time: ~1s

Incremental materialize (ideal):
  - Detect 1 file changed
  - Write 1 file
  - Time: ~1ms
```

**Impact:**
- 1000x slower than necessary for small changes
- Wears out SSDs (unnecessary writes)
- Wastes CPU cycles

**Recommendation:**
```rust
pub fn materialize_incremental(
    rfsource: &RFSource,
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize, MaterializeError> {
    let manifest_path = output_dir.as_ref().join(".rfsource.manifest");
    let last_state: Option<BTreeMap<String, String>> = 
        fs::read_to_string(&manifest_path).ok()
            .and_then(|s| serde_json::from_str(&s).ok());
    
    let artifacts = rfsource.current_artifacts()?;
    let mut new_state = BTreeMap::new();
    let mut count = 0;
    
    for (art_id, artifact) in &artifacts {
        let content_hash = compute_content_hash(artifact, chunks)?;
        new_state.insert(art_id.clone(), content_hash.clone());
        
        // Skip if unchanged
        if last_state.as_ref().and_then(|s| s.get(art_id)) == Some(&content_hash) {
            continue;
        }
        
        // Materialize changed file
        write_artifact(&file_path, artifact, chunks)?;
        count += 1;
    }
    
    // Save new manifest
    fs::write(&manifest_path, serde_json::to_string(&new_state)?)?;
    Ok(count)
}
```

**Audit Questions Answered:**
- **D8-009: Are updates incremental?** ❌ NO - full rebuild

---

## Positive Findings ✅

### 1. Grant Filtering
- Respects `allowed_grants`
- Supports wildcard "*"
- Skips unauthorized artifacts

### 2. Deleted Artifact Handling
- Removes files for deleted artifacts
- Keeps filesystem in sync with `.rfsource` state

### 3. Directory Creation
- Automatically creates parent directories
- Handles nested paths (src/foo/bar/baz.rs)

### 4. Clean Error Handling
- Returns `Result` (not panic)
- Wraps IO errors
- Error types use thiserror

### 5. Simple Implementation
- 170 lines (easy to understand)
- No complex logic
- Follows crate law (filesystem projection only)

---

## Audit Questions Answered

### D1 (Documentation)
- **D1-001: Is crate purpose documented?** ✅ YES - clear purpose

### D2 (Test Coverage)
- **D2-001: Does every module have tests?** ⚠️ MINIMAL - 1 test
- **D2-016: Are edge cases tested?** ❌ NO - grants, deletion, branches not tested

### D3 (Domain Model)
- **D3-011: Are file permissions preserved?** ❌ NO

### D4 (Versioning & Time Travel)
- **D4-001: Can you materialize any branch?** ❌ NO - only main
- **D4-002: Does branch switching work?** ❌ NO - not implemented

### D7 (Error Handling)
- **D7-011: Are writes atomic?** ❌ NO - file-by-file
- **D7-013: Is partial state detectable?** ❌ NO

### D8 (Performance)
- **D8-009: Are updates incremental?** ❌ NO - full rebuild every time

---

## Recommendations by Priority

### CRITICAL (Pre-Production)
1. **Implement branch materialization** - Make `materialize_branch()` work
2. **Add atomic writes** - Materialize to temp dir, atomic rename

### IMPORTANT (Post-MVP)
3. **Add file permissions** - Preserve executable bit
4. **Add incremental updates** - Skip unchanged files
5. **Add comprehensive tests** - Grants, deletion, branches, permissions

### NICE-TO-HAVE (Future)
6. **Add progress reporting** - Show which files being materialized
7. **Add dry-run mode** - Preview what would be materialized
8. **Add selective materialization** - Materialize specific paths only

---

## Summary

**rfsource-materialize** is:
- ✅ Simple filesystem projection
- ✅ Grant-aware
- ✅ Handles deleted artifacts
- ✅ Creates directories

**BUT:**
- ❌ **Branch materialization not implemented** (critical gap)
- ⚠️ **Non-atomic writes** (crash → partial state)
- ⚠️ **No file permissions** (executables lose +x)
- ⚠️ **No incremental updates** (full rebuild every time)
- ⚠️ **Minimal testing** (1 test)

**Verdict:** ⚠️ **MODERATE RISK** - Works for basic MVP use cases (single branch, small repos), but needs atomicity and branch support for production.

**Recommendation:**
1. **Immediate:** Fix `materialize_branch()` to actually use the branch parameter
2. **Short-term:** Add atomic writes (temp dir + rename)
3. **Long-term:** Add incremental updates for performance

---

## Files Reviewed

1. `lib.rs` (170 lines) - 2 functions, 1 error type, 1 test
2. `Cargo.toml` - Dependencies

**Total:** ~170 lines of code, 1 integration test

---

## Next Steps

Complete Phase 3 evidence collection with:
- **rfsource-service** - HTTP/gRPC API (final crate)

Then proceed to Phase 4: Gap Analysis
