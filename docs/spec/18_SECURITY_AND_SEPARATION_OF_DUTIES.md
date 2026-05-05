---
doc_id: DOC-SPEC-018
title: Security And Separation Of Duties
status: draft
owner: security-architect
reviewers: [cto, qa, release-manager, backend]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: security
work_path_ids: [WP-SKILL-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-SECURITY-SOD]
visual_edge_ids: []
approval_state: pending
---

# Security And Separation Of Duties

## Threat Model

Primary risks:

- Agent edits outside approved scope.
- Agent obtains broader catalog knowledge than packet permits.
- Agent writes direct Postgres state.
- Agent hides incomplete evidence.
- Runtime runs unsigned or policy-missing bundle.
- Watch creates unauthorized remediation.

## Required Controls

- Gateway-only mutation.
- Postgres views for agent reads.
- Deny-list precedence for files.
- Skill grant TTL and budget.
- Audit hash chain for all mutations.
- Snapshot anchor before packet apply.
- Human approval for high-risk files, policy changes, migrations, and bundle activation.
- Separation of duties for implement, review, verify, release.

## High-Risk Artifact Classes

`migration`, `runtime_policy`, `runtime_contract`, `compiled_source` in policy/gateway/store/runtime crates, `operator_config`, bundle signatures.

## Release Safety Rule

No runtime bundle can become active until QA evidence, release manager approval, snapshot validation, signature validation, and rollback preview are all recorded.
