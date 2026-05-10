# Documentation Archive

This directory contains references to historical documentation that tracked the evolution of RealmForge through its early development phases.

## Purpose

These documents captured progress, decisions, and iterations during the initial buildout of core systems. They are archived here to:

* **Preserve decision context** - Understanding why architectural choices were made
* **Track evolution** - Showing how systems progressed from design to implementation
* **Historical reference** - Available for future contributors seeking context
* **Maintain focus** - Keeping main docs focused on current state, not historical process

## Archived Documentation Series

### Intake System Development (Completed)

The intake system underwent multiple design and approval phases before reaching its current implementation in `crates/intake-engine`.

**Historical Documents** (from original repository):
* `INTAKE_SYSTEM_DESIGN.md` - Initial design proposal
* `INTAKE_SYSTEM_COUNCIL_PRESENTATION.md` - Council review materials
* `INTAKE_SYSTEM_APPROVAL_REQUESTS.md` - Stakeholder signoff requests
* `INTAKE_SYSTEM_SIGNOFF_TRACKER.md` - Approval tracking
* `INTAKE_SYSTEM_FINAL_APPROVAL.md` - Final approval documentation
* `INTAKE_SYSTEM_COMPLETION_SUMMARY.md` - Implementation completion summary
* `INTAKE_PHASE_0a_PROGRESS.md` - Phase 0a progress tracking
* `INTAKE_PHASE_0b_PROGRESS.md` - Phase 0b progress tracking  
* `INTAKE_PHASE_0b_SUMMARY.md` - Phase 0b summary

**Current Status**: Intake system is implemented in `crates/intake-engine`. See `docs/spec/00_INDEX.md` for current specifications.

### Gap Analysis & Backend Planning (Completed)

Early analysis of missing functionality and backend implementation requirements.

**Historical Documents**:
* `BACKEND_GAP_ANALYSIS.md` - Analysis of missing backend components
* `GAP_FILLING_PROGRESS.md` - Progress tracking for gap closure
* `APPLICATION_TYPE_MATRIX.md` - Application type taxonomy

**Current Status**: These analyses informed the current crate structure. See `docs/architecture/` for current architecture documentation.

## Storage Format Decision (Resolved)

A major architectural decision involved choosing between custom `.rfsource` format, DuckDB, and Delta Lake.

**Decision**: Custom `.rfsource` format with git-like commit/history functions
**Rationale**: POC completed in <2 hours, Parquet-equivalent compression, full governance control
**Implementation**: `crates/rfsource-*` (8-crate suite)
**Migration**: Replaced `parquet-store` crate

See `docs/architecture/rfsource-overview.md` for current architecture and `CHANGELOG.md` for migration notes.

## Accessing Historical Documents

Historical documents are preserved in the git history of the original `realmforge-core` repository. To review them:

```bash
# Checkout the original branch
git checkout main

# View historical docs
ls docs/*.md | grep -E "INTAKE|GAP|PHASE"

# View specific document
cat docs/INTAKE_SYSTEM_DESIGN.md
```

## Current Documentation

For up-to-date project documentation, see:

* `README.md` - Project overview and quick start
* `docs/spec/00_INDEX.md` - Canonical specifications
* `docs/architecture/` - Architecture documentation
* `docs/decisions/` - Architectural decision records (ADRs)
* `CHANGELOG.md` - Breaking changes and migrations

---

*This archive was created during the storage architecture refactor when transitioning from `parquet-store` to the `rfsource-*` suite.*
