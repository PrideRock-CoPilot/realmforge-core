# RFSource Governance Code Review - Evidence Collection

**Date:** 2026-05-07  
**Reviewer:** AI Agent (Domain Audit Skill)  
**Scope:** `crates/rfsource-governance` crate  
**Purpose:** Phase 3 evidence collection for RFSource audit (210 questions)

---

## Executive Summary

**CRITICAL SECURITY FINDING:** Governance checks are purely **client-side** with **NO enforcement at storage layer**. Anyone with direct file access can bypass all policies.

**Status:**
* ⚠️ **Client-side only** - No server-side enforcement
* ❌ **Bypassable** - Direct file writes skip governance entirely
* ❌ **No audit trail** - Findings not logged or tracked
* ⚠️ **Hardcoded rules** - Not data-driven despite claims
* ⚠️ **Limited scope** - Only 8 basic rules (300/500 line limits, naming conventions)
* ✅ **Simple implementation** - 330 lines, easy to understand
* ✅ **Pure logic** - No I/O, no dependencies, deterministic

---

## Critical Security Findings

### 🔴 CRITICAL: Governance Can Be Bypassed

**Finding:** Governance checks are **OPTIONAL** and can be completely bypassed.

**Evidence:**

**1. Store layer CALLS governance, but doesn't ENFORCE it at format layer:**

From `rfsource-store/src/rf_source.rs`:
```rust
pub fn commit_artifact_on_branch(&self, ...) -> Result<CommitOutcome> {
    // ... grant check ...
    
    // Run governance checks
    let findings = run_checks(&req.logical_path, &req.content, &object_ref);
    let blocking_findings: Vec<_> = findings.iter().filter(|f| f.blocking).collect();
    if !blocking_findings.is_empty() {
        return Err(StoreError::Governance(format!(
            "Blocking governance findings for '{}': {:?}",
            req.logical_path,
            blocking_findings
        )));
    }
    
    // Write the bundle frames (mode-aware)
    self.append_frames_internal(&[serde_json::to_value(&bundle)?])?;
    // ...
}
```

**2. Format layer has NO governance enforcement:**

From `rfsource-format/src/frame.rs`:
```rust
/// Append a single serializable frame to the `.rfsource` file.
pub fn append_frame<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
    // NO governance checks here
    // NO policy enforcement
    // Anyone can call this directly
    
    let json = serde_json::to_vec(value)?;
    // ... write frame ...
    f.sync_all()?;
    Ok(())
}
```

**3. Governance crate is pure function, no enforcement:**

From `rfsource-governance/src/lib.rs`:
```rust
/// Validate source content against governance rules.
///
/// Returns a list of findings. If any finding has `blocking: true`,
/// the artifact should not be committed.
pub fn run_checks(logical_path: &str, content: &str, object_ref: &str) -> Vec<CheckFinding> {
    let mut findings = Vec::new();
    // ... validation logic ...
    findings  // ← RETURNS findings, doesn't BLOCK anything
}
```

**Impact:**

1. **Direct File Access Bypass:**
   ```rust
   // Malicious or accidental bypass:
   use rfsource_format::append_frame;
   
   // Skips ALL governance checks - directly writes to file
   append_frame("/path/to/.rfsource", &malicious_bundle)?;
   ```

2. **No Enforcement Mechanism:**
   * Format layer doesn't know about governance
   * No middleware layer to enforce checks
   * Callers can choose to ignore findings
   * No server-side validation

3. **Trust Boundary Violation:**
   * Client (store layer) is trusted to enforce policy
   * No verification that checks were run
   * No cryptographic proof of compliance
   * Format layer trusts all callers

**Comparison:**

```
❌ Current (client-side):
  User → Store (checks policy) → Format (writes)
         ↑ Can be bypassed by calling Format directly

✅ Should be (server-side):
  User → Store → Format (ENFORCES policy) → Disk
                         ↑ Mandatory gate, cannot bypass
```

---

## Detailed Findings by Audit Dimension

