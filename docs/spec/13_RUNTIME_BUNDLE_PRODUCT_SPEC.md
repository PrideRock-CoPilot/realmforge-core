---
doc_id: DOC-SPEC-013
title: Runtime Bundle Product Spec
status: draft
owner: cto
reviewers: [release-manager, security-architect, data-architect, backend]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: runtime-bundle
work_path_ids: [WP-RUNTIME-BUNDLE-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-MODULE-RUNTIME-BUNDLE]
visual_edge_ids: []
approval_state: pending
---

# Runtime Bundle Product Spec

## Responsibility

Runtime Bundle Builder creates signed governed app bundles from approved app state.

## Bundle Layout

```text
dist/bundles/{bundle_id}/
  manifest/bundle_manifest.json
  manifest/signature.sig
  compiled/
  runtime_state/parquet/
  policies/
  contracts/
  traces/
  evidence/
  rollback/
  docs/
```

## Boot Refusal Rules

Live Runtime refuses a bundle if:

- Signature is missing or invalid.
- Manifest hash fails.
- Required Parquet dataset is missing.
- Policy table is missing a route policy.
- Contract schema version is incompatible.
- Handler binding references an unregistered compiled handler.
- Bundle is not approved for the app and tenant.

## Bundle Manifest Fields

`bundle_id`, `app_id`, `tenant_id`, `source_snapshot_id`, `build_packet_id`, `created_at`, `created_by_actor_id`, `compiled_artifacts`, `runtime_datasets`, `policy_hash`, `contract_hash`, `trace_profile_hash`, `evidence_hash`, `rollback_manifest_hash`, `signature_algorithm`, `signature`.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-RUNTIME-BUNDLE-001` |
| Visual node IDs | `VN-MODULE-RUNTIME-BUNDLE` |
| Visual edge IDs | `VE-RUNTIME-BUNDLE-TO-LIVE-RUNTIME` |
