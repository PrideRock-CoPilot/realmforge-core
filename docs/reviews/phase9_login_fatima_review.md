# Phase 9 Login Vertical — Security Architect Review (Fatima Al-Hassan)

## Surface Area

**Endpoint:** `POST /v1/login`
**Transport:** control-api (REST)
**Service:** control-service → login_handler → session_service → control-store
**Policy:** LoginPolicy (rate limits, credential validation rules, block rules)
**Audit:** Every login event (success, failure, block) recorded in audit trail
**Snapshot/Rollback:** State capture and restoration

## Assets

| Asset | Sensitivity | Where it lives |
|-------|-------------|----------------|
| User credentials | HIGH — in-transit only, must not be stored | HTTP request body, validated immediately |
| LoginPolicy config | MEDIUM — controls access rules | control-store, loaded by login_handler |
| Session tokens | HIGH — access tokens | Issued by session_service, returned to caller, stored in token store |
| Audit events | MEDIUM — integrity-sensitive | audit-log crate with hash chain |
| Block state | MEDIUM — who is blocked and until when | control-store |

## Trust Zones

```
┌─────────────────────────────────────────────────────────────────┐
│  ZONE 0: UNTRUSTED                                              │
│  External callers (any network source)                          │
│  HTTP request body — credentials                                 │
│  Control-api route layer                                        │
├─────────────────────────────────────────────────────────────────┤
│  TRUST BOUNDARY — Authentication check at route layer            │
│  (POST /v1/login is the ONE unauthenticated endpoint)            │
├─────────────────────────────────────────────────────────────────┤
│  ZONE 1: EDGE-TRUST                                             │
│  Validated request (fields checked, policy consulted)            │
│  Session service (creates new sessions)                          │
├─────────────────────────────────────────────────────────────────┤
│  ZONE 2: POLICY-TRUST                                           │
│  LoginPolicy evaluator                                           │
│  Rate limit calculator                                           │
├─────────────────────────────────────────────────────────────────┤
│  ZONE 3: SYSTEM-TRUST                                           │
│  control-store (persistence)                                     │
│  audit-log (integrity chain)                                     │
│  snapshot-ledger (state capture)                                 │
└─────────────────────────────────────────────────────────────────┘
```

## STRIDE Analysis

### S — Spoofing

| Threat | Likelihood | Impact | Mitigation | Residual |
|--------|-----------|--------|------------|----------|
| Attacker sends fake credentials claiming to be another user | High | High | Credential validation against stored credentials in LoginPolicy; no implicit trust of caller identity | Low — protected by credential validation |
| Attacker replays a captured login request | Medium | High | Nonce/state field in login request (per spec); short-lived request validity window | Low — nonce prevents replay |
| Attacker spoofs the login response to deliver a fake session token | Low | Critical | Session token is signed by authority-domain key; caller validates signature on receipt | Low — signing prevents forgery |

### T — Tampering

| Threat | Likelihood | Impact | Mitigation | Residual |
|--------|-----------|--------|------------|----------|
| Attacker modifies login request in transit | Medium | High | HTTPS/TLS for all API traffic; request body validation at route layer | Low — TLS prevents MITM |
| Attacker modifies LoginPolicy in storage | Low | Critical | LoginPolicy stored with integrity check (hash); policy changes are audited; only authorized operators can modify | Low — integrity controls |
| Attacker modifies audit trail | Medium | High | Audit log uses hash chain — tampering breaks chain integrity; chain verified on read | Medium — detection but not prevention |

### R — Repudiation

| Threat | Likelihood | Impact | Mitigation | Residual |
|--------|-----------|--------|------------|----------|
| User denies attempting to log in | Low | Medium | Every login attempt (success AND failure) produces an audit event with timestamp, actor identity, outcome | Low — audit trail provides non-repudiation |
| System denies processing a login | Low | High | Handler always produces audit event before returning; if handler crashes before audit write, no session is issued | Medium — crash-before-audit is detectable via missing expected events |

### I — Information Disclosure

| Threat | Likelihood | Impact | Mitigation | Residual |
|--------|-----------|--------|------------|----------|
| Credentials leaked in error message | Medium | Critical | Error messages must NOT include credential values, partial matches, or hints that reveal valid vs invalid patterns | Low — strict output sanitization |
| Rate limit response reveals valid accounts | Medium | Low | Rate limit message should not distinguish "valid account rate-limited" vs "invalid account rate-limited" — same message for both | Medium — behavioral oracle exists but minimal impact |
| Session token leaked in logs | Medium | High | Session tokens must never be logged; log sanitization at the transport layer | Low — logging control |
| Stack trace or internal path leaked in 500 error | Medium | High | Generic error message for all server errors; no internal paths, SQL, or stack traces in HTTP responses | Low — error handling pattern |

### D — Denial of Service

