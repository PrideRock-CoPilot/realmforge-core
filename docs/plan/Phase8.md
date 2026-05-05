---
doc_id: DOC-PLAN-P8
title: Phase 8 — Live Watch
parent: DOC-PLAN-INDEX
status: draft
owner: backend
reviewers: [cto, security-architect, biz-user, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-LIVE-WATCH-001]
related_decision_ids: [DEC-USER-006]
related_file_ids: [FILE-CRATE-LIVE-WATCH-LIB]
visual_node_ids: [VN-MODULE-LIVE-WATCH]
approval_state: pending
---

# Phase 8: Live Watch

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 8 |
| Title | Live Watch |
| Work paths | `WP-LIVE-WATCH-001` |
| Product module | Live Watch |
| Owner | backend |
| Risk | medium |
| Decision blockers | `DEC-USER-006` (Live Watch auto-remediation authority) |

**Mandate:** Build the production monitoring and proposed remediation system. Live Watch monitors live runtime health signals, detects anomalies, and proposes remediation work packets. It does **not** auto-remediate in the first governed release.

⚠️ **DECISION BLOCKER:** `DEC-USER-006` must be resolved before Live Watch can auto-remediate low-risk issues. The phase proceeds with monitoring and proposing only.

---

## Workflow

```
CREATE NEW CRATE: live-watch

Crate dependency order:
  1. live-watch — monitoring engine, signal detection, remediation proposal
  2. authority-domain — add WatchSignal, RemediationProposal types
  3. control-store — add watch signal and remediation persistence
  4. control-service — add live_watch_service module
  5. control-api — add live watch route group
  6. operator-cli — add live-watch commands
  7. agent-mcp — add live-watch tools

Implementation order:
  a. Watch signal types and detection rules
  b. Signal collection from runtime health/bundle metrics
  c. Anomaly detection engine
  d. Remediation proposal generation (proposes packets, does not execute)
  e. Watch profiles — configurable signal thresholds per app
  f. Thin transports
```

---

## File Manifest

### live-watch (NEW CRATE)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `live-watch/Cargo.toml` | 20 | Dependencies: authority-domain, control-store, snapshot-ledger, chrono, serde, thiserror, tracing, tokio. |
| NEW | `live-watch/src/lib.rs` | 40 | Module declarations, `LiveWatchEngine` — main monitoring loop. `WatchConfig` — poll interval, signal thresholds, max proposals per window. |
| NEW | `live-watch/src/signals.rs` | 120 | `SignalCollector` — collects signals from runtime health (latency, error rate, action count), bundle health (version skew, artifact age), cost signals (spend rate, token usage). `AnomalyDetector` — detects threshold breaches, trend deviations, missing heartbeats. |
| NEW | `live-watch/src/remediation.rs` | 100 | `RemediationGenerator` — creates work packet proposals from detected anomalies. `propose_remediation(signal, app_scope)` — generates `RemediationProposal` with severity, impact analysis, proposed actions. Does **not** execute. Returns proposal for human or Board approval. |
| NEW | `live-watch/src/profile.rs` | 80 | `WatchProfile` — per-app configuration: signal thresholds, severity mappings, notification channels, max remediation proposals per day. `ProfileManager` — CRUD for watch profiles. |

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `authority-domain/src/live_watch.rs` | 80 | `WatchSignal` — signal_id, app_id, signal_type (Latency, ErrorRate, ActionCount, VersionSkew, CostRate, MissingHeartbeat), value, threshold, severity, timestamp. `RemediationProposal` — proposal_id, app_id, triggering_signal_ids, proposed_actions (list of scoped commands), impact_analysis, status (Proposed, Approved, Rejected, Executed). |
| UPDATE | `authority-domain/src/lib.rs` | 5 | Export `live_watch` module. |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 80 | `insert_watch_signal()`, `query_signals(app_id, time_range, severity)`. `insert_remediation_proposal()`, `list_proposals(app_id, status)`, `update_proposal_status()`. `get_watch_profile(app_id)`, `upsert_watch_profile()`. |

