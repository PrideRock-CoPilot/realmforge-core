---
doc_id: DEC-COUNCIL-RFSOURCE-003
title: RFSource Source Control Feature Release Approval
status: draft
owner: council
reviewers: [cto, security-architect, qa, pm, backend]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: rfsource
work_path_ids: [WP-RFSOURCE-002]
related_decision_ids: [DEC-USER-RFSOURCE-001, DEC-COUNCIL-RFSOURCE-001, DEC-COUNCIL-RFSOURCE-002]
related_file_ids: [FILE-CRATE-RFSOURCE-STORE, FILE-CRATE-RFSOURCE-SERVICE, FILE-CRATE-RFSOURCE-CATALOG]
visual_node_ids: [VN-RFSOURCE-COUNCIL-RELEASE]
visual_edge_ids: []
approval_state: accepted
---

# DECISION RECORD — RFSource Source Control Feature Release

**Council session date:** 2026-05-06
**Participants:**
- Rena Okafor (CTO) — DRI
- Fatima Al-Hassan (Security Architect)
- Margaret Thompson (QA Lead)
- Alex Rivera (PM)
- Dmitri Volkov (Backend)

**DRI (Directly Responsible Individual):** Rena Okafor, CTO

─────────────────────────────────────────

## Decision Question

Should the Council approve the RFSource source control feature for release, given completed peer review (Nora Patel), code review (Owen Brooks), QA certification (Meg Thompson, 2026-05-06), resolved threat model findings (Fatima Al-Hassan, DOC-SPEC-023), and all quality gates passing?

## Options Considered

### Option A: APPROVE for release

The feature passes all quality gates:
- 28 unit tests passing
- cargo fmt + clippy + test all green
- Peer review: DONE — one finding (file-size) resolved
- Code review: DONE — all 9 files APPROVED
- QA certification: CERTIFIED FOR COUNCIL — all 15 acceptance tests passed
- Threat model: 10 findings — 2 critical FIXED, 1 partial, 7 documented as known limitations

**Proponent:** Dmitri Volkov (Backend)
**Arguments for:** All required gates are satisfied. Known limitations are documented with named status. The feature is complete within its defined scope.

### Option B: HOLD for remediation

Require one or more of the following before release:
1. Time warp scope GOVERN gate (F-006 partial) — implement scope-level grant checking
2. Proposal apply atomicity (F-007) — implement atomic frame writes or WAL checkpoint
3. Transport-layer authentication (F-003) — implement session-based actor identity

**Proponent:** Fatima Al-Hassan (Security Architect)
**Arguments for:** F-003 is a critical STRIDE finding. Without transport auth, the `actor` field is trusted input. Holding ensures the security baseline is met before adoption.
**Challenges raised:** Scope was always documented as transport-layer deferred. Grant checks provide defense-in-depth at the store layer. Holding would block all downstream RFSource adoption without clear benefit for MVP — the `.rfsource` format is local-file-first.

## Decision

**APPROVED for release** — with the following conditions:

1. All documented known limitations (F-003, F-004, F-006, F-007, F-008, F-009, F-010) remain open and tracked in the threat model.
2. The file-size debt on `source_control.rs` (1202 lines, exceeding 500-line cap) is accepted with the documented refactor plan. Refactor to per-operation files before the next feature addition.
3. No further feature additions to the RFSource source control surface until the file-size refactor is completed.

## Rationale

- All defined gates are green. Peer review, code review, and QA are complete with reproducible evidence.
- The two critical threat model findings (F-001, F-002) are fixed. No blocking security finding remains unaddressed in the store layer.
- Known limitations are documented with named deferred status in DOC-SPEC-023 and the QA certification.
- Holding for F-003 would impose a transport-layer requirement on a storage-layer feature, blurring the Layer Law boundary. Correct layering is to ship with documented risk and add transport auth when the API/MCP surface is built.
- Holding for F-006/F-007 would improve but not transform the security posture for MVP. These are correctly deferred to the post-refactor phase.

## Dissenting Opinions

- **Fatima Al-Hassan (Security Architect):** "I accept the decision but note that the residual risk of unbounded actor identity means all grant-level controls are advisory, not enforceable, until transport-layer authentication is implemented. I recommend the Council revisit this decision when the API/MCP surface for RFSource is specified."

- **Margaret Thompson (QA Lead):** "The proposal atomicity gap (F-007) is a real crash-safety concern. I certify because the risk is bounded and documented, but I recommend this be elevated to 'Required' before the feature is used in production with concurrent writers."

## Consequences

**Positive:** RFSource source control feature is released. Downstream consumers (time-warp tool, branch management, proposal workflows) can proceed. The artifact registry and governance layers are now end-to-end usable.

**Negative:** The file-size debt on `source_control.rs` must be resolved before the next feature addition. The known limitations constrain production readiness — this is an MVP release.

**Risks:**
- Crash during `apply_proposal` can leave orphaned bundles (F-007) — mitigated by manual recovery
- Unauthenticated `actor` field (F-003) — mitigated by local-file trust model for MVP
- Project-wide time warp does not require GOVERN grant (F-006) — mitigated by per-artifact grant checks

## Follow-up

| Who | What | By when |
|-----|------|---------|
| Backend (Dmitri) | Refactor `source_control.rs` into per-operation files (`branch_ops.rs`, `proposal_ops.rs`, `comment_ops.rs`, `time_warp_ops.rs`) | Before next RFSource feature addition |
| Security (Fatima) | Revisit F-003 status when API/MCP surface is specified | When API/MCP transport layer is defined for RFSource |
| QA (Meg) | Elevate F-007 to "Required" before production deployment with concurrent writers | When concurrent-writer scenario is specified |

─────────────────────────────────────────

**STATUS: CLOSED — 2026-05-06**

**DRI: Rena Okafor, CTO**
