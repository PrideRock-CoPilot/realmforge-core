---
doc_id: DOC-SPEC-026
title: "Code Review Module — Automated Check Engine and Standards Registry"
status: draft
owner: tech-writer
reviewers: [code-review, cto, domain-architect, backend, security-architect, qa, pm, api-architect]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: [DEC-COUNCIL-CODE-REVIEW-001]
related_file_ids: [FILE-SKILL-CODE-REVIEW, FILE-DOCS-CODE-REVIEW-PROCESS, FILE-DOCS-CODE-REVIEW-LEDGER]
visual_node_ids: [VN-CODE-REVIEW-MODULE]
visual_edge_ids: [VE-CODE-REVIEW-TO-LEADER, VE-CODE-REVIEW-TO-PEER, VE-CODE-REVIEW-TO-QA]
approval_state: pending
---

# RealmForge Code Review Module Specification

> **Council Decision Record:** DEC-COUNCIL-CODE-REVIEW-001 — Adopted with conditions.
> **DRI:** Rena (CTO) — **Date:** 2026-05-06

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [System Architecture](#2-system-architecture)
3. [Three-Axis Taxonomy](#3-three-axis-taxonomy)
4. [Standards Registry Format](#4-standards-registry-format)
5. [Check Engine Design](#5-check-engine-design)
6. [ReviewReport and Finding Types](#6-reviewreport-and-finding-types)
7. [Project Configuration Format](#7-project-configuration-format)
8. [Integration with Existing Process](#8-integration-with-existing-process)
9. [Crate & Module Layout](#9-crate--module-layout)
10. [API / MCP / CLI Contracts](#10-api--mcp--cli-contracts)
11. [Phased Rollout Plan](#11-phased-rollout-plan)
12. [Security & Threat Model](#12-security--threat-model)
13. [Testing Strategy](#13-testing-strategy)
14. [Open Questions](#14-open-questions)

---

## 1. Executive Summary

### Problem

Current code review at RealmForge is entirely manual — Owen Brooks reviews every file by hand. There is no automated check engine to catch common issues (unused imports, missing error handling, file-size violations) before human review. Standards are implicit in the reviewer's experience rather than codified as reusable data per language.

### Solution

A **three-layer code review system** that separates standards data from check execution:

1. **Standards Registry** (`code-review-standards/`) — Per-language YAML files defining checks, severity levels, and framework overlays
2. **Automated Check Engine** (`crates/code-review/`) — Rust crate that reads standards, runs checks, and produces structured ReviewReport output
3. **Human Review Interface** — Owen's existing file-by-file review process, augmented with automated findings as input

### Council Decision Summary

| Element | Decision |
|---|---|
| Architecture | Three-layer: standards data → check engine → human overlay |
| Language taxonomy | Three-axis: Base Language × Framework Overlay × Project Context |
| Framework detection | Explicit `.code-review.yaml` only — no auto-detection |
| Check types | ExecutableCheck (external CLI), RegexCheck (pattern matching), FilePropertyCheck (size, encoding) |
| Severity mapping | BLOCKER, REQUIRED_CHANGE, SHOULD_FIX, NOTE — aligned with existing process |
| Crate placement | `crates/code-review/` (pure engine), standards as YAML data at `code-review-standards/` |
| Phase 1 scope | Rust standards + crate engine with executable + regex + file-property checks |

---

## 2. System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  code-review-standards/                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ rust.yaml     │  │ python.yaml  │  │ language-        │  │
│  │ (Rust checks) │  │ (Python chk) │  │ agnostic.yaml    │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                  │                    │            │
│         └──────────────────┴────────────────────┘            │
│                              │                               │
│         ┌────────────────────┴───────────────────┐            │
│         │    Framework overlays (optional)       │            │
│         │  ┌──────────┐  ┌───────────┐           │            │
│         │  │ react    │  │ tokio-    │           │            │
│         │  │ overlay   │  │ axum-over│           │            │
│         │  └──────────┘  └───────────┘           │            │
│         └──────────────────────────────────────────────────│
└─────────────────────┬───────────────────────────────────────┘
                      │ reads
                      ▼
┌──────────────────────────────────────────────────────────────┐
│              crates/code-review/ (Check Engine)               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Standards    │  │ Check Runner │  │ Report Builder   │  │
│  │ Loader       │──▶│ (exec,regex, │──▶│ (aggregates      │  │
│  │ (parse YAML) │  │  file-prop)  │  │  findings)       │  │
│  └──────────────┘  └──────────────┘  └────────┬─────────┘  │
│                                               │              │
│  ┌────────────────────────────────────────────┘              │
│  │  Public API: review_project() -> ReviewReport             │
│  └───────────────────────────────────────────────────────────┘
                      │ outputs
                      ▼
┌──────────────────────────────────────────────────────────────┐
│              ReviewReport → Owen's Review Process             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Automated    │  │ Human        │  │ Final            │  │
│  │ Findings     │──▶│ Override     │──▶│ Review Record    │  │
│  │ (pre-populate)│  │ (confirm/    │  │ (Area DONE or    │  │
│  │              │  │  reject)     │  │ CHANGES_REQUIRED)│  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

| Layer | Responsibility | Location |
|-------|---------------|----------|
| Standards Registry | Store per-language check definitions as data | `code-review-standards/` |
| Check Engine | Parse standards, execute checks, build structured report | `crates/code-review/` |
| Human Review | Confirm/reject automated findings, add human-only findings | Owen's existing process |

### Integration Points

- **MCP tool:** `run_code_review` — takes project path, language, optional framework config; returns ReviewReport
- **CLI subcommand:** `realmforge code-review run [--config <path>]` — wrapper over the crate
- **Owen's workflow:** Automated findings populate half of the File Review Record template before human review
- **Ledger:** ReviewReport findings map directly to the ledger format in `06_CODE_REVIEW_LEDGER.md`

---

## 3. Three-Axis Taxonomy

### Axis 1: Base Language

The language dimension captures rules specific to a programming language's semantics, idioms, and tooling.

| Language ID | Standards file | Primary tools |
|------------|----------------|---------------|
| `rust` | `rust.yaml` | `cargo clippy`, `cargo fmt --check` |
| `python` | `python.yaml` | `ruff`, `black --check`, `mypy` |
| `typescript` | `typescript.yaml` | `eslint`, `prettier --check`, `tsc --noEmit` |
| `go` | `go.yaml` | `gofmt`, `go vet`, `staticcheck` |
| `cobol` | `cobol.yaml` | (when added — no auto-detection, explicit config only) |
| `csharp` | `csharp.yaml` | (when added — explicit config only) |

### Axis 2: Framework Overlay

Framework rules are **additive overlays** on top of base language rules. They are never applied unless explicitly declared in `.code-review.yaml`.

| Framework ID | Base language | Example checks |
|-------------|---------------|----------------|
| `react` | typescript | Hook rules of hooks, JSX accessibility, propTypes |
| `tokio-axum` | rust | Async best practices, tower service patterns |
| `django` | python | ORM query patterns, middleware ordering |
| `actix-web` | rust | Actor boundary checks (when added) |

### Axis 3: Project Context

Language-agnostic rules that apply regardless of language:
- File size limits (≤300 lines target, ≤500 hard cap per layer law)
- Naming conventions (project-level overrides)
- Header/license checks
- Banned import patterns (per project config)

### Taxonomy Resolution Algorithm

```
1. Read .code-review.yaml from project root (if absent → base language only)
2. Load base language standards from code-review-standards/<language>.yaml
3. If .code-review.yaml declares frameworks, load each overlay:
     code-review-standards/<framework>.yaml
4. Merge: framework overlay rules supersede base rules on same check ID
5. Apply project context rules from .code-review.yaml
6. Run all resolved checks against the project source
```

### Example `.code-review.yaml`

```yaml
# .code-review.yaml
language: rust
frameworks:
  - tokio-axum
project_context:
  max_file_size: 500
  naming_convention: snake_case
  banned_patterns:
    - pattern: "unwrap\\(\\)"
      reason: "Require proper error handling — use ? operator or typed error"
      severity: REQUIRED_CHANGE
    - pattern: "unsafe\\s*\\{"
      reason: "Unsafe blocks require review and SAFETY comment"
      severity: BLOCKER
```

---

## 4. Standards Registry Format

### Directory Structure

```
code-review-standards/
├── language-agnostic.yaml      # Rules that apply to all languages
├── rust.yaml                   # Rust-specific rules and tool config
├── python.yaml                 # Python-specific rules (Phase 2)
├── typescript.yaml             # TypeScript rules (Phase 3)
├── react.yaml                  # React framework overlay (Phase 4+)
├── tokio-axum.yaml             # Tokio/Axum framework overlay (Phase 4+)
└── django.yaml                 # Django framework overlay (Phase 4+)
```

### Schema

Every standards file uses the same top-level structure:

```yaml
# code-review-standards/rust.yaml
language: rust
version: "1.0"
description: "RealmForge Rust code review standards"

checks:
  - id: RUST-EXEC-001
    name: "Clippy Linting"
    description: "Run cargo clippy with -D warnings"
    check_type: executable
    command: ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
    working_dir: workspace_root
    timeout_secs: 120
    severity: BLOCKER

  - id: RUST-EXEC-002
    name: "Format Check"
    description: "Check Rust formatting with cargo fmt"
    check_type: executable
    command: ["cargo", "fmt", "--all", "--", "--check"]
    working_dir: workspace_root
    timeout_secs: 60
    severity: REQUIRED_CHANGE

  - id: RUST-REGEX-001
    name: "No Bare Unwrap"
    description: "Detect unwrap() calls without invariant comment"
    check_type: regex
    pattern: "unwrap\\(\\)"
    allowed_pattern: "//\\s*SAFETY:|//\\s*invariant:"
    file_pattern: "**/*.rs"
    severity: REQUIRED_CHANGE

  - id: RUST-REGEX-002
    name: "Unsafe Block Comment Required"
    description: "Every unsafe block must have a SAFETY comment"
    check_type: regex
    pattern: "unsafe\\s*\\{"
    required_above: "//\\s*SAFETY:"
    file_pattern: "**/*.rs"
    severity: BLOCKER

  - id: RUST-PROP-001
    name: "File Size Check"
    description: "Source files should not exceed the hard cap"
    check_type: file_property
    property: line_count
    max: 500
    warning_at: 300
    file_pattern: "**/*.rs"
    severity: REQUIRED_CHANGE
```

### Check Type Definitions

#### `executable`

Runs an external command and parses its exit code and stdout/stderr.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique check ID (e.g. `RUST-EXEC-001`) |
| `name` | string | yes | Human-readable check name |
| `description` | string | yes | What the check validates |
| `check_type` | string | yes | Must be `"executable"` |
| `command` | string[] | yes | Executable and arguments |
| `working_dir` | string | no | Where to run (default: `workspace_root`) |
| `timeout_secs` | integer | no | Maximum execution time (default: 60) |
| `severity` | string | yes | Mapping of exit codes to severity |
| `passthrough` | boolean | no | If true, stdout is included verbatim in findings |

#### `regex`

Searches file contents for forbidden patterns.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique check ID |
| `name` | string | yes | Human-readable name |
| `description` | string | yes | What the check validates |
| `check_type` | string | yes | Must be `"regex"` |
| `pattern` | string | yes | Regex to search for (forbidden occurrence) |
| `allowed_pattern` | string | no | Regex that negates the finding when present above match |
| `required_above` | string | no | Comment pattern that must appear on a preceding line |
| `file_pattern` | string | yes | Glob pattern for files to search |
| `severity` | string | yes | Severity if pattern is found |

#### `file_property`

Validates properties of individual files.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique check ID |
| `name` | string | yes | Human-readable name |
| `description` | string | yes | What the check validates |
| `check_type` | string | yes | Must be `"file_property"` |
| `property` | string | yes | Property to check (`line_count`, `file_size_bytes`, `encoding`) |
| `max` | integer | no | Maximum allowed value |
| `min` | integer | no | Minimum allowed value |
| `warning_at` | integer | no | Threshold for SHOULD_FIX severity |
| `file_pattern` | string | yes | Glob pattern for files to check |
| `severity` | string | yes | Severity if property exceeds max/min |

### Language-Agnostic Standards

File: `code-review-standards/language-agnostic.yaml`

```yaml
language: "*"
version: "1.0"
description: "Language-agnostic code review standards"

checks:
  - id: ALL-PROP-001
    name: "File Size Hard Cap"
    description: "No file exceeds 500 lines without written justification"
    check_type: file_property
    property: line_count
    max: 500
    warning_at: 300
    file_pattern: "**/*"
    severity: REQUIRED_CHANGE

  - id: ALL-REGEX-001
    name: "No TODO Commits"
    description: "Detect unresolved TODOs in production code"
    check_type: regex
    pattern: "(?i)TODO|FIXME|HACK|XXX"
    file_pattern: "src/**/*.rs"
    severity: NOTE

  - id: ALL-REGEX-002
    name: "No Absolute Filesystem Paths"
    description: "Detect hardcoded filesystem paths in source"
    check_type: regex
    pattern: '["\']/[a-zA-Z0-9_/-]+/["\']'
    file_pattern: "src/**/*.rs"
    severity: SHOULD_FIX
```

---

## 5. Check Engine Design

### Public API

```rust
// crates/code-review/src/lib.rs

/// Run a full code review for a project.
///
/// * `project_root` — Path to the project root (where .code-review.yaml lives)
/// * `config` — Optional explicit config. If None, engine looks for .code-review.yaml
/// * `checks_filter` — Optional list of check IDs to run (None = all)
///
/// Returns a ReviewReport with all findings grouped by file.
pub fn review_project(
    project_root: &Path,
    config: Option<CodeReviewConfig>,
    checks_filter: Option<&[CheckId]>,
) -> Result<ReviewReport, ReviewError>;
```

### Internal Module Structure

```
crates/code-review/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public API: review_project()
    ├── config.rs           # CodeReviewConfig parsing from .code-review.yaml
    ├── standards.rs        # StandardsLoader — loads and merges YAML files
    ├── check.rs            # Check trait and per-type implementations
    │   ├── executable.rs   # ExecutableCheck runner (spawns process)
    │   ├── regex.rs        # RegexCheck runner (grep over files)
    │   └── file_property.rs# FilePropertyCheck runner (stat files)
    ├── report.rs           # ReviewReport, ReviewSummary, Finding types
    ├── error.rs            # Typed error enum
    └── severity.rs         # Severity enum (BLOCKER, REQUIRED_CHANGE, SHOULD_FIX, NOTE)
```

### Check Runner Sequence

```
1. StandardsLoader.load_all(code-review-standards/) → Vec<CheckDefinition>
2. Merge with framework overlays from .code-review.yaml
3. For each check:
   a. ExecutableCheck: spawn process, capture exit code + stdout/stderr
   b. RegexCheck: walk files matching file_pattern, run regex
   c. FilePropertyCheck: stat matching files, compare against limits
4. Collect all Finding results
5. Build ReviewReport (grouped by file, sorted by severity)
6. Return ReviewReport
```

### Error Handling

All errors are typed via the `ReviewError` enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Standards loading failed: {0}")]
    StandardsError(String),

    #[error("Check execution failed: {0}")]
    ExecutionError(String),

    #[error("Timeout executing check '{check_id}': {duration}s elapsed")]
    Timeout { check_id: CheckId, duration: u64 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(String),
}
```

---

## 6. ReviewReport and Finding Types

### Core Types

```rust
/// ID representing a named check (e.g., "RUST-EXEC-001")
pub type CheckId = String;

/// Severity aligned with Owen Brooks' existing classification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Must be fixed before area can proceed — blocks DONE
    Blocker,
    /// Should be fixed before area is DONE; owner may accept risk
    RequiredChange,
    /// Should be fixed but does not block the area
    ShouldFix,
    /// Observation or suggestion — no action required
    Note,
}

/// A single finding from one check on one file
#[derive(Debug, Clone)]
pub struct Finding {
    /// Check ID that produced this finding
    pub check_id: CheckId,
    /// Human-readable check name
    pub check_name: String,
    /// File path relative to project root
    pub file_path: PathBuf,
    /// Line number if applicable
    pub line_number: Option<usize>,
    /// Severity of the finding
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Detailed explanation (optional)
    pub detail: Option<String>,
    /// Suggested fix or remediation (optional)
    pub suggestion: Option<String>,
}

/// Summary of the entire code review run
#[derive(Debug, Clone)]
pub struct ReviewSummary {
    pub total_checks: usize,
    pub checks_passed: usize,
    pub checks_failed: usize,
    pub checks_skipped: usize,
    pub total_files_checked: usize,
    pub blocker_count: usize,
    pub required_change_count: usize,
    pub should_fix_count: usize,
    pub note_count: usize,
}

/// Complete report from a code review run
#[derive(Debug, Clone)]
pub struct ReviewReport {
    /// Project root path
    pub project_root: PathBuf,
    /// Language that was reviewed
    pub language: String,
    /// Optional framework overlays applied
    pub frameworks: Vec<String>,
    /// Timestamp of the review
    pub reviewed_at: chrono::DateTime<chrono::Utc>,
    /// All findings grouped by file path
    pub findings: Vec<Finding>,
    /// Summary statistics
    pub summary: ReviewSummary,
    /// True if no BLOCKER or REQUIRED_CHANGE findings exist
    pub is_clean: bool,
}
```

### Mapping to Existing Review Record

The automated ReviewReport feeds directly into Owen's file review record format:

| Automated Field | Maps to Existing Record Field |
|----------------|-------------------------------|
| `Finding.severity` | `severity` in review record |
| `Finding.file_path` | `file_path` in review record |
| `Finding.check_id` | Prefix for `finding_id` |
| `Summary.blocker_count` | Used in area summary |

Example mapping (from `docs/codex/05_CODE_REVIEW_PROCESS.md`):

```markdown
## File Review Record: src/policy/engine.rs
**Reviewer:** automated (code-review v1.0)
**File type:** compiled_source — Authority Core
**Review date:** 2026-05-06

| # | Finding | Severity | Required owner | Resolved? |
|---|---------|----------|----------------|-----------|
| 1 | [RUST-REGEX-001] `unwrap()` call at line 142 without invariant comment | REQUIRED_CHANGE | backend | OPEN |
| 2 | [RUST-PROP-001] File is 487 lines (warning at 300, hard cap 500) | REQUIRED_CHANGE | backend | OPEN |

**Automated check result:** 12 checks passed, 2 findings, 0 blockers
```

---

## 7. Project Configuration Format

The `.code-review.yaml` file is the **only** mechanism for declaring frameworks and project-specific overrides.

### Full Schema

```yaml
# .code-review.yaml — Project code review configuration

# Required: programming language
language: rust

# Optional: list of active frameworks
frameworks:
  - tokio-axum

# Optional: project-context overrides
project_context:
  # Override max file size for this project
  max_file_size: 500

  # Override naming convention
  naming_convention: snake_case

  # Add project-specific banned patterns
  banned_patterns:
    - pattern: "unwrap\\(\\)"
      reason: "Require proper error handling"
      severity: REQUIRED_CHANGE

  # Exclude specific files or directories from regex/file-property checks
  exclude_patterns:
    - "generated/**"
    - "vendor/**"
    - "**/*.pb.rs"

# Optional: disable specific checks by ID
disabled_checks:
  - ALL-REGEX-002  # Absolute paths check not relevant for this project
```

### Placement Rules

- Must be at the **project root** (the directory passed to `review_project()`)
- Only one `.code-review.yaml` per project
- If absent, only base language rules apply — no framework overlays, no project overrides
- Unknown framework IDs produce a `NOTE`-severity finding (not a hard error)

---

## 8. Integration with Existing Process

### Before-Automation Process

```
Owen reads every file manually
  → Creates review records by hand
  → Runs cargo clippy / cargo fmt manually
  → Populates findings from memory
```

### After-Automation Process

```
1. Automated Check Engine runs (CLI or MCP)
   → Produces ReviewReport with all automated findings
2. Owen receives ReviewReport as input
   → Automated findings pre-populate ~50% of review records
3. Owen reviews each finding
   → Confirms (accepts into final record) or rejects (false positive)
4. Owen adds human-only findings
   → Architecture concerns, design patterns, contract boundary issues
5. Owen produces Final Review Record
   → Area DONE or CHANGES_REQUIRED recommendation
```

### Integration Points

| Integration | Mechanism | Phase |
|------------|-----------|-------|
| Pre-populate review records | ReviewReport → template filling | 1 |
| CI/CD pipeline check | CLI subcommand in pre-commit hook | 1 |
| MCP tool invocation | `run_code_review` tool called by agent | 3 |
| Governance gate | ReviewReport.is_clean must be true before peer review | 1 |
| Ledger update | Summary stats recorded in review ledger | 1 |

### Skill Update: Owen's New Workflow

Owen's existing `skills/code-review/SKILL.md` gains a new step:

5. **Run automated checks.** Before manual review, invoke the code review engine (CLI or MCP) to produce a ReviewReport. Automated findings pre-populate the file review template. Confirm or reject each automated finding before adding human-only findings.

---

## 9. Crate & Module Layout

### `crates/code-review/` — Check Engine

| File | Artifact class | Responsibility | Risk |
|------|---------------|----------------|------|
| `Cargo.toml` | `operator_config` | Crate manifest, dependencies | medium |
| `src/lib.rs` | `compiled_source` | Public API (`review_project()`) | high |
| `src/config.rs` | `compiled_source` | `.code-review.yaml` parsing | high |
| `src/standards.rs` | `compiled_source` | Standards YAML loader and merger | high |
| `src/check.rs` | `compiled_source` | Check trait definition | medium |
| `src/executable.rs` | `compiled_source` | External CLI tool runner | critical |
| `src/regex.rs` | `compiled_source` | Regex-based file checker | medium |
| `src/file_property.rs` | `compiled_source` | File property validator | medium |
| `src/report.rs` | `compiled_source` | ReviewReport, Finding, Summary types | medium |
| `src/error.rs` | `compiled_source` | Typed error enum | low |
| `src/severity.rs` | `compiled_source` | Severity enum | low |

### `code-review-standards/` — Standards Data

| File | Artifact class | Responsibility | Risk |
|------|---------------|----------------|------|
| `language-agnostic.yaml` | `runtime_policy` | Language-agnostic rules | low |
| `rust.yaml` | `runtime_policy` | Rust-specific rules | low |

### Dependencies

The `crates/code-review/` crate depends on:
- `serde` + `serde_yaml` — YAML parsing
- `regex` — Regex execution
- `globset` — File pattern matching
- `chrono` — Timestamps
- `thiserror` — Typed errors
- `walkdir` — Recursive file walking
- `tokio` (optional) — Async process execution for executable checks

The crate does **not** depend on:
- `authority-domain` — No domain coupling (cross-cutting tool)
- `control-store` — No persistence dependency
- `policy-engine` — No policy model coupling

---

## 10. API / MCP / CLI Contracts

### CLI Subcommand (Phase 1)

```
realmforge code-review run [OPTIONS]

Options:
  -p, --project-path <PATH>     Project root (default: .)
  -c, --config <PATH>           Explicit config path (default: .code-review.yaml)
  -l, --language <LANG>         Override language detection
  --checks <IDS>                Run only specific checks (comma-separated)
  --json                        Output as JSON
  --output <FILE>               Write report to file
  --verbose                     Verbose output

Exit codes:
  0 — All checks passed or only NOTES found
  1 — SHOULD_FIX findings exist
  2 — REQUIRED_CHANGE findings exist
  3 — BLOCKER findings exist
  >10 — Internal error
```

### MCP Tool (Phase 3)

```
Tool: run_code_review
Description: Run automated code review for a project

Input:
  project_root: string  (required) — Path to project root
  language: string     (required) — Programming language
  frameworks: string[] (optional) — Active framework overlays
  checks: string[]     (optional) — Specific check IDs to run

Output:
  review_report: ReviewReport (structured finding collection)
  is_clean: boolean

Errors:
  REVIEW_CONFIG_NOT_FOUND   — .code-review.yaml missing
  REVIEW_LANGUAGE_UNSUPPORTED — Language has no standards file
  REVIEW_EXECUTION_FAILED    — Check execution error
```

### REST API (Phase 3 — if needed)

```
POST /api/v1/code-review/run
Body: { project_root, language, frameworks?, checks? }
Response: 200 { review_report, is_clean }
Response: 422 { error: "Unsupported language" }
Response: 500 { error: "Execution failed" }
```

---

## 11. Phased Rollout Plan

| Phase | Scope | Timeline | Deliverables |
|-------|-------|----------|-------------|
| **Phase 1** | Rust standards + crate engine | Sprint 1 | `crates/code-review/` engine, `code-review-standards/` (rust.yaml + language-agnostic.yaml), CLI subcommand, `.code-review.yaml` format, pre-populated review records |
| **Phase 2** | Python standards | Sprint 2 | `python.yaml`, framework taxonomy for both languages, exectuable checks for ruff/black/mypy |
| **Phase 3** | TypeScript/JavaScript + MCP tool | Sprint 3 | `typescript.yaml`, MCP `run_code_review` tool, REST API route |
| **Phase 4** | Framework overlays per adoption | Ongoing | `react.yaml`, `tokio-axum.yaml`, `django.yaml` as projects adopt them |

### Phase 1 Detailed Deliverables

1. `code-review-standards/language-agnostic.yaml` — File size, TODO checks, absolute path checks
2. `code-review-standards/rust.yaml` — Clippy, fmt, unwrap check, unsafe check, file size per Rust file
3. `crates/code-review/` with:
   - YAML standards loader
   - ExecutableCheck runner (spawns process, captures output)
   - RegexCheck runner (walks files, applies regex)
   - FilePropertyCheck runner (stats files, compares limits)
   - ReviewReport builder with severity sorting
   - Typed error handling
4. `.code-review.yaml` schema and parser
5. CLI subcommand skeleton (real output, not stub)
6. Unit tests for each check runner
7. Integration test: run `cargo clippy` on a test project via ExecutableCheck

---

## 12. Security & Threat Model

### Executable Check Surface

The ExecutableCheck runner spawns external processes. This is a **command injection surface** (per Fatima Al-Hassan's Council challenge).

| Threat | Mitigation | Status |
|--------|-----------|--------|
| Arbitrary command in standards YAML | Commands must be from a whitelist (registered CLI tools: `cargo`, `ruff`, `black`, `eslint`, etc.) | Design |
| Shell injection via file paths | No shell execution — use `std::process::Command` with argument array, not shell string | Design |
| Timeout exhaustion | Per-check timeout (configurable in standards YAML, default 60s) | Design |
| Resource exhaustion | Maximum concurrent checks = 1 (serial execution); total timeout per project | Design |
| Path traversal via config | `.code-review.yaml` path is validated to be within project root; symlink checking | Design |

### Sandbox Boundary

The check engine runs with the calling process's permissions. For CI/CD pipeline use:
- Must be invoked in an isolated environment (container or restricted runner)
- No network access during check execution
- Temporary working directory is cleaned after each run

### Security Review Gate

Before Phase 1 ships, the security-architect (Fatima) must review:
1. The executable check runner implementation (command whitelist, argument validation, timeout)
2. The YAML parser's handling of untrusted `.code-review.yaml` files
3. The file walker's behavior with symlinks, fifos, and other special files

---

## 13. Testing Strategy

### Unit Tests

| Test | Scope | Covers |
|------|-------|--------|
| `test_standards_loader` | `standards.rs` | Load rust.yaml, verify check count and structure |
| `test_standards_merge` | `standards.rs` | Merge base + framework overlay, verify superseding |
| `test_executable_check` | `executable.rs` | Run echo command, verify exit code mapping |
| `test_executable_timeout` | `executable.rs` | Verify timeout kills hung process |
| `test_regex_check_pass` | `regex.rs` | File with no forbidden pattern |
| `test_regex_check_fail` | `regex.rs` | File with forbidden pattern, verify finding |
| `test_regex_allowed_pattern` | `regex.rs` | Pattern with allowed_pattern, verify suppression |
| `test_file_property_line_count` | `file_property.rs` | Small file and oversized file |
| `test_report_empty` | `report.rs` | ReviewReport with no findings |
| `test_severity_ordering` | `severity.rs` | Blocker > RequiredChange > ShouldFix > Note |
| `test_config_parse` | `config.rs` | Valid `.code-review.yaml` parsing |
| `test_config_missing` | `config.rs` | Missing config file → base language only |

### Integration Tests

| Test | Description |
|------|-------------|
| `test_integration_basic_rust` | Run code-review CLI on a well-formed Rust project, verify clean report |
| `test_integration_violations` | Run on a project with known violations, verify findings detected |
| `test_integration_framework_merge` | Run with framework overlay, verify framework rules added |

### Acceptance Tests

| Test ID | Assertion |
|---------|-----------|
| `TEST-CODE-REVIEW-001` | Check engine loads and validates a standards YAML file |
| `TEST-CODE-REVIEW-002` | ExecutableCheck runs an external command and captures exit code |
| `TEST-CODE-REVIEW-003` | RegexCheck finds violations in source files |
| `TEST-CODE-REVIEW-004` | FilePropertyCheck detects oversized files |
| `TEST-CODE-REVIEW-005` | Framework overlay rules supersede base rules on same check ID |
| `TEST-CODE-REVIEW-006` | ReviewReport groups findings by file and sorts by severity |
| `TEST-CODE-REVIEW-007` | Missing `.code-review.yaml` falls back to base language rules |
| `TEST-CODE-REVIEW-008` | Explicit `.code-review.yaml` with framework ID activates overlay |
| `TEST-CODE-REVIEW-009` | ExecutableCheck timeout kills hung processes |
| `TEST-CODE-REVIEW-010` | Security: command whitelist rejects unregistered executables |

---

## 14. Open Questions

| Question | Status | Owner |
|----------|--------|-------|
| Where should `crates/code-review` appear in the workspace dependency order? | Under Phase 2 (Authority Core) — it needs workspace compilation but no crate dependency | Will be resolved in phase planning |
| Should the engine cache results between runs? | Deferred — no caching in Phase 1; revisit in Phase 3 if performance is a concern | backend |
| How to handle multi-language projects (monorepos)? | Deferred — Phase 1 assumes single-language projects. Monorepo support in Phase 3+ | domain-architect |
| Should executable checks be sandboxed (container, WASM)? | Deferred — security review gate before Phase 1 ships; sandbox decision documented in security review | security-architect |
