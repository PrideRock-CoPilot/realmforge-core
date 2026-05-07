---
doc_id: DEC-COUNCIL-WFE-001
title: "Workflow Engine Crate Formal Adoption Decision"
status: active
owner: council
reviewers: [cto, backend, peer-review, code-review, qa, pm, tech-writer]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: workflow-engine
work_path_ids: [WP-WF-001]
related_decision_ids: [DEC-CTO-WFE-001]
related_file_ids:
  [
    FILE-WFE-CORE,
    FILE-WFE-STAGE-TRAIT,
    FILE-WFE-STAGES,
    FILE-WFE-ERROR,
    FILE-WFE-TEST,
  ]
visual_node_ids: [VN-WORKFLOW-ENGINE]
visual_edge_ids: []
approval_state: accepted
---

# DECISION RECORD — Workflow Engine Crate Formal Adoption

**Council session date:** 2026-05-06
**Participants:**
- Rena (CTO) — architecture review completed 2026-05-06
- Dmitri (Backend) — implementation author
- Nora (Peer Review) — peer review lead
- Owen (Code Review) — code review lead
- Meg (QA) — QA lead
- Alex (PM) — planning and scope oversight
- Clara (Tech Writer) — documentation and ADR production [absent, notified]

**DRI (Directly Responsible Individual):** Rena (CTO) — architectural decision

---

## Decision Question

> **Should the `workflow-engine` crate (`crates/workflow-engine/`) be formally adopted into the RealmForge governed workspace with its current architecture, accepting the documented file-size condition on `orchestrator.rs` (582 lines > 500-line hard cap), so that integration work into transport crates can proceed?**

---

## Governance Chain Preceding This Session

| Gate | Skill | Status | Date |
|---|---|---|---|
| Architecture | CTO (Rena) | ✅ APPROVED with condition | 2026-05-06 |
| Peer Review | Nora Patel | ✅ PARTIAL → Code Review | 2026-05-06 |
| Code Review | Owen Brooks | ✅ DONE (0 blockers) | 2026-05-06 |
| QA Certification | Meg Thompson | ✅ CERTIFIED FOR RELEASE | 2026-05-06 |
| Council | The Decision Council | ⬅️ **THIS SESSION** | 2026-05-06 |

---

## Options Considered

### Option A: Adopt as-is with accepted file-size condition

**Description:** Formally adopt the workflow-engine crate with its current 14-file structure, including `orchestrator.rs` at 582 lines. The file-size finding is accepted as `ACCEPTED_WITH_RISK` by CTO, with the condition that it is refactored before the next feature addition to the crate.

**Proponent:** Dmitri (Backend) — implementation is complete, tested, and all quality gates pass.

**Arguments for:**
- All 226 workspace tests pass across 19 crates
- 55 tests specific to workflow domain (37 engine + 17 domain types)
- Cargo clippy: 0 warnings. Cargo fmt: clean.
- Layer law fully satisfied — workflow-engine depends only on authority-domain
- No IO, no transport concerns, no persistence coupling
- Stage agent pattern is consistent across all 10 agents (easy to maintain)
- File-size condition has clear, specific mitigation path (extract MockStageAgent)

**Challenges raised:** None. All gates have passed with this condition already known and accepted.

**Risk to Option A:** orchestrator.rs at 582 lines is a maintenance concern. The file is well-organized but large. Mitigation is concrete and understood.

### Option B: Require file-size refactor before adoption

**Description:** Block adoption until `orchestrator.rs` is refactored below 500 lines by extracting MockStageAgent into a test helper module and splitting execution methods into a separate file.

**Proponent:** None formally — this is the default "not yet" option.

**Arguments for:** Enforcement of architecture laws creates discipline.

**Arguments against:**
- Delays integration work in transport crates (control-api, agent-mcp, operator-cli)
- All gates (CTO, Peer Review, Code Review, QA) have accepted the risk
- The refactor is mechanical and can be completed in < 30 minutes when scheduled
- No production risk from the file size — the code is correct, tested, and reviewed

---

## Decision

**Option A: Adopt as-is with accepted file-size condition.**

The workflow-engine crate is formally adopted into the RealmForge governed workspace. All upstream gates are green. The file-size condition on `orchestrator.rs` is recorded as technical debt with a specific mitigation path.

## Rationale

The crate is production-ready by every measurable standard:
1. **All 11 acceptance criteria pass** (DOC-SPEC-027)
2. **All 6 quality gates pass** (check, test, clippy, fmt, no unsafe, typed errors)
3. **Every file has a code review record** — 16/16 files APPROVED
4. **QA issued CERTIFIED FOR RELEASE** — no defects found
5. **The single open finding is accepted-with-risk** by the CTO and carries a clear mitigation plan

Delaying adoption would block downstream integration work in the transport crates for no measurable quality gain.

## Conditions

1. The `orchestrator.rs` file-size refactor (extract MockStageAgent into a test helper module) must be completed before any new feature is added to the workflow-engine crate.
2. DOC-SPEC-027 must be promoted from `draft` to `active` status by Clara (Tech Writer).
3. File registry entries (FILE-WFE-CORE, FILE-WFE-STAGE-TRAIT, FILE-WFE-STAGES, FILE-WFE-ERROR, FILE-WFE-TEST) must be verified/created in `docs/spec/04_METADATA_STANDARD.md`.

## Dissenting Opinions

No dissenting opinions were recorded. All participating Council members agree on the decision.

## Consequences

**Positive:**
- Integration work in transport crates can proceed (control-api, agent-mcp, operator-cli)
- The workflow lifecycle agent system is now a governed part of the codebase
- Future development can add features to the engine within the governed framework
- The stage agent pattern sets a consistent template for all future lifecycle automation

**Negative/Constrained:**
- The file-size condition on orchestrator.rs must be resolved before new features
- The file registry entries need creation (documentation backlog)

**Risks:**
- orchestrator.rs will grow if the refactor is deferred beyond the first feature addition
- DOC-SPEC-027 remaining in `draft` status could cause confusion about spec authority

## Follow-up Actions

| Action | Owner | Due | Status |
|---|---|---|---|
| Extract MockStageAgent from `orchestrator.rs` into test helper module | Dmitri (Backend) | Before next feature addition to workflow-engine | ⬜ Pending |
| Promote DOC-SPEC-027 from `draft` to `active` | Clara (Tech Writer) | Documentation cycle 2026-05-06 | ✅ COMPLETE (2026-05-06) |
| Verify/create file registry entries for workflow-engine files | Clara (Tech Writer) | Documentation cycle 2026-05-06 | ✅ COMPLETE — 16 entries verified in docs/spec/04_METADATA_STANDARD.md |
| Integrate workflow-engine into transport crates (control-api, agent-mcp, operator-cli) | Dmitri (Backend) | Next phase | ⬜ Pending |

---

## ADR Reference

ADR pending — Clara (Tech Writer) will produce the formal Architecture Decision Record.

---

**STATUS: CLOSED — 2026-05-06**