| Threat | Likelihood | Impact | Mitigation | Residual |
|--------|-----------|--------|------------|----------|
| Attacker floods login endpoint with requests | High | High | LoginPolicy rate limits per actor; rate limit enforced before credential validation (cheap check first) | Medium — rate limit can still be overwhelmed at infrastructure level; rate limit should also apply per IP |
| Attacker sends large request bodies to exhaust memory | Medium | Low | Request body size limits at route layer; field length validation on credentials and scope | Low — size limits |
| Attacker causes expensive credential validation to exhaust CPU | Medium | Medium | Credential validation should be constant-time (prevent timing oracle) and bounded CPU | Low — constant-time validation |

### E — Elevation of Privilege

| Threat | Likelihood | Impact | Mitigation | Residual |
|--------|-----------|--------|------------|----------|
| Attacker crafts a session token with elevated scope | Low | Critical | Session token is signed by authority key; scope is encoded inside the signed token; signature validation prevents scope tampering | Low — signed tokens |
| Attacker uses a pre-authentication session as authenticated | Medium | High | Session tokens are NOT issued until authentication succeeds; there is no "pre-auth session" in this flow | Low — correct flow design |
| Attacker exploits session fixation | Medium | High | Session token is created ONLY after successful authentication; no token exists before auth to fixate | Low — correct design |

## Security Requirements for Implementation

### SR-1: Authentication Transport (CRITICAL)
`POST /v1/login` is the only endpoint that accepts unauthenticated requests. A threat model for this endpoint must:
- **SR-1.1:** Validate all fields before any policy check (fail fast on malformed input)
- **SR-1.2:** Never log credential values
- **SR-1.3:** Enforce request body size limits (max 64KB for login requests)
- **SR-1.4:** Return generic errors — never distinguish "invalid credentials" vs "account not found" in the response message

### SR-2: Credential Validation (CRITICAL)
- **SR-2.1:** Credential validation must be constant-time (prevent timing side-channel attacks)
- **SR-2.2:** Credential comparison must use a secure comparison function, not `==` or `memcmp`
- **SR-2.3:** Credentials must be discarded immediately after validation — never stored, cached, or logged

### SR-3: Rate Limiting (HIGH)
- **SR-3.1:** Rate limit check MUST happen BEFORE credential validation (cheap check first, prevent DoS amplification)
- **SR-3.2:** Rate limit response must be indistinguishable from "invalid credentials" to prevent account enumeration
- **SR-3.3:** Rate limit window and thresholds must be configurable via LoginPolicy
- **SR-3.4:** Block state must have a TTL (automatic expiry)

### SR-4: Session Token (HIGH)
- **SR-4.1:** Session token must be signed by the authority-domain key
- **SR-4.2:** Token scope must be encoded inside the signed payload — not mutable after issuance
- **SR-4.3:** Token expiry must be set explicitly (not default/forever)
- **SR-4.4:** Session token must never appear in audit trail, log output, or error messages

### SR-5: Audit Trail (MEDIUM)
- **SR-5.1:** Every login attempt must produce an audit event BEFORE the response is returned
- **SR-5.2:** Audit event fields: actor_id (if identifiable), timestamp, outcome (success/failure/blocked), audit_event_id
- **SR-5.3:** Audit event must NOT contain credential values or session tokens
- **SR-5.4:** Audit hash chain integrity must be verifiable

### SR-6: Input Validation (MEDIUM)
- **SR-6.1:** All request fields must have maximum length limits (e.g., provider max 256 chars, scope max 1024 chars)
- **SR-6.2:** Scope values must match a known allowlist pattern (no arbitrary strings)
- **SR-6.3:** HTTP response headers must include `Cache-Control: no-store` (prevent session token caching by proxies)

## Fatima's Verdict

**Risk level: HIGH** — This is the authentication gateway for the entire system. A vulnerability here compromises everything behind it.

**Critical items that must be in Phase 9, not deferred:**
1. Constant-time credential comparison — no exceptions
2. Generic error messages (no account enumeration)
3. Rate limit before credential validation
4. Signed session tokens with explicit scope
5. Audit trail before response
6. No credential logging

**Concerns about spec alignment:**
The spec in `docs/spec/19_FIRST_VERTICAL_LOGIN_MODULE.md` lists request fields as `provider`, `redirect_uri`, `tenant_hint`, `nonce`, `state`. These are OAuth/OIDC fields. If the implementation uses these as-is, the threat model needs to account for:
- **Open Redirect attack**: `redirect_uri` could redirect users to malicious sites after authentication
- **CSRF with `state`**: If `state` is not validated, CSRF attacks are possible

I concur with Iris's recommendation: simplify to `username/credential` + `scope` for Phase 9. The OAuth/OIDC contracts should be a follow-up phase with their own threat model.

**Handoff from Fatima Al-Hassan:** Threat model complete with 6 security requirements (SR-1 through SR-6). Sending to Dmitri (Backend) for implementation, and to Rena (CTO) for architectural acceptance.
