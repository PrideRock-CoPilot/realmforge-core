---
doc_id: DEC-COUNCIL-CODE-REVIEW-001
title: "Automated Code Review Module Architecture"
status: accepted
owner: cto
reviewers: [code-review, cto, domain-architect, backend, qa, tech-writer, pm, security-architect, api-architect]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: [FILE-SKILL-CODE-REVIEW, FILE-DOCS-CODE-REVIEW-PROCESS, FILE-DOCS-CODE-REVIEW-LEDGER]
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---

# DEC-COUNCIL-CODE-REVIEW-001 — Automated Code Review Module Architecture

**Council session date:** 2026-05-06
**Participants:** code-review (Owen Brooks), cto (Rena Okafor), domain-architect (Yusuf Osman), backend (Dmitri Volkov), qa (Meg Thompson), tech-writer (Clara Mills), pm (Alex Rivera), security-architect (Fatima Al-Hassan), api-architect (Marcus Webb)
**DRI (Directly Responsible Individual):** cto (Rena Okafor)

---

## Decision Question

Should the automated code review module use a three-axis standards taxonomy (language × framework × project context) with a dedicated `crates/code-review/` engine and `code-review-standards/` data directory, or should it use a simpler flat-language model with script-based execution?

## Options Considered

### Option A: Full Architecture — Three-Axis Taxonomy + Dedicated Crate

**Description:**
- `code-review-standards/` directory with YAML files organized by language, then framework overlay
- `crates/code-review/` — dedicated workspace crate with typed API, check engine, structured output
- Framework detection via explicit project config only (`.code-review.yaml`)
- ReviewReport maps directly to Owen's existing file review records

**Proponent:** cto (Rena Okafor)

**Arguments for:**
- Every new language or framework is a YAML file, not a code change
- Engine built once with typed API surface for MCP/CLI integration
- Structured ReviewReport feeds directly into area DONE logic
- Severity mapping (BLOCKER/REQUIRED_CHANGE/SHOULD_FIX/NOTE) is built-in, aligned with existing process

**Challenges raised:**
- Security (Fatima): CLI tool invocation (cargo clippy, ruff) is a command injection surface — sandbox boundary must be defined
- PM (Alex): 3-sprint investment — needs a clear phase placement
- Domain (Yusuf): Framework dependency manifest detection crosses bounded contexts; explicit config is cleaner
- The Council adopted this challenge: **framework detection must be explicit-only, no auto-detection**

### Option B: Flat Language + Script-Based Runner

**Description:**
- Standards live in `skills/code-review/standards/` as flat per-language YAML
- No dedicated crate — shell script or simple Rust binary
- Framework concerns delegated to native tooling (eslint, clippy)

**Proponent:** backend (Dmitri Volkov)

**Arguments for:**
- Less architectural overhead
- Faster to ship — could work today
- Native tooling already handles framework concerns

**Challenges raised:**
- CTO (Rena): Flat model doesn't scale — when we have 6 languages and 4 frameworks, base rules are duplicated across every variant
- Code Review (Owen): No structured ReviewReport means no automated area DONE logic; script stdout doesn't integrate with the ledger

### Option C: Three-Axis YAML Taxonomy Only, No Crate

**Description:**
- Design and ship the YAML data model and standards files only
- Use a Rust binary or script runner as interim execution layer
- Crate comes later as Phase 2

**Proponent:** pm (Alex Rivera)

**Arguments for:**
- Ships the data model (the hard part) faster
- Standards YAML is forward-compatible regardless of executor

**Challenges raised:**
- Backend (Dmitri): Script runner would need to parse YAML, detect languages, run regex, execute commands, produce structured output — effectively building the engine twice
- Tech Writer (Clara): Risk of divergence between the documented standards format and what the script actually produces

---

## Decision

**Option A — Adopted** with the following binding condition:

> **Framework detection shall use explicit project configuration only.** No auto-detection from file extensions (`.tsx`), dependency manifests (`package.json`), or any other implicit source. A `.code-review.yaml` file at the project root must declare which frameworks are active. If no config file exists, only base language rules apply.

## Implementation Scope

| Phase | Scope | Timeline |
|-------|-------|----------|
| Phase 1 | Rust standards only + crate engine with executable + regex + file-property checks | Sprint 1 |
| Phase 2 | Python standards + framework taxonomy for both languages | Sprint 2 |
| Phase 3 | TypeScript/JavaScript standards + MCP tool + CLI subcommand | Sprint 3 |
| Phase 4 | Framework standards for React, Django, Tokio/Axum (as projects adopt them) | Ongoing per need |

**Phase 1 deliverables:**
- `code-review-standards/` directory with `language-agnostic.yaml`, `rust.yaml`
- `crates/code-review/` with working engine for Rust checks
- `.code-review.yaml` config format defined
- Output format matching `docs/codex/05_CODE_REVIEW_PROCESS.md` file review records

## Rationale

1. **The taxonomy is the investment.** The three-axis model (language × framework × project context) is what makes the system extensible. Building a flat model now guarantees a rewrite later. The cost of getting the data model right the first time is lower than the cost of migrating from flat to multi-axis.

2. **The crate is warranted.** The engine is a cross-cutting concern — it needs a typed public API (ReviewReport, Finding, Severity), structured error handling, and the ability to be called from MCP tools and the CLI. A script runner cannot provide these without effectively becoming a crate with worse maintainability.

3. **Explicit config removes ambiguity.** The Council's binding condition — no auto-detection — eliminates false positives, cross-context coupling (reading dependency manifests from other build systems), and surprises when a project uses an unexpected framework variant. If a team wants framework rules, they opt in explicitly.

4. **Reversibility is acceptable.** The engine is a new crate with no runtime dependency on production services. No schema migrations. No changes to existing audit/policy/snapshot/command lifecycle. If the approach is wrong, the crate can be removed or deprecated without affecting the rest of the workspace.

## Dissenting Opinions

No dissenting opinions were recorded. The Council's position was unified on Option A with the explicit-config condition. The primary substantive modification (framework auto-detection → explicit config) was incorporated into the final decision, and no skill maintained a remaining objection.

## Consequences

**Positive:**
- Language extensibility is data-driven — any new language = one YAML file
- Structured output feeds directly into Owen's existing review ledger format
- MCP/CLI integration is a thin wrapper over a typed API
- Explicit framework config prevents surprise failures from misdetection
- File registry and metadata standard can reference check IDs for traceability

**Negative:**
- First sprint produces no visible output until the crate compiles and standards are defined
- Teams using frameworks must explicitly configure them (no "just works" from file extension)
- Security review required for the executable check runner (command injection surface)

**Risks:**
- Framework rules may proliferate — need governance to prevent per-project custom standards outside the shared taxonomy
- The crate must be kept thin over the data — if engine logic and standards logic blur, the extensibility benefit is lost
- Without auto-detection, framework rules will be underutilized until teams remember to configure them

## Follow-up

| Action | Owner | By |
|--------|-------|----|
| Write ADR (this document) | cto | 2026-05-06 |
| Write spec document in `docs/spec/` | tech-writer | Next sprint |
| Write phase plan in `docs/plan/` | pm | Next sprint |
| Register crate files in metadata standard | tech-writer | Next sprint |
| Security review of executable check runner | security-architect | Before Phase 1 ships |

**ADR reference:** DEC-COUNCIL-CODE-REVIEW-001 (this document)

---

**STATUS: CLOSED — 2026-05-06**