### D6: Security & Governance

 Question ID | Question | Status | Evidence |
------------|----------|--------|----------|
 D6-001 | Are permissions checked at every layer? | ❌ FAIL | Only checked at store layer, not format layer |
 D6-002 | Can permissions be bypassed? | ❌ CRITICAL | YES - direct file writes bypass governance |
 D6-003 | Are there role-based access controls? | ❌ FAIL | No RBAC, only grant-based checks at store layer |
 D6-004 | Are permissions audited? | ⚠️ PARTIAL | Actor logged in commits, but findings NOT audited |
 D6-007 | Are all operations logged? | ⚠️ PARTIAL | Commits logged, but governance findings NOT logged |
 D6-009 | Are logs immutable? | ✅ PASS | Commit log is append-only |
 D6-012 | Is data encrypted at rest? | ❓ UNKNOWN | Not visible at governance layer |

**Governance Rules (8 total):**

```rust
pub fn rule_definitions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("PATH-001", "Logical path must not contain '..' or '//'"),
        ("PATH-002", "Rust files should use snake_case naming"),
        ("SIZE-001", "Hard line limit: 500 lines (blocking)"),
        ("SIZE-002", "Target line limit: 300 lines (warning)"),
        ("HEADER-001", "File should start with a descriptive header"),
        ("DOC-001", "Public items should have doc comments"),
        ("RUST-001", "unwrap() requires INVARIANT comment"),
        ("RUST-002", "unsafe requires SAFETY: comment"),
    ]
}
```

**Blocking vs. Non-Blocking:**

* **Blocking (prevents commit):**
  * PATH-001 - Path traversal (`..*`, `//`)
  * SIZE-001 - File >500 lines
  * RUST-002 - `unsafe` without `SAFETY:` comment

* **Non-Blocking (warning only):**
  * PATH-002 - Non-snake_case naming
  * SIZE-002 - File 300-500 lines
  * HEADER-001 - Missing file header
  * DOC-001 - Missing doc comments on public items
  * RUST-001 - `unwrap()` without `INVARIANT:` comment

**CRITICAL ISSUE:** All rules can be bypassed by writing directly to format layer.

### D1: Documentation Completeness

 Question ID | Question | Status | Evidence |
------------|----------|--------|----------|
 D1-020 | Are security boundaries documented? | ❌ GAP | No documentation that governance is client-side only |
 D1-021 | Is threat model documented? | ❌ GAP | No threat model, no discussion of bypass scenarios |

**Crate Law (from lib.rs):**
```rust
//! ## Crate Law
//!
//! - Pure validation logic — no file IO, no database
//! - Rules are data-defined (extensible without code changes)
//! - Validation is deterministic
```

**INCONSISTENCY:** Claims "rules are data-defined" but they're hardcoded in Rust:
```rust
// SIZE-001: Hard cap of 500 lines
if line_count > 500 {
    findings.push(CheckFinding {
        // ... hardcoded rule ...
    });
}
```

**MISSING DOCUMENTATION:**
* ❌ No documentation that checks are client-side only
* ❌ No guidance on when/where to call `run_checks()`
* ❌ No discussion of bypass scenarios
* ❌ No threat model for malicious actors
* ❌ No explanation of why format layer doesn't enforce

### D2: Test Coverage

 Question ID | Question | Status | Evidence |
------------|----------|--------|----------|
 D2-004 | Are error paths tested? | ✅ PASS | 5 tests for various rule violations |
 D2-024 | Are bypass scenarios tested? | ❌ CRITICAL GAP | NO tests verify governance cannot be bypassed |
 D2-025 | Are malicious inputs tested? | ⚠️ PARTIAL | Path traversal tested, but limited |

**Test Files Found (in lib.rs):**
* `test_empty_file_passes` - Empty file has no blocking issues
* `test_rust_file_without_header` - Flags missing header
* `test_snake_case_check` - Flags non-snake_case names
* `test_path_traversal_check` - Flags `..` in paths
* `test_size_cap` - Flags files >500 lines

