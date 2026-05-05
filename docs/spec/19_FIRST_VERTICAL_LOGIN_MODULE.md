---
doc_id: DOC-SPEC-019
title: First Vertical Login Module
status: draft
owner: pm
reviewers: [biz-user, security-architect, api-architect, backend, frontend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: login-module
work_path_ids: [WP-LOGIN-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-MODULE-LOGIN]
visual_edge_ids: []
approval_state: pending
---

# First Vertical Login Module

## Purpose

Login is the first end-to-end proof slice because it touches UI, API, contracts, policy, session creation, audit, traces, evidence, runtime bundle, Live Watch, and rollback.

## Module Contents

| File ID | Path | Artifact class | Required tests |
| --- | --- | --- | --- |
| `FILE-CATALOG-LOGIN-MODULE` | `catalog/global/modules/auth/login/module.yaml` | `runtime_definition` | `TEST-LOGIN-MODULE-001` |
| `FILE-CONTRACT-LOGIN` | `catalog/global/modules/auth/login/contracts/login.v1.json` | `runtime_contract` | `TEST-LOGIN-CONTRACT-001` |
| `FILE-POLICY-LOGIN` | `catalog/global/modules/auth/login/policies/login.policy.yaml` | `runtime_policy` | `TEST-LOGIN-POLICY-001` |
| `FILE-HANDLER-LOGIN` | `apps/{app_id}/handlers/auth/login.rs` | `compiled_source` | `TEST-LOGIN-HANDLER-001` |
| `FILE-WATCH-LOGIN` | `catalog/global/modules/auth/login/watch/login.watch.yaml` | `runtime_trace_profile` | `TEST-LOGIN-WATCH-001` |

## Contracts

Login request fields: `provider`, `redirect_uri`, `tenant_hint`, `nonce`, `state`.

Login response fields: `session_id`, `actor_id`, `expires_at`, `redirect_uri`, `audit_event_id`.

EntraID extension fields: `entra_tenant_id`, `client_id_reference`, `scopes`, `claims_policy_id`. Secrets are references only.

## Required Trace Points

`TRACE-LOGIN-REQUEST-RECEIVED`, `TRACE-LOGIN-CONTRACT-VALIDATED`, `TRACE-LOGIN-POLICY-CHECKED`, `TRACE-LOGIN-PROVIDER-REDIRECT`, `TRACE-LOGIN-CALLBACK-RECEIVED`, `TRACE-LOGIN-SESSION-CREATED`, `TRACE-LOGIN-AUDIT-WRITTEN`, `TRACE-LOGIN-RESPONSE-RETURNED`.

## Acceptance

The module is accepted when a tenant copies it from global catalog, an app creates a Login work path, agents receive scoped packets, tests emit evidence, a signed bundle is built, Live Watch records login traces, and rollback preview can restore the previous app state.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-LOGIN-001` |
| Visual node IDs | `VN-MODULE-LOGIN` |
| Visual edge IDs | `VE-LOGIN-USES-AUTHORITY`, `VE-LOGIN-EMITS-LIVE-WATCH` |
