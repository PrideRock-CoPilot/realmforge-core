---
doc_id: DOC-PLAN-CODE-REVIEW
title: Phase Code Review — Automated Code Review Module
parent: DOC-PLAN-INDEX
status: draft
owner: backend
reviewers: [code-review, cto, domain-architect, security-architect, qa, pm, tech-writer]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: [DEC-COUNCIL-CODE-REVIEW-001]
related_file_ids: [FILE-SKILL-CODE-REVIEW, FILE-DOCS-CODE-REVIEW-PROCESS, FILE-DOCS-CODE-REVIEW-LEDGER, FILE-CRATE-CODE-REVIEW-LIB, FILE-CRATE-CODE-REVIEW-CONFIG, FILE-CRATE-CODE-REVIEW-STANDARDS, FILE-CRATE-CODE-REVIEW-CHECK, FILE-CRATE-CODE-REVIEW-EXEC, FILE-CRATE-CODE-REVIEW-REGEX, FILE-CRATE-CODE-REVIEW-FILEPROP, FILE-CRATE-CODE-REVIEW-REPORT, FILE-CRATE-CODE-REVIEW-ERROR, FILE-CRATE-CODE-REVIEW-SEVERITY, FILE-STANDARDS-AGNOSTIC, FILE-STANDARDS-RUST]
visual_node_ids: [VN-CODE-REVIEW-MODULE]
visual_edge_ids: [VE-CODE-REVIEW-TO-LEADER, VE-CODE-REVIEW-TO-PEER, VE-CODE-REVIEW-TO-QA]
approval_state: pending
---

# Phase Code Review: Automated Code Review Module

## Overview

| Field | Value |
|-------|-------|
| Phase ID | Phase-CodeReview |
| Title | Automated Code Review Module |
| Work paths | `WP-DOCS-000` |
| Product module | codex-operations |
| Owner | backend |
| Risk | medium |
| Decision blockers | `DEC-COUNCIL-CODE-REVIEW-001` (resolved — Option A adopted) |

**Mandate:** Build the automated code review check engine crate (`crates/code-review/`), the standards registry (`code-review-standards/`), and the `.code-review.yaml` project configuration format. Phase 1 delivers Rust standards only. Update Owen's existing workflow to incorporate automated findings.

---

## Workflow

```
1. Create code-review-standards/ directory with initial YAML files
   a. language-agnostic.yaml — file size, TODO checks, absolute path checks
   b. rust.yaml — clippy, fmt, unwrap/unsafe checks, Rust-specific file size

2. Create crates/code-review/ with Cargo.toml and dependencies
   a. Add crates/code-review/ to workspace Cargo.toml members
   b. Dependencies: serde, serde_yaml, regex, globset, chrono, thiserror, walkdir

3. Implement Check Engine modules (in dependency order)
   a. error.rs — ReviewError enum with typed variants
   b. severity.rs — Severity enum (Blocker > RequiredChange > ShouldFix > Note)
   c. report.rs — Finding, ReviewSummary, ReviewReport structs
   d. config.rs — CodeReviewConfig parser from .code-review.yaml
   e. standards.rs — StandardsLoader, YAML schema deserialization, merge logic
   f. check.rs — Check trait, CheckDefinition, resolved check list
   g. executable.rs — ExecutableCheck: spawn, capture, timeout
   h. regex.rs — RegexCheck: walk files, apply patterns
   i. file_property.rs — FilePropertyCheck: stat, compare limits
   j. lib.rs — Public API: review_project()

4. Update Owen Brooks' skill file
   a. Add step 5 to workflow: "Run automated checks before manual review"
   b. Add automated findings as review record input

5. Update code-review.prompt.md
   a. Add automated check engine invocation instructions
   b. Add ReviewReport input format to prompt template

6. Create .code-review.yaml for realmforge workspace root
   a. language: rust
   b. framework: (empty — base rust rules only)
   c. project_context: RealmForge-specific rules

7. Run verification gates
   a. cargo check — workspace compiles with new crate
   b. cargo test — unit tests pass
   c. cargo clippy — no warnings
   d. Integration: run review_project() on realmforge workspace, verify findings

8. Update documentation
   a. 00_INDEX.md — add code review module entry
   b. 04_METADATA_STANDARD.md — add file registry entries
   c. 20_IMPLEMENTATION_ROADMAP.md — add code review phase
   d. 21_ACCEPTANCE_TEST_PLAN.md — add acceptance test entries
   e. docs/plan/README.md — add phase to index
```

---

## File Manifest

### NEW — `code-review-standards/`

| File | Artifact class | Risk |
|------|---------------|------|
| `code-review-standards/language-agnostic.yaml` | `runtime_policy` | low |
| `code-review-standards/rust.yaml` | `runtime_policy` | low |

### NEW — `crates/code-review/`