**Missing Tests:**
* ❌ **Bypass tests** - Verify that direct format layer calls bypass governance
* ❌ **Enforcement tests** - Verify that store layer MUST call governance
* ❌ **Audit trail tests** - Verify findings are logged
* ❌ **Policy violation recovery** - What happens after blocking finding?
* ❌ **Malicious payload tests** - SQL injection, path traversal, code injection
* ❌ **Performance tests** - Governance overhead on large files

### D7: Error Handling & Recovery

 Question ID | Question | Status | Evidence |
------------|----------|--------|----------|
 D7-013 | Are malicious inputs handled safely? | ⚠️ PARTIAL | Some checks (path traversal) but limited |
 D7-014 | Is input validation comprehensive? | ❌ GAP | Only 8 rules, many attack vectors unchecked |

**Input Validation Coverage:**

**✅ Checked:**
* Path traversal (`..`, `//`)
* File size (300/500 line limits)
* Rust naming conventions (snake_case)
* Rust safety (unsafe, unwrap)
* Documentation presence

**❌ NOT Checked:**
* **Code injection** - No validation of actual Rust syntax
* **SQL injection** - If content includes SQL
* **Command injection** - If content includes shell commands
* **Buffer overflow** - No checks on very long lines (only line COUNT)
* **Unicode attacks** - No normalization or sanitization
* **Resource exhaustion** - Can commit 499 lines of crypto mining code
* **Dependency injection** - No Cargo.toml validation
* **Malicious imports** - No check of `use` statements

### D8: Performance & Resource Usage

 Question ID | Question | Status | Evidence |
------------|----------|--------|----------|
 D8-010 | Are operations bounded in time? | ⚠️ PARTIAL | O(n) in lines, but no timeout |
 D8-011 | Are operations bounded in memory? | ⚠️ PARTIAL | Splits into lines, but no limit |

**Performance Characteristics:**

```rust
pub fn run_checks(logical_path: &str, content: &str, object_ref: &str) -> Vec<CheckFinding> {
    let lines: Vec<&str> = content.lines().collect();  // ← O(n) memory
    let line_count = lines.len();                      // ← O(1)
    
    // Multiple passes over lines:
    for (i, line) in lines.iter().enumerate() {        // ← O(n) for each rule
        // Check unwrap, unsafe, doc comments, etc.
    }
    
    findings
}
```

**Complexity:**
* **Time:** O(n * r) where n = lines, r = rules that iterate over lines (~3)
* **Memory:** O(n) - stores entire file as Vec<&str>
* **No timeouts** - Can run indefinitely on malformed input
* **No memory limits** - Can allocate unbounded memory for huge files

**Example Attack:**
```rust
// 1 million lines, each 1000 chars = 1 GB of content
let malicious_content = "x".repeat(1000) + "\n";
let big_content = malicious_content.repeat(1_000_000);

// This will allocate ~1GB+ in memory
run_checks("src/huge.rs", &big_content, "obj");
// ← No timeout, no memory limit, no streaming
```

### D10: Standardization & Repo Structure

 Question ID | Question | Status | Evidence |
------------|----------|--------|----------|
 D10-001 | Is there a standard directory layout? | ✅ PASS | Single file, follows Cargo conventions |
 D10-006 | Does code pass linting? | ❓ UNKNOWN | cargo clippy not available |

**Crate Structure:**
```
rfsource-governance/
├── Cargo.toml
└── src/
    └── lib.rs    # 330 lines - all code in one file
```

**Simplicity is good here** - governance logic is simple enough to fit in one file.

---

## Architecture Analysis

### Current Design (Client-Side Trust Model)

