# Focused Workflow Lifecycle

Every task moves through the same governed path:

```text
Idea -> Planning -> Architecture -> Development -> Peer Review -> Code Review -> Testing -> Documentation
```

Each stage is as important as the next. Do not treat planning, architecture, testing, or documentation as optional ceremony around development.

## Core Rule

Before acting, identify the current lifecycle stage and the next required handoff. If the user asks for a later stage directly, check whether earlier-stage artifacts already exist. If they do not, create or request the missing earlier artifact first.

Small tasks may compress stages into one response, but they may not skip stages. If a stage is not applicable, state `N/A` with a reason.

## Stage Gates

| Stage | Owner skill | Required artifact | Exit gate |
| --- | --- | --- | --- |
| Idea | `biz-user`, `pm` | Problem statement, user outcome, constraints, non-goals | The request is clear enough to plan or has named open questions. |
| Planning | `pm`, `orchestrator` | Work slice, scope boundary, acceptance criteria, handoff map | The next implementable slice is named and blockers are visible. |
| Architecture | `cto` plus domain architect as needed | Boundary contract, affected crates/files, rollback path, risks | Layer law is satisfied and unresolved decisions are recorded. |
| Development | `backend`, `frontend`, or `data-engineer` | Focused implementation diff within approved files | Code follows docs-first law and does not expand scope silently. |
| Peer Review | `peer-review` | Area readiness evidence and next-gate recommendation | Work is coherent enough to enter file-level code review. |
| Code Review | `code-review` | File review records and area code-review recommendation | Every in-scope file has a review record and no blocking finding remains. |
| Testing | `qa` | Test results, acceptance evidence, defects or residual risk | Required gates are run or explicitly blocked. |
| Documentation | `tech-writer`, `pm`, `qa` | Updated specs, phase gates, roadmap notes, verification evidence | Docs match implementation and do not over-certify. |

## Workflow Discipline

- Keep one active lifecycle stage at a time unless explicitly mapping a plan.
- State the current stage in status updates for multi-step work.
- Never start development before planning and architecture gates are satisfied.
- Never mark a task complete before peer review, code review, testing, and documentation gates are satisfied.
- If implementation exposes a missing decision, stop development and record it in `docs/spec/22_OPEN_DECISIONS.md`.
- If testing fails, return to Development with a named defect and expected fix.
- If documentation exposes a mismatch, return to the earliest stage where the mismatch began.

## Stage Deliverables

Idea deliverable:

- User outcome in one sentence
- Main user or system actor
- Success signal
- Known constraints
- Open questions

Planning deliverable:

- Work path or phase reference
- In-scope and out-of-scope boundaries
- Acceptance criteria
- Required skills and handoffs
- Blockers and decisions

Architecture deliverable:

- System boundary
- Affected crates, docs, contracts, migrations, or UI surfaces
- Layer-law check
- Security and rollback considerations
- Decision register updates when needed

Development deliverable:

- Smallest coherent implementation slice
- No broad refactors unless planned
- No unregistered files
- Typed errors instead of incomplete production stubs
- Tests added or updated alongside behavior

Testing deliverable:

- Commands run
- Acceptance test IDs covered
- Pass/fail result
- Unverified areas
- Defects with owner and reproduction

Documentation deliverable:

- Specs, metadata, roadmap, and phase plans updated for changed behavior
- Verification note with exact date when gates are claimed
- Remaining unchecked gates left unchecked
- Final response names the next honest step

## Handling User Shortcuts

If the user says "just build it":

1. Check whether the plan, architecture, metadata, and tests already exist.
2. If they exist, proceed to Development.
3. If they do not exist, produce the missing Planning or Architecture artifact first.

If the user says "what is next":

1. Read the roadmap and phase plan.
2. Identify the earliest open gate that is not blocked.
3. Confirm the lifecycle stage for that gate.
4. Execute the next stage, not a later one.

If the user asks for cleanup:

1. Follow `06-repo-hygiene-and-verification.md`.
2. Do not delete or revert user work.
3. Update docs if cleanup changes build, verification, or roadmap status.

## Final Response Shape

For non-trivial work, the final response should report:

- Lifecycle stage completed
- Artifact produced
- Verification evidence
- Docs updated
- Next lifecycle stage
- Any blocker or skipped stage with reason
