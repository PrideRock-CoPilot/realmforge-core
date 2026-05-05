---
doc_id: DOC-PLAN-P7
title: Phase 7 — Runtime Bundle And Live Runtime
parent: DOC-PLAN-INDEX
status: draft
owner: cto
reviewers: [backend, security-architect, release-manager, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-RUNTIME-BUNDLE-001, WP-LIVE-RUNTIME-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-BUNDLE-LIB, FILE-CRATE-SERVICE-LIB]
visual_node_ids: [VN-MODULE-RUNTIME-BUNDLE, VN-MODULE-LIVE-RUNTIME]
approval_state: pending
---

# Phase 7: Runtime Bundle And Live Runtime

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 7 |
| Title | Runtime Bundle And Live Runtime |
| Work paths | `WP-RUNTIME-BUNDLE-001`, `WP-LIVE-RUNTIME-001` |
| Product module | Runtime Bundle, Live Runtime |
| Owner | cto (Dr. Rena Okafor) |
| Risk | critical |
| Decision blockers | none |

**Mandate:** Build the signed governed app bundle creation system and the governed execution engine. Runtime bundles are signed, versioned, and carry their governance manifest. The Live Runtime loads approved bundles and executes them within the governance framework.

---

## Workflow

```
CREATE NEW CRATE: runtime-bundle
CREATE NEW CRATE: live-runtime

Crate dependency order:
  1. runtime-bundle — bundle creation, signing, verification
  2. live-runtime — bundle loading, execution, governance enforcement
  3. authority-domain — add BundleManifest, RuntimeExecution types
  4. control-store — add bundle and runtime persistence
  5. control-service — add bundle_service and runtime_service modules
  6. control-api — add bundle and runtime route groups
  7. operator-cli — add bundle and runtime commands
  8. agent-mcp — add bundle and runtime tools

Implementation order:
  a. Bundle manifest types and signing
  b. Bundle creation pipeline (collect artifacts → build manifest → sign → package)
  c. Bundle verification (signature, integrity, governance match)
  d. Runtime execution engine (load → verify → execute in governed context)
  e. Runtime health and lifecycle management
  f. Thin transports
```

---

## File Manifest

### runtime-bundle (NEW CRATE)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `runtime-bundle/Cargo.toml` | 20 | Dependencies: serde, serde_json, sha2, ed25519-dalek, chrono, thiserror. |
| NEW | `runtime-bundle/src/lib.rs` | 60 | `BundleManifest` — bundle id, version, app id, governance refs, artifact hashes, signature, created_at. `BundlePackage` — manifest + signed artifact archive. |
| NEW | `runtime-bundle/src/builder.rs` | 120 | `BundleBuilder` — collect artifacts, compute content hashes, build manifest, sign with private key. `build_bundle(artifacts, governance_ref)` → `BundlePackage`. |
| NEW | `runtime-bundle/src/verifier.rs` | 100 | `verify_bundle(bundle)` — verify signature, verify all artifact hashes, verify governance integrity. `verify_deployment_readiness(bundle)` — check all required approvals exist. Returns `VerificationReport`. |
| NEW | `runtime-bundle/src/signing.rs` | 80 | `sign_manifest(manifest, private_key)` → signature. `verify_signature(manifest, public_key)` → bool. Key pair generation for development. |

### live-runtime (NEW CRATE)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `live-runtime/Cargo.toml` | 20 | Dependencies: runtime-bundle, authority-domain, policy-engine, audit-log, control-store, snapshot-ledger, tokio, tracing. |
| NEW | `live-runtime/src/lib.rs` | 60 | `LiveRuntime` — runtime instance with loaded bundle, governance context. `RuntimeStatus` — Loading, Running, Paused, Stopped, Failed. |
| NEW | `live-runtime/src/loader.rs` | 100 | `load_bundle(bundle_id, governance_ctx)` — verify bundle, extract artifacts, prepare execution sandbox. `unload_bundle(bundle_id)` — graceful shutdown and state preservation. |
| NEW | `live-runtime/src/executor.rs` | 120 | `execute_action(action, scope)` — run governed action through bundle's handler. Every execution goes through: policy check → audit event → snapshot anchor. `execute_packet(packet_id)` — execute work packet in runtime context. |
| NEW | `live-runtime/src/health.rs` | 60 | `runtime_health()` — status, uptime, active sessions, last action. `runtime_metrics()` — action count, error rate, latency. |

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `authority-domain/src/bundle.rs` | 80 | `BundleManifest` — bundle_id, version, app_id, artifact_hashes, governance_signature, release_approval_ref, created_at. `BundleStatus` — Building, Signed, Verified, Deployed, Running, Failed. |
| UPDATE | `authority-domain/src/lib.rs` | 5 | Export `bundle` module. |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 80 | `insert_bundle_manifest()`, `get_bundle(id)`, `list_bundles(app_id)`. `insert_runtime_instance()`, `update_runtime_status()`, `get_runtime_health()`. |