```
┌─────────────────────────────────────────────────┐
│                 User / Caller                   │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│         rfsource-store (TRUSTED CLIENT)         │
│  - Calls run_checks() before commit             │
│  - Checks if blocking findings exist            │
│  - Rejects commit if blocking findings          │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│       rfsource-governance (PURE FUNCTION)       │
│  - Returns Vec<CheckFinding>                    │
│  - NO enforcement                               │
│  - NO audit logging                             │
└────────────────┬────────────────────────────────┘
                 │
                 ▼ (Caller decides what to do)
┌─────────────────────────────────────────────────┐
│         rfsource-format (DUMB WRITER)           │
│  - Accepts ANY serializable data                │
│  - NO governance checks                         │
│  - NO policy enforcement                        │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
              [ DISK ]
```

**Problems:**

1. **Trust Boundary:** Store layer is trusted to enforce policy
2. **No Verification:** Format layer doesn't verify governance was run
3. **Bypassable:** Direct calls to format layer skip governance
4. **No Audit:** Findings are not logged, cannot detect bypass attempts
5. **Single Point of Failure:** If store layer has bug, governance fails

### Recommended Design (Server-Side Enforcement)

```
┌─────────────────────────────────────────────────┐
│                 User / Caller                   │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│              rfsource-store                     │
│  - Prepares commit request                      │
│  - Passes to enforcement layer                  │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│       rfsource-enforcement (NEW LAYER)          │
│  - MANDATORY gate before format layer           │
│  - Runs governance checks                       │
│  - Logs ALL findings to audit trail            │
│  - Rejects writes if blocking findings          │
│  - Signs approved writes with proof             │
└────────────────┬────────────────────────────────┘
                 │
                 ▼ (Only approved writes pass)
┌─────────────────────────────────────────────────┐
│       rfsource-format (ENFORCED WRITER)         │
│  - Verifies approval signature                  │
│  - Rejects unsigned writes                      │
│  - Writes only approved data                    │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
              [ DISK ]

AUDIT LOG (parallel, immutable):
  - All governance findings
  - All approval/rejection decisions
  - All bypass attempts
  - Cryptographically signed
```

**Benefits:**

1. **Mandatory Enforcement:** Format layer won't accept unapproved writes
2. **Complete Audit Trail:** All decisions logged immutably
3. **Bypass Detection:** Attempts to call format directly are caught
4. **Cryptographic Proof:** Approved writes carry proof of governance
5. **Defense in Depth:** Multiple layers enforce policy

---

## Bypass Scenarios

### Scenario 1: Direct Format Layer Access

**Attack:**
```rust
// Attacker with repo access:
use rfsource_format::append_frame;

// Create malicious bundle with >500 lines
let malicious_bundle = CommitBundle {
    // ... 1000 lines of malicious code ...
};

// Write directly to format layer - bypasses ALL governance
append_frame("/path/.rfsource", &malicious_bundle)?;
```

**Impact:** Governance completely bypassed, malicious code committed

### Scenario 2: Store Layer Bug

**Attack:**
```rust
// Bug in store layer (hypothetical):
pub fn commit_artifact_on_branch(&self, ...) -> Result<CommitOutcome> {
    // BUG: Forgets to check blocking findings
    let findings = run_checks(...);  // ← Returns findings
    // Missing: if !blocking_findings.is_empty() { return Err }
    
    self.append_frames_internal(...)?;  // ← Writes anyway
}
```

**Impact:** Store layer bug defeats governance

### Scenario 3: Custom Client

**Attack:**
```rust
// Attacker writes custom client:
mod custom_client {
    use rfsource_format::append_frame;
    
    pub fn commit_without_governance(path: &str, bundle: &Bundle) {
        // Skip governance entirely
        append_frame(path, bundle).expect("write");
    }
}
```

**Impact:** Custom clients can bypass all policies

### Scenario 4: File System Access

**Attack:**
```bash
# Direct file manipulation:
python3 << EOF
import json

# Read existing .rfsource file
with open(".rfsource", "rb") as f:
    data = f.read()

# Append malicious frame directly
malicious_frame = b'...'  # Crafted frame with >500 lines
with open(".rfsource", "ab") as f:
    f.write(malicious_frame)
