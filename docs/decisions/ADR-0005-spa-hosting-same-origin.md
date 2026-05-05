# ADR-0005: Same-Origin SPA Hosting from Axum (Phase 1)

**Date:** 2026-05-04
**Status:** Accepted
**Deciders:** Nadia Kovacs (Infra Architect)
**Consulted:** Rena Okafor (CTO), Kai Sato (Frontend), Dmitri Volkov (Backend), Fatima Al-Hassan (Security Architect, observing)

## Context

The React SPA (Vite build output) must be served to browsers. Two models exist: serving static
files from the Axum `control-api` binary at the same origin, or deploying separately to a CDN
or independent static host at a different origin. The choice determines CORS configuration
requirements, `API_BASE_URL` strategy, deployment coupling, and operational complexity.

## Decision Drivers

- Zero production users at time of decision — CDN infrastructure has no current beneficiary
- CORS misconfiguration is a real attack surface; eliminating the requirement eliminates the risk
- Operational simplicity at zero-to-one scale is more valuable than deployment flexibility
- CDN migration is a planned future decision, not a permanent exclusion

## Options Considered

### Option A: Served by Axum (same-origin, single binary)

The Vite `dist/` output is served by `control-api` via `tower-http::ServeDir`. A catch-all
route serves `index.html` for all non-API paths (client-side routing). `VITE_API_BASE_URL=""`
— all API calls go to the same origin. Local dev: Vite's built-in proxy forwards `/v1/*` to
the local Axum instance. Single Docker image, single deployment unit.

- Pro: No CORS headers required. No CORS misconfiguration attack surface.
- Pro: Single deployment artifact. Simpler CI/CD pipeline.
- Pro: Local development setup is standard Vite proxy — no additional tooling.
- Con: Frontend and backend deployments are coupled. A frontend-only change requires a backend
  redeploy. At scale, Axum handles static file serving load.

### Option B: CDN / Separate static host (cross-origin)

Vite `dist/` is published independently. `VITE_API_BASE_URL` is set per environment.
CORS headers are required on `control-api` for the CDN origin.

- Pro: Decoupled release cycles. Global edge caching. Independent scaling.
- Con: CORS allowlist management per environment is operational overhead. Wildcard CORS is
  not acceptable (Fatima's threat model). CDN infrastructure must be operated before it has
  users. Premature complexity.

## Decision

The React SPA is served as static files from the Axum `control-api` binary. Same-origin.
`VITE_API_BASE_URL=""`. No CORS required. This is the Phase 1 hosting model.

CDN migration is explicitly deferred to Phase 2 and requires a new ADR before any production
CDN deployment is made.

## Rationale

At zero-to-one scale, CDN infrastructure has no current beneficiary. The coupling cost (backend
redeploy for frontend changes) is not a real cost until the frontend has a release cadence that
diverges from the backend — which it does not yet. Eliminating CORS removes an entire class of
security misconfiguration risk with no offsetting cost at the current phase. The migration path
to CDN is deliberate: it is documented, triggered by observable criteria, and governed by a
future ADR. It is not an omission.

## Consequences

**Positive:**
- No CORS configuration. No CORS attack surface.
- Single deployment artifact. Simpler CI/CD at zero-to-one scale.
- `VITE_API_BASE_URL=""` — no per-environment API URL management.
- Local dev: `vite.config.ts` proxy to `localhost:{PORT}` is the only dev config needed.

**Negative / Trade-offs:**
- Frontend and backend deployments are coupled. Frontend-only changes require a backend redeploy.
- Axum handles static file serving load (mitigated by `tower-http::ServeDir` efficiency and HTTP/2).

**Risks:**
- Deployment coupling becomes friction if frontend release cadence increases significantly.
  Nadia to flag this at 500 concurrent users or when frontend and backend release cycles diverge.
- `tower-http::ServeDir` must serve `index.html` as the fallback for all non-asset, non-API paths
  to support client-side routing. Dmitri to implement and test the catch-all route.

## Phase 2 Migration Trigger Criteria

Nadia to document in the infrastructure runbook. Recommended triggers for CDN migration ADR:

- 500+ concurrent users sustained
- Frontend release cadence exceeds backend release cadence for two consecutive sprints
- Static asset serving measurably impacts API response latency

## Review Date

2027-05-04, or when a Phase 2 migration trigger criterion is met.
