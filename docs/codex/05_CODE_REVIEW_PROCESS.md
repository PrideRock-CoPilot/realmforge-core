---
doc_id: DOC-CODEX-005
title: Code Review Process And File Review Format
status: active
owner: code-review
reviewers: [peer-review, cto, qa, tech-writer]
created_at: 2026-05-05
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: [FILE-DOCS-CODE-REVIEW-PROCESS, FILE-DOCS-CODE-REVIEW-LEDGER, FILE-SKILL-CODE-REVIEW]
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---

# Code Review Process And File Review Format

## Purpose

This document defines the RealmForge code-review gate. It is active for the
temporary project board in `docs/codex/04_CLINE_NEXT_PHASE_PLANNING_HANDOFF.md`
until a dedicated board surface exists.

Code review is a file-level implementation gate. An area cannot move its
`Code Review` matrix cell to `DONE` unless every in-scope file has gone through
the file review process below.

## Gate Position

Lifecycle order:

```text
Developed -> Peer Review -> Code Review -> QA Test -> QA Sign Off -> UAT
```

Code review consumes peer-reviewed implementation evidence and produces a
reviewed build candidate for QA. It does not replace peer review, QA, UAT,
release, or architecture approval.

## Area DONE Rule

An area's `Code Review` cell may move to `DONE` only when all conditions are true:

- The area has an explicit file inventory.
- The inventory includes metadata-registry files, phase-plan files, current diff
  files, generated files, config files, contracts, migrations, tests, and runtime
  definitions that affect the area.
- Every in-scope file has one `File Review Record`.
- Every file status is `APPROVED` or `ACCEPTED_WITH_RISK`.
- No `BLOCKER` or `REQUIRED_CHANGE` finding remains open.
- Required static checks are cited in the area summary.
- The next gate is named.

If any file is missing from the inventory, the area is `PENDING` or `BLOCKED`,
not `DONE`.

## Status Values

| Status | Meaning |
| --- | --- |
| `PENDING` | Review has not started or inventory is incomplete. |
| `IN_REVIEW` | File review records are being created. |
| `CHANGES_REQUIRED` | At least one required change is open. |
| `BLOCKED` | Review cannot continue without a decision, artifact, or owner action. |
| `APPROVED` | File passed review with no required changes. |
| `ACCEPTED_WITH_RISK` | File has a known non-blocking risk accepted by the authorized owner. |

## Finding Severity

| Severity | Blocks DONE | Use |
| --- | --- | --- |
| `BLOCKER` | Yes | Security boundary break, layer-law violation, missing required file, invalid contract, failing gate. |
| `REQUIRED_CHANGE` | Yes | Correctness, maintainability, test, validation, or error-handling issue that must be fixed. |
| `SHOULD_FIX` | No, unless owner escalates | Improvement that should be addressed soon but does not block this gate. |
| `NOTE` | No | Observation, context, or follow-up for later work. |

## File Review Record

Every reviewed file must use this format:

```markdown
### FILE-REVIEW-[AREA]-[NNN]

| Field | Value |
| --- | --- |
| Area | Phase N - Name |
| File | `path/to/file` |
| File ID | `FILE-ID` or `UNREGISTERED_BLOCKER` |
| Artifact class | compiled_source / migration / runtime_definition / documentation / etc. |
| Owner skill | backend / frontend / data-engineer / tech-writer / etc. |
| Risk | low / medium / high / critical |
| Governing docs | `docs/spec/...`, `docs/plan/...` |
| Required tests | `TEST-ID`, command, or `N/A with reason` |
| Review status | PENDING / APPROVED / CHANGES_REQUIRED / BLOCKED / ACCEPTED_WITH_RISK |
| Reviewer | code-review |
| Review date | YYYY-MM-DD |

Checks:
- [ ] Purpose matches governing plan/spec.
- [ ] Layer boundaries are preserved.
- [ ] Authorization and scope behavior are bounded.
- [ ] Inputs are validated and errors are typed.
- [ ] Mutations are audited or explicitly non-mutating.
- [ ] Rollback/snapshot impact is understood.
- [ ] Tests or documented verification cover the behavior.
- [ ] File-size and modularity rules are respected.
- [ ] No secrets, raw SQL exposure to agents, or broad agent powers are introduced.

Findings:
- `NONE`
```

## Area Review Summary

Each area review must end with this format:

```markdown
## CODE-REVIEW-[AREA]

| Field | Value |
| --- | --- |
| Area | Phase N - Name |
| Inventory source | Metadata registry, phase plan, current diff, manual expansion |
| File records complete | yes/no |
| Files approved | count |
| Files accepted with risk | count |
| Open blockers | count |
| Open required changes | count |
| Static checks cited | commands and results |
| Recommendation | DONE / CHANGES_REQUIRED / BLOCKED / PENDING |
| Next gate | QA Test / owner fix / architect decision / etc. |
```

## Exclusion Rule

Generated, vendor, cache, and build-output files can be excluded only with a
recorded reason. `frontend/node_modules/**`, `target/**`, `target-quality/**`,
and `*.tsbuildinfo` are excluded from review and must not be used as approval
evidence. Lockfiles are not excluded.

## Board Update Rule

The temporary board may link to a code-review summary, but the summary is not
enough by itself. The board cell moves to `DONE` only after the file records in
the ledger prove complete coverage.
