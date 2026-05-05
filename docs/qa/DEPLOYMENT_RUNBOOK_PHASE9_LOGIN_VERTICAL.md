---
doc_id: DOC-QA-001
title: "Deployment Runbook — Phase 9 / Login Vertical"
status: accepted
owner: release-manager
reviewers: [qa, infra-architect, backend]
created_at: 2026-05-05
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: login
work_path_ids: [WP-LOGIN-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# DEPLOYMENT RUNBOOK — Phase 9 / Login Vertical to Development

**Prepared by:** Sam Osei, Release Manager  
**Date:** 2026-05-04  
**Build:** RealmForge Core — Phase 9 (Login Vertical)

---

## QA SIGN-OFF

- **Certified by:** Margaret Thompson, QA Lead
- **Certification date:** 2026-05-04
- **Certification document:** `docs/qa/RELEASE_CERTIFICATION_PHASE9_LOGIN_VERTICAL.md`
- **Status:** CERTIFIED FOR RELEASE

---

## PRE-FLIGHT CHECKLIST

- [ ] Staging environment verified to match production config
- [ ] PostgreSQL migration `009_login_vertical.sql` reviewed and tested
- [ ] Secrets and environment variables confirmed in place
  - Note: `subtle` feature flag needs to be enabled for constant-time credential comparison
- [ ] Monitoring and alerting active
- [ ] Rollback window confirmed with stakeholders
- [ ] On-call rotation notified
- [ ] Deployment window confirmed (NOT Friday after 2pm)
- [ ] Migration idempotency verified — all queries use `INSERT ... ON CONFLICT DO UPDATE` or guarded DDL

The pre-flight checklist must be completed before proceeding to deployment.

---

## DEPLOYMENT STEPS

### Step 1: Apply database migration

| Field | Value |
|-------|-------|
| Action | Execute `db/migrations/009_login_vertical.sql` against PostgreSQL |
| Success criteria | Migration completes without errors. Tables `login_attempts`, `login_blocks`, `login_policies`, `stored_credentials` exist. |
| Rollback if failed | See Rollback Procedure (Step R1) |
| Estimated duration | 2 seconds |

Tables created:
- `login_attempts` — actor_id, tenant_id, timestamp, outcome
- `login_blocks` — actor_id, tenant_id, blocked_at, blocked_until, reason (PK on actor_id + tenant_id)
- `login_policies` — tenant_id, config_json (PK on tenant_id)
- `stored_credentials` — actor_id, tenant_id, credential_hash (PK on actor_id + tenant_id)

Seed data inserted:
- Default credential for `actor-GARY-001` with known SHA-256 hash

### Step 2: Build and deploy updated crate binaries

| Field | Value |
|-------|-------|
| Action | Execute `cargo build --release --features authority-domain/subtle` |
| Success criteria | All 6 modified crates compile without errors or warnings |
| Rollback if failed | Revert to previous build artifacts |
| Estimated duration | 5-10 minutes |

Affected crates:
- `authority-domain` (new: login types, login_policy, security features)
- `control-store` (extended: login persistence functions)
- `control-service` (new: login_handler.rs, extended: error types)
- `control-api` (new: login routes)
- `operator-cli` (new: login command)
- `agent-mcp` (new: login tool)

### Step 3: Run post-deployment migration verification

| Field | Value |
|-------|-------|
| Action | Verify 4 tables exist and seed data is queryable: `SELECT * FROM login_policies; SELECT * FROM stored_credentials;` |
| Success criteria | Both queries return expected rows |
| Rollback if failed | See Rollback Procedure (Step R1) |
| Estimated duration | 30 seconds |

### Step 4: Verify health endpoints

| Field | Value |
|-------|-------|
| Action | `GET /v1/health`, `GET /v1/health/ready`, `GET /v1/health/live` via control-api |
| Success criteria | All 3 endpoints return HTTP 200 |
| Rollback if failed | See Rollback Procedure |
| Estimated duration | 30 seconds |

### Step 5: Verify login endpoint

| Field | Value |
|-------|-------|
| Action | `POST /v1/login` with valid test credentials |
| Success criteria | Returns 200 with session_token, actor_id, scope, expires_at, audit_event_id |
| Rollback if failed | See Rollback Procedure |
| Estimated duration | 5 seconds |

---

## ROLLBACK PROCEDURE

> **Written first. Rollback is never an afterthought.**

### Rollback steps

#### Step R1: Roll back database migration

| Action | `DROP TABLE IF EXISTS stored_credentials, login_blocks, login_policies, login_attempts CASCADE;` |
|--------|--------------------------------------------------------------------------------------------------|
| Success criteria | All 4 tables removed. `\dt` shows no login-related tables. |
| Verification | `SELECT * FROM information_schema.tables WHERE table_name LIKE 'login%';` returns 0 rows |
| Estimated duration | 2 seconds |

#### Step R2: Roll back binary

| Action | Revert to previous build artifacts (restore from backup) |
|--------|-----------------------------------------------------------|
| Success criteria | Previous versions of all 6 crates are running |
| Verification | Run health check + existing integration tests |
| Estimated duration | 5 minutes |

### Rollback trigger conditions

Roll back immediately if any of:
- Migration fails or produces errors
- Health endpoints return non-200 after deployment
- Login endpoint returns 500 or unexpected errors with valid credentials
- Error rate exceeds 5% above baseline for 2 minutes
- Any of the existing integration test suites fail post-deployment (session lifecycle, command lifecycle, audit)

### Total estimated rollback time: 8 minutes (worst case)

---

## POST-DEPLOYMENT VERIFICATION

- [ ] Health check endpoints return expected status (200 OK)
- [ ] Login flow: valid credentials → session token + audit event recorded
- [ ] Login flow: invalid credentials → `InvalidCredentials` error (not account enumeration)
- [ ] Login flow: rate limited request → `RateLimited` error
- [ ] All existing session lifecycle operations work (issue, validate, revoke)
- [ ] All existing command lifecycle operations work (propose, authorize, apply)
- [ ] Audit chain integrity verified
- [ ] Error rates within baseline
- [ ] Latency within baseline
- [ ] No unexpected alerts triggered

---

## DEPLOYMENT HOLD WINDOW

**Do not declare success for 30 minutes** after the last deployment step.

Rollback remains available until: 30 minutes after step 5 completion.

---

## STATUS

**⬜ APPROVED FOR DEPLOYMENT** (pre-flight pending execution)

*Prepared by,*  
**Sam Osei**  
Release Manager, RealmForge  
2026-05-04
