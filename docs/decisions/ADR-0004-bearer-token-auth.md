---
doc_id: DOC-ADR-0004
title: "ADR-0004: Bearer Token as Frontend Authentication Mechanism"
status: accepted
owner: security-architect
reviewers: [cto, security-architect, frontend, backend]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: auth
work_path_ids: [WP-LOGIN-001, WP-BOARDS-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-API-LIB, FILE-CRATE-SERVICE-LIB]
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# ADR-0004: Bearer Token as Frontend Authentication Mechanism

**Date:** 2026-05-04
**Status:** Accepted
**Deciders:** Fatima Al-Hassan (Security Architect)
**Consulted:** Rena Okafor (CTO), Kai Sato (Frontend), Dmitri Volkov (Backend), Marcus Webb (API Architect)

## Context

The frontend `client.ts` must authenticate API requests. Two conventions existed in the
codebase: a custom `X-Session-Id` header (implicit prior convention, never formally decided)
and the RFC 6750 `Authorization: Bearer <token>` standard. The `POST /v1/login` response
already returns a `session_token` field distinct from the internal `session_id` DB key.
A formal decision was needed before Kai could implement `client.ts`.

## Decision Drivers

- Security tooling (OWASP ZAP, Burp, WAF middleware) must be able to identify the auth credential
- OpenAPI security scheme declarations require a recognized auth mechanism
- The `session_token` and `session_id` are distinct: one is a client credential, one is a DB key
- Authorization header content must never appear in server logs

## Options Considered

### Option A: Custom X-Session-Id header

Pass the session ID as a custom request header. The backend looks it up in the session store.

- Pro: Readable and debuggable. Avoids confusion with OAuth Bearer flows.
- Con: Not a recognized auth header scheme. Invisible to API gateways, WAFs, security scanners,
  OpenAPI security declarations, and Swagger UI. Every middleware layer must be custom-configured
  to treat it as an auth credential. CSRF analysis is non-standard.

### Option B: Authorization: Bearer (RFC 6750)

The `session_token` returned by `POST /v1/login` is the bearer credential. The frontend sends
`Authorization: Bearer <session_token>` on every protected request. Backend middleware extracts
the token, validates it against the session store, and populates request context with the resolved
`ActorScope`.

- Pro: RFC 6750 standard. Recognized by OpenAPI security schemes, API gateways, WAFs, Postman,
  Swagger UI, and all security scanning tooling.
- Pro: Cleanly separates `session_id` (internal DB key, never leaves server) from `session_token`
  (client-facing bearer credential).
- Con: Bearer tokens must not be logged. Requires explicit Authorization header redaction in
  Axum's `TraceLayer` configuration.

## Decision

`Authorization: Bearer <session_token>` is the authentication mechanism for all frontend API
requests. The `session_token` field from `POST /v1/login` is the bearer credential.
`X-Session-Id` is retired as an auth header convention.

## Rationale

Fatima's threat model requires that auth credentials be identifiable to standard middleware
stacks without custom configuration. `X-Session-Id` cannot satisfy this requirement — it is
opaque to every layer that does not know about it in advance. RFC 6750 Bearer tokens are the
established standard for this exact use case. The `session_token`/`session_id` distinction is
correct domain modeling: the token is what the client holds; the ID is a DB reference that
never leaves the server.

## Consequences

**Positive:**
- OpenAPI spec can declare a Bearer security scheme — Swagger UI gets a working "Authorize" button
- API gateway and WAF auth middleware works without custom configuration
- Security scanners correctly identify and test the auth mechanism
- `session_id` remains an internal server concept; the client never sees it

**Negative / Trade-offs:**
- Dmitri must add Bearer token extraction middleware to `control-api` that validates the token
  against the session store and populates request extensions with `ActorScope`
- The `Authorization` header must be redacted in Axum's `TraceLayer` to prevent token leakage in logs

**Risks:**
- Bearer token leakage in logs is a P1 security incident. The TraceLayer redaction must be
  verified before any log aggregation is enabled in a non-local environment.
- Token expiry must be handled gracefully in the frontend — expired tokens must trigger a
  re-login flow, not a silent 401 that leaves the user in a broken state.

## Review Date

2027-05-04, or when a more granular scope/claims mechanism (e.g., JWT with embedded scope)
is considered.