### control-service

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/src/bundle_service.rs` | 100 | `create_bundle(app_id, artifacts, governance_ref)` → builds, signs, and registers bundle. `verify_bundle(bundle_id)` → runs full verification. `list_bundles(app_id)` → version history. |
| NEW | `control-service/src/runtime_service.rs` | 100 | `deploy_bundle(bundle_id)` → load into runtime. `execute_runtime_action(action)` → governed execution. `get_runtime_status()` → health and metrics. `stop_runtime(bundle_id)` → graceful shutdown. |
| UPDATE | `control-service/src/lib.rs` | 10 | Declare `bundle_service` and `runtime_service` modules. |

### control-api

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-api/src/routes/bundle.rs` | 80 | `POST /v1/bundles`, `GET /v1/bundles`, `GET /v1/bundles/:id`, `POST /v1/bundles/:id/verify`, `POST /v1/bundles/:id/deploy`. |
| NEW | `control-api/src/routes/runtime.rs` | 60 | `GET /v1/runtime/status`, `POST /v1/runtime/action`, `POST /v1/runtime/stop`. |
| EXTEND | `control-api/src/routes/mod.rs` | 10 | Register bundle and runtime route groups. |

### operator-cli

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `operator-cli/src/commands/bundle.rs` | 80 | `bundle create`, `bundle list`, `bundle verify`, `bundle deploy`. |
| NEW | `operator-cli/src/commands/runtime.rs` | 80 | `runtime status`, `runtime action`, `runtime metrics`, `runtime stop`. |
| EXTEND | `operator-cli/src/commands/mod.rs` | 10 | Register bundle and runtime command modules. |

### agent-mcp

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `agent-mcp/src/tools/bundle.rs` | 60 | `core_create_bundle`, `core_verify_bundle`, `core_deploy_bundle`. |
| NEW | `agent-mcp/src/tools/runtime.rs` | 60 | `core_execute_runtime_action`, `core_get_runtime_status`. |
| EXTEND | `agent-mcp/src/tools/mod.rs` | 10 | Register bundle and runtime tool modules. |

---

## Key Types

### BundleManifest (NEW)
```rust
pub struct BundleManifest {
    pub bundle_id: BundleId,
    pub version: String,
    pub app_id: AppId,
    pub artifact_hashes: Vec<(String, String)>,   // (artifact_path, sha256)
    pub governance_signature: String,
    pub release_approval_ref: Option<ApprovalId>,
    pub built_at: DateTime<Utc>,
    pub status: BundleStatus,
}

pub enum BundleStatus {
    Building, Signed, Verified, Deployed, Running, Failed(String),
}
```

---

## Completion Gates

- [ ] `TEST-BUNDLE-001` — Runtime bundle refuses invalid signature
- [ ] Bundle creation pipeline produces valid signed manifest
- [ ] Bundle verification detects tampered artifacts
- [ ] Runtime deploys verified bundle and responds to actions
- [ ] Every runtime execution goes through policy check → audit → snapshot anchor
- [ ] Runtime health endpoint returns correct status and metrics
- [ ] API, CLI, and MCP surfaces all functional
- [ ] `cargo test --workspace` passes with 0 failures
- [ ] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-RUNTIME` | Create and modify runtime-bundle and live-runtime crates |
| `SGL-BACKEND-DOMAIN` | Add bundle types to authority-domain |
| `SGL-BACKEND-SERVICE` | Add bundle_service and runtime_service |
| `SGL-BACKEND-API` | Add route groups for both modules |
| `SGL-BACKEND-CLI` | Add command modules for both modules |
| `SGL-RELEASE` | Review bundle signing and release readiness |
| `SGL-SECURITY-REVIEW` | Review bundle signing and runtime governance |

---

## Dependencies

- Phase 6 complete (Boards and Build Watch — release commands from Boards trigger bundle builds)
- Phase 4 complete (Gateway — runtime actions go through policy enforcement)
