---
doc_id: DOC-SPEC-019
title: First Vertical Login Module
status: draft
owner: pm
reviewers: [biz-user, security-architect, api-architect, backend, frontend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: login-module
work_path_ids: [WP-LOGIN-001]
related_decision_ids: []
related_file_ids: [FILE-CATALOG-LOGIN-MODULE, FILE-CONTRACT-LOGIN, FILE-CONTRACT-LOGIN-RESPONSE, FILE-POLICY-LOGIN, FILE-HANDLER-LOGIN, FILE-WATCH-LOGIN]
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
| `FILE-CATALOG-LOGIN-MODULE` | `catalog/Login/catalog.json` | `runtime_definition` | `TEST-LOGIN-MODULE-001` |
| `FILE-CONTRACT-LOGIN` | `catalog/Login/contracts/login_request.json` | `runtime_contract` | `TEST-LOGIN-CONTRACT-001` |
| `FILE-CONTRACT-LOGIN-RESPONSE` | `catalog/Login/contracts/login_response.json` | `runtime_contract` | `TEST-LOGIN-CONTRACT-002` |
| `FILE-POLICY-LOGIN` | `catalog/Login/policy/login_policy.json` | `runtime_policy` | `TEST-LOGIN-POLICY-001` |
| `FILE-HANDLER-LOGIN` | `crates/control-service/src/login_handler.rs` | `compiled_source` | `TEST-LOGIN-HANDLER-001` |
| `FILE-WATCH-LOGIN` | `catalog/Login/watch_profile.json` | `runtime_trace_profile` | `TEST-LOGIN-WATCH-001` |

## Contracts

Login request fields: `tenant_id`, `project_id`, `actor_id`, `credential`, `scope`.

Login response fields: `session_token`, `actor_id`, `scope`, `expires_at`, `audit_event_id`, `snapshot_id`.

EntraID extension fields: `entra_tenant_id`, `client_id_reference`, `scopes`, `claims_policy_id`. Secrets are references only.

## Required Trace Points

`TRACE-LOGIN-REQUEST-RECEIVED`, `TRACE-LOGIN-CONTRACT-VALIDATED`, `TRACE-LOGIN-POLICY-CHECKED`, `TRACE-LOGIN-SNAPSHOT-ANCHORED`, `TRACE-LOGIN-SESSION-CREATED`, `TRACE-LOGIN-AUDIT-WRITTEN`, `TRACE-LOGIN-RESPONSE-RETURNED`.

## Acceptance

The module is accepted when a tenant copies it from global catalog, an app creates a Login work path, agents receive scoped packets, tests emit evidence, a signed bundle is built, Live Watch records login traces, and rollback preview can restore the previous app state.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-LOGIN-001` |
| Visual node IDs | `VN-MODULE-LOGIN` |
| Visual edge IDs | `VE-LOGIN-USES-AUTHORITY`, `VE-LOGIN-EMITS-LIVE-WATCH` |