### control-service

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/src/live_watch_service.rs` | 120 | `start_monitoring(app_id)` — starts background signal collection loop. `stop_monitoring(app_id)` — gracefully stops. `get_signals(app_id, filters)` — query recent signals. `propose_remediation(app_id)` — trigger anomaly scan → generate proposals. `list_proposals(app_id, status)` — review pending proposals. `approve_proposal(proposal_id)` — routes approved proposal to gateway for execution. |
| UPDATE | `control-service/src/lib.rs` | 5 | Declare `live_watch_service` module. |

### control-api

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-api/src/routes/live_watch.rs` | 80 | `POST /v1/live-watch/start`, `POST /v1/live-watch/stop`, `GET /v1/live-watch/signals`, `POST /v1/live-watch/propose`, `GET /v1/live-watch/proposals`, `POST /v1/live-watch/proposals/:id/approve`. |
| EXTEND | `control-api/src/routes/mod.rs` | 5 | Register live_watch route group. |

### operator-cli

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `operator-cli/src/commands/live_watch.rs` | 80 | `live-watch start`, `live-watch stop`, `live-watch signals`, `live-watch propose`, `live-watch proposals`, `live-watch approve`. |
| EXTEND | `operator-cli/src/commands/mod.rs` | 5 | Register live_watch command module. |

### agent-mcp

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `agent-mcp/src/tools/live_watch.rs` | 60 | `core_get_watch_signals`, `core_propose_remediation`, `core_list_proposals`. |
| EXTEND | `agent-mcp/src/tools/mod.rs` | 5 | Register live_watch tool module. |

---

## Key Types

### WatchSignal (NEW)
```rust
pub enum SignalType {
    Latency, ErrorRate, ActionCount,
    VersionSkew, ArtifactAge,
    CostRate, TokenUsage,
    MissingHeartbeat,
}

pub struct WatchSignal {
    pub id: WatchSignalId,
    pub app_id: AppId,
    pub signal_type: SignalType,
    pub value: f64,
    pub threshold: f64,
    pub severity: WatchSeverity,
    pub timestamp: DateTime<Utc>,
}

pub struct RemediationProposal {
    pub id: ProposalId,
    pub app_id: AppId,
    pub triggering_signal_ids: Vec<WatchSignalId>,
    pub proposed_actions: Vec<ProposedAction>,
    pub impact_analysis: String,
    pub status: ProposalStatus,   // Proposed, Approved, Rejected, Executed
}
```

---

## Completion Gates

- [ ] `TEST-LIVE-WATCH-001` — Live Watch proposes remediation packet for repeated latency signal
- [ ] Signal collection runs on configurable interval
- [ ] Anomaly detection fires on threshold breach and trend deviation
- [x] Remediation proposals include impact analysis (never auto-execute)
- [x] Proposals can be approved via API, which routes to gateway for execution
- [x] Watch profiles can be created, updated, and applied per app
- [x] No auto-remediation occurs without `DEC-USER-006` resolution
- [x] API, CLI, and MCP surfaces all functional
- [ ] `cargo test --workspace` passes with 0 failures
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [x] `cargo check --workspace` passes with 0 errors
- [x] Live Watch domain types defined (SignalType, SignalSeverity, WatchSignal, RemediationProposal, WatchProfile, SignalThreshold)
- [x] DB migration created (008_live_watch.sql with watch_signals, remediation_proposals, watch_profiles tables)
- [x] control-store persists signals, proposals, and profiles
- [x] control-service LiveWatchService wired into ServiceContext
- [x] control-api routes registered (10 endpoints: start, stop, record_signal, get_signals, propose, list_proposals, approve, get_profile, update_profile)
- [x] operator-cli subcommands registered (6 commands: start, stop, signals, propose, proposals, approve)
- [x] agent-mcp tools registered (4 tools: core_get_watch_signals, core_propose_remediation, core_list_proposals, core_approve_proposal)
- [ ] Live Watch engine cycle (collect → detect → propose) functional

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-WATCH` | Create and modify live-watch crate |
| `SGL-BACKEND-DOMAIN` | Add live_watch types to authority-domain |
| `SGL-BACKEND-SERVICE` | Add live_watch_service to control-service |
| `SGL-BACKEND-API` | Add live_watch route group |
| `SGL-BACKEND-CLI` | Add live-watch commands |
| `SGL-SECURITY-REVIEW` | Review remediation proposal scope and auto-execution safety |

---

## Dependencies

- Phase 7 complete (Live Runtime — Live Watch monitors runtime health signals)
- `DEC-USER-006` resolved (for auto-remediation authority)
