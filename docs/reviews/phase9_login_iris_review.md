---
doc_id: DOC-REVIEW-003
title: "Phase 9 Login Vertical — Biz User Review (Iris Park)"
status: accepted
owner: biz-user
reviewers: [pm, qa]
created_at: 2026-05-05
last_reviewed_at: 2026-05-05
source_of_truth: false
product_area: login
work_path_ids: [WP-LOGIN-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# Phase 9 Login Vertical — Biz User Review (Iris Park)

## User Stories

### Story 1: Operator Authentication

```
USER STORY — Operator Login
─────────────────────────────────────────
As an operator of RealmForge,
I want to authenticate with my credentials,
so that I can access governed operations within my authorized scope.

Acceptance Criteria:
  AC1: GIVEN I have valid credentials
       WHEN I submit them to the login endpoint
       THEN I receive a session token
       AND the token includes my authorized scope
       AND the token has an expiration time

  AC2: GIVEN I have submitted valid credentials
       WHEN I receive my session token
       THEN I can use that token to make subsequent governed requests
       AND those requests are associated with my identity

  AC3 (error): GIVEN I submit invalid credentials
       WHEN the system validates them
       THEN I receive a clear error message indicating authentication failed
       AND I am NOT given any session token

  AC4 (edge case): GIVEN I submit credentials that pass validation BUT exceed the rate limit
       WHEN the system checks the rate limit policy
       THEN I receive a message indicating I should wait before trying again
       AND I am NOT given any session token

Out of scope for this story:
  - Password reset flow
  - Multi-factor authentication
  - Self-service account registration
  - UI/login page — this is API-only for Phase 9

User assumptions:
  - The operator has already been provisioned with credentials administratively
  - The operator knows their credentials
  - The operator is using a CLI tool, API client, or MCP client to authenticate

Definition of done (from user's perspective):
  An operator who has credentials can authenticate and receive a usable session token,
  or receive a clear, specific message if authentication fails.
─────────────────────────────────────────
```

### Story 2: Policy-Governed Login Attempts

```
USER STORY — Rate Limited Login
─────────────────────────────────────────
As an operator,
I want the system to enforce login policies (rate limits, credential rules),
so that I am protected against brute-force attacks and the system remains secure.

Acceptance Criteria:
  AC1: GIVEN the system has a login policy that limits attempts to 5 per minute
       WHEN I submit 6 login attempts within one minute
       THEN the 6th attempt is rejected with a rate limit message
       AND I am told how long I should wait

  AC2: GIVEN my account has been temporarily blocked due to repeated failed attempts
       WHEN I attempt to log in during the block period
       THEN I receive a message indicating my account is temporarily blocked
       AND I am told when I can try again (or who to contact)

  AC3: GIVEN my block period has expired
       WHEN I attempt to log in with valid credentials
       THEN I am successfully authenticated
       AND I receive a valid session token

Out of scope:
  - Permanent account lockout
  - Admin unblock workflow (deferred)
  - Notification on block

User assumptions:
  - The operator can see the rate limit or block message
  - The operator understands they need to wait

Definition of done:
  An operator who exceeds policy limits receives a clear response explaining why
  they were rejected and what to do next.
─────────────────────────────────────────
```

### Story 3: Auditable Login Trail

```
USER STORY — Login Audit Trail
─────────────────────────────────────────
As a security operator or auditor,
I want every login attempt to be recorded in an audit trail,
so that I can investigate authentication events and detect anomalies.

Acceptance Criteria:
  AC1: GIVEN I (or any operator) submit a login attempt
       WHEN the attempt completes (success or failure)
       THEN an audit event is recorded
       AND the audit event includes: who attempted, when, and whether it succeeded

  AC2: GIVEN multiple login attempts have occurred
       WHEN I query the audit trail
       THEN I can see all login events in chronological order
       AND each event has a verifiable hash chain link

Out of scope:
  - Audit trail UI (API query only)
  - Real-time alerting on anomalies
  - Retention policies

User assumptions:
  - The auditor has access to query the audit trail
  - The auditor trusts the hash chain integrity

Definition of done:
  An auditor can query login events and verify the integrity of the audit trail.
─────────────────────────────────────────
```

### Story 4: System State Recovery (Rollback)

```
USER STORY — Login State Rollback
─────────────────────────────────────────
As a system operator,
I want the ability to roll back the system to a pre-login state,
so that I can recover from a bad deployment or incident.

Acceptance Criteria:
  AC1: GIVEN the system has processed login attempts and created state
       WHEN I request a snapshot of the current state
       THEN a snapshot is created capturing the login state

  AC2: GIVEN a snapshot exists
       WHEN I request a rollback to that snapshot
       THEN the system state is restored to what it was when the snapshot was taken
       AND the audit trail remains intact (rollback itself is audited)

  AC3: GIVEN a rollback has been performed
       WHEN I verify the system state
       THEN login attempts made after the snapshot are no longer present
       AND the system is in its previous consistent state

Out of scope:
  - Partial rollback (full system rollback only)
  - Rollback of individual login records
  - Granular undo

User assumptions:
  - The operator has permission to perform snapshots and rollbacks
  - The operator understands the scope of a rollback operation

Definition of done:
  An operator can create a snapshot and roll back to it, verifying the system
  returns to a consistent pre-login state.
─────────────────────────────────────────
```

## Iris's Verdict

These four stories cover the user-facing requirements for Phase 9 Login Vertical:

1. **Operator Authentication** — the core "I need to log in" story
2. **Policy-Governed Login** — "I need protection from brute force"
3. **Auditable Login Trail** — "I need to see what happened"
4. **System State Recovery** — "I need to undo if something goes wrong"

The acceptance criteria are written from observable user behavior — not system state.
A non-engineer can verify each one by observing the response received.

**One concern I raise:** The current spec defines the Login request as `provider`, `redirect_uri`, `tenant_hint`, `nonce`, `state` — these feel like OAuth/OIDC fields. If Phase 9 is the first vertical slice, I'd recommend starting simpler: `username/credential` + `scope`. The OAuth-compatible fields can come in a follow-up. This keeps the first end-to-end test focused on proving the governance loop works, not on proving OAuth compliance.

**Handoff from Iris Park:** These user stories and acceptance criteria are ready — sending to Alex (PM) for sprint planning, and to Dmitri (Backend) and Fatima (Security Architect) as context for their reviews.