| File | Artifact class | Risk | Lines (est) |
|------|---------------|------|-------------|
| `crates/code-review/Cargo.toml` | `operator_config` | medium | 20 |
| `crates/code-review/src/lib.rs` | `compiled_source` | high | 80 |
| `crates/code-review/src/config.rs` | `compiled_source` | high | 100 |
| `crates/code-review/src/standards.rs` | `compiled_source` | high | 150 |
| `crates/code-review/src/check.rs` | `compiled_source` | medium | 60 |
| `crates/code-review/src/executable.rs` | `compiled_source` | critical | 120 |
| `crates/code-review/src/regex.rs` | `compiled_source` | medium | 80 |
| `crates/code-review/src/file_property.rs` | `compiled_source` | medium | 70 |
| `crates/code-review/src/report.rs` | `compiled_source` | medium | 100 |
| `crates/code-review/src/error.rs` | `compiled_source` | low | 40 |
| `crates/code-review/src/severity.rs` | `compiled_source` | low | 30 |

### NEW — project config

| File | Artifact class | Risk |
|------|---------------|------|
| `.code-review.yaml` (workspace root) | `operator_config` | low |

### EXTEND — workspace root

| File | Change | Risk |
|------|--------|------|
| `Cargo.toml` | Add `crates/code-review` to workspace members | medium |
| `Cargo.lock` | Updated automatically by cargo | low |

### UPDATE — documentation

| File | Change |
|------|--------|
| `docs/spec/00_INDEX.md` | Add entry for `26_CODE_REVIEW_MODULE_SPEC.md` |
| `docs/spec/04_METADATA_STANDARD.md` | Add file registry entries for new crate files |
| `docs/spec/20_IMPLEMENTATION_ROADMAP.md` | Add code review phase to roadmap |
| `docs/spec/21_ACCEPTANCE_TEST_PLAN.md` | Add code review acceptance tests |
| `docs/spec/22_OPEN_DECISIONS.md` | Already updated with DEC-COUNCIL-CODE-REVIEW-001 |
| `docs/plan/README.md` | Add phase to phase file index |
| `skills/code-review/SKILL.md` | Add automated check step to workflow |
| `code-review.prompt.md` | Add automated engine invocation section |

---

## Completion Gates

| Gate | Criteria | Check |
|------|----------|-------|
| Standards files exist | `code-review-standards/language-agnostic.yaml` and `rust.yaml` contain valid YAML with correct schema | `cargo test -p code-review --test test_standards_loader` |
| Crate compiles | `cargo check -p code-review` passes | `cargo check -p code-review --target-dir target-quality` |
| All unit tests pass | `cargo test -p code-review` passes | `cargo test -p code-review --target-dir target-quality` |
| ExecutableCheck works | External command (`echo test`) runs, exit code captured | Test: `test_executable_check` |
| RegexCheck works | Forbidden pattern detected in test file | Test: `test_regex_check_fail` |
| FilePropertyCheck works | Oversized file detected | Test: `test_file_property_line_count` |
| Config parsing works | `.code-review.yaml` parsed correctly | Test: `test_config_parse` |
| Missing config fallback | No `.code-review.yaml` → base language rules only | Test: `test_config_missing` |
| Workspace compiles | `cargo check --workspace` passes with new crate | `cargo check --workspace --target-dir target-quality` |
| Workspace clippy clean | `cargo clippy --workspace -D warnings` passes | `cargo clippy --workspace --all-targets --target-dir target-quality -- -D warnings` |
| Docs updated | All documentation files modified as listed above | Manual verification |
| Skill updated | `skills/code-review/SKILL.md` includes automated check step | Manual verification |

---

## Required Skill Grants

| Grant | Purpose | Scope |
|-------|---------|-------|
| `SGL-BACKEND-CORE` | Create and modify Rust crates | `crates/code-review/` |
| `SGL-TECH-WRITER` | Update documentation and spec files | `docs/*`, `skills/*` |

---

## Dependencies

| Dependency | Type | Status |
|------------|------|--------|
| `DEC-COUNCIL-CODE-REVIEW-001` | Decision | Resolved (2026-05-06) |
| Phase 0 (Documentation Certification) | Infrastructure | Completed |
| Phase 1 (Workspace) | Infrastructure | Completed |
| Rust toolchain | Tooling | Available |
| `serde` + `serde_yaml` | Crate dependency | Available on crates.io |
| `regex` | Crate dependency | Available on crates.io |
| `globset` | Crate dependency | Available on crates.io |
| `chrono` | Crate dependency | Available on crates.io |
| `thiserror` | Crate dependency | Available on crates.io |
| `walkdir` | Crate dependency | Available on crates.io |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Executable runner command injection | Low | Critical | Whitelist approach + no shell invocation; security review gate before Phase 1 ships |
| YAML parsing complexity | Medium | Medium | Use serde_yaml with strongly-typed structs; extensive unit tests for edge cases |
| Regex performance on large codebases | Low | Medium | File pattern filtering limits scope; serial execution prevents resource exhaustion |
| Framework overlay merge conflicts | Low | Low | Simple superseding rule (framework > base on same check ID); documented in spec |
| Standards YAML schema drift from engine | Medium | Low | Unit tests validate that every check definition in YAML deserializes correctly |
