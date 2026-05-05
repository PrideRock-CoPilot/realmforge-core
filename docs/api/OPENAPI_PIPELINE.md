---
doc_id: DOC-API-PIPELINE
title: "OpenAPI to TypeScript Client Pipeline"
status: active
owner: api-architect
reviewers: [api-architect, frontend, backend]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: interfaces
work_path_ids: [WP-API-001, WP-BOARDS-001]
related_decision_ids: [DOC-ADR-0002]
related_file_ids: [FILE-CRATE-API-LIB]
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# OpenAPI → TypeScript Client Pipeline

**Owner:** Marcus Webb (API Architect)
**Status:** Approved — unblocked by DEC-COUNCIL-001 (2026-05-04)
**Required by:** All frontend work. No frontend API call may be written by hand.
**ADR reference:** ADR-0002 (React + TypeScript frontend stack)

---

## Purpose

This document defines the contract between the Axum REST backend and the React TypeScript
frontend. The frontend never writes API calls by hand. The pipeline generates a fully-typed
TypeScript client from the Axum OpenAPI schema. If the schema and the generated client
diverge in CI, the build fails.

This is the hard dependency that unblocks Boards. Marcus owns this pipeline.
Kai (Frontend) cannot write a single API call until this pipeline is in place.

---

## Pipeline Architecture

```
control-api (Axum + utoipa)
    │
    ├─── cargo run --bin generate-schema
    │         └── openapi.json  (OpenAPI 3.1 schema)
    │
    ├─── openapi-typescript  (generates TypeScript types)
    │         └── src/api/types.gen.ts
    │
    └─── orval  (generates React Query hooks)
              └── src/api/client.gen.ts
                  src/api/hooks.gen.ts
```

**CI enforcement:** Every backend change that touches `control-api/src/` triggers schema
regeneration. The committed `openapi.json`, `types.gen.ts`, and `hooks.gen.ts` must match
the regenerated output. If they differ, CI fails with a diff showing exactly what changed
and what must be recommitted.

---

## Backend: Schema Generation with utoipa

### Dependencies to add to `control-api/Cargo.toml`

```toml
[dependencies]
utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
utoipa-axum = "0.1"

[dev-dependencies]
# Schema generation binary — not shipped in production
```

### Schema generation binary

A separate binary `crates/control-api/src/bin/generate_schema.rs` generates and writes
the OpenAPI JSON:

```rust
// Generates openapi.json from all annotated routes and types.
// Run: cargo run -p control-api --bin generate-schema > openapi.json
fn main() {
    let schema = ApiDoc::openapi();
    let json = schema.to_pretty_json().expect("schema serialization failed");
    print!("{}", json);
}
```

### Annotation requirements

Every route handler must have a `#[utoipa::path(...)]` annotation. Every request/response
type must implement `utoipa::ToSchema`. The schema is the contract — if it compiles,
the contract is defined.

**Required metadata per endpoint:**
- `operation_id`: unique, verb-noun form (`issue_session`, `propose_command`)
- `summary`: one sentence, caller-perspective
- `request_body`: all request types with field-level descriptions
- `responses`: success response + all named error responses
- `tags`: resource group tag (`sessions`, `commands`, `audit`, etc.)

**Example annotation:**

```rust
#[utoipa::path(
    post,
    path = "/v1/session",
    operation_id = "issue_session",
    summary = "Issue a new session for an actor within a tenant and project.",
    tag = "sessions",
    request_body = IssueSessionRequest,
    responses(
        (status = 201, description = "Session issued", body = SessionResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 409, description = "Session already active", body = ApiError),
    )
)]
pub async fn issue_session(...) { ... }
```

---

## Error Response Standard (Enforced)

The current `docs/api/README.md` documents an inconsistent error format:
```json
{ "error": "ErrorKind", "message": "..." }
```

This is not the standard. The standard error shape for all RealmForge surfaces is:

```json
{
  "error": {
    "code": "SESSION_NOT_FOUND",
    "message": "The requested session 'ses_abc123' does not exist.",
    "status": 404,
    "context": {
      "session_id": "ses_abc123"
    },
    "trace_id": "req_9f3k2j"
  }
}
```

**Requirements:**
- `code`: `SCREAMING_SNAKE_CASE`, stable across versions, machine-readable
- `message`: human-readable, specific, actionable — never "something went wrong"
- `status`: HTTP status code, mirrored in body for client convenience
- `context`: structured data for diagnosis. May be empty `{}` but never absent.
- `trace_id`: the `X-Request-Id` from the request, for log correlation

The `ApiError` struct in `control-api/src/error.rs` must be updated to match this shape
and annotated with `#[derive(utoipa::ToSchema)]`. Dmitri (Backend) owns this fix.

**All error codes are named and stable.** Adding a new error code is backward-compatible.
Renaming or removing one is a breaking change requiring a version bump.

---

## Frontend: TypeScript Client Generation

### Tool: `openapi-typescript` (types) + `orval` (React Query hooks)

These are separate concerns:
- **`openapi-typescript`** generates pure TypeScript type definitions from the schema.
  No runtime dependency. Just types.
- **`orval`** generates React Query hooks (`useQuery`, `useMutation`) from the operations.
  These are the hooks Kai (Frontend) uses in components — never raw `fetch`.

### `openapi-typescript` invocation

```bash
npx openapi-typescript openapi.json -o src/api/types.gen.ts
```

Output: `src/api/types.gen.ts` — TypeScript types for every request body,
response body, and error type. Do not edit by hand.

### `orval` configuration: `orval.config.ts`

```typescript
import { defineConfig } from 'orval';

export default defineConfig({
  realmforge: {
    input: './openapi.json',
    output: {
      mode: 'tags-split',              // one file per API tag (sessions, commands, etc.)
      target: './src/api/hooks.gen.ts',
      schemas: './src/api/types.gen.ts',
      client: 'react-query',           // generates useQuery / useMutation hooks
      override: {
        mutator: {
          path: './src/api/client.ts', // custom fetch wrapper (auth headers, base URL)
          name: 'customFetch',
        },
        query: {
          useQuery: true,
          useInfiniteQuery: true,      // for paginated endpoints (audit events, snapshots)
          useMutation: true,
        },
      },
    },
  },
});
```

### Custom fetch wrapper: `src/api/client.ts`

The custom fetch wrapper is the only place authentication headers, base URL, and
error normalization are written. Kai (Frontend) owns this file.

```typescript
// src/api/client.ts — NOT generated, written once, owned by Kai
export const customFetch = async <T>(
  url: string,
  options: RequestInit,
): Promise<T> => {
  const session = getSession(); // from auth store
  const response = await fetch(`${API_BASE_URL}${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      'X-Session-Id': session?.id ?? '',
      ...options.headers,
    },
  });
  if (!response.ok) {
    const error = await response.json();
    throw new ApiClientError(error.error); // normalized error shape
  }
  return response.json();
};
```

### Generated hook usage pattern

```typescript
// Components use generated hooks — never raw fetch
import { useGetSession, useIssueSession } from '@/api/hooks.gen';

function SessionPanel({ sessionId }: { sessionId: string }) {
  const { data, isLoading, error } = useGetSession(sessionId);
  const { mutate: issueSession } = useIssueSession();
  // ...
}
```

---

## File Ownership

| File | Owner | Edit by hand? |
|------|-------|---------------|
| `openapi.json` | Generated by `generate-schema` binary | No — committed, regenerated in CI |
| `src/api/types.gen.ts` | Generated by `openapi-typescript` | No |
| `src/api/hooks.gen.ts` | Generated by `orval` | No |
| `src/api/client.ts` | Kai (Frontend) | Yes — auth, base URL, error normalization |
| `orval.config.ts` | Marcus (API Architect) | Only on structural change |

---

## CI Pipeline: `api-contract-check`

This job runs on every push that touches `crates/control-api/src/` or `frontend/src/api/`.

```yaml
# .github/workflows or equivalent CI config
api-contract-check:
  steps:
    - name: Generate OpenAPI schema
      run: cargo run -p control-api --bin generate-schema > openapi.json.new

    - name: Check schema diff
      run: |
        diff openapi.json openapi.json.new || {
          echo "ERROR: openapi.json is stale. Run 'cargo run -p control-api --bin generate-schema > openapi.json' and commit."
          exit 1
        }

    - name: Regenerate TypeScript types
      run: npx openapi-typescript openapi.json -o src/api/types.gen.ts.new

    - name: Check types diff
      run: |
        diff src/api/types.gen.ts src/api/types.gen.ts.new || {
          echo "ERROR: types.gen.ts is stale. Run 'npx openapi-typescript openapi.json -o src/api/types.gen.ts' and commit."
          exit 1
        }

    - name: Regenerate React Query hooks
      run: npx orval --config orval.config.ts

    - name: Check hooks diff
      run: |
        diff src/api/hooks.gen.ts src/api/hooks.gen.ts.new || {
          echo "ERROR: hooks.gen.ts is stale. Regenerate with orval and commit."
          exit 1
        }
```

**If this job fails:** The backend changed an API contract without regenerating the client.
Find the diff, run the generation commands, and commit the updated generated files
alongside the backend change. Never merge a backend API change without the updated client.

---

## Breaking Change Protocol

A breaking change is any change to the OpenAPI schema that removes or renames a field,
changes a field type, removes an endpoint, or renames an error code.

**Before merging a breaking change:**
1. Marcus (API Architect) must assess and document the break
2. A new API version (`/v2/...`) must be defined for the changed endpoint(s)
3. The old version remains active with a deprecation notice in the schema
4. Kai (Frontend) must update the frontend to the new version before the old is removed
5. Old version sunset: minimum 90 days after the new version ships

A schema change that only adds fields or adds new endpoints is backward-compatible
and does not require a version bump.

---

## Endpoint Coverage Requirement

The OpenAPI schema must cover 100% of registered routes. Every route in `control-api/src/lib.rs`
must appear in the schema. The `generate-schema` binary CI check will enforce this —
any unannotated route is an error.

**Current undocumented route groups** (as of 2026-05-04, per audit):
Routes exist in `control-api/src/lib.rs` for: catalog, gateway, knowledge, work-paths,
bundles, runtime, login, live-watch, boards, build-watch. These are not yet documented
in `docs/api/README.md` and are not yet annotated with `utoipa`. Annotation is required
before any of these surfaces can be consumed by the frontend.

Priority order for annotation (driven by Boards vertical dependency):
1. `boards` — required for first Boards view
2. `session` — required for auth flow
3. `commands` — required for all board interactions
4. `audit` — required for Evidence Board
5. `work-paths` — required for Work Path Board
6. `snapshots`, `rollback` — required for Release Board
7. All remaining groups

---

## Frontend Workspace Setup

The frontend lives at `frontend/` in the workspace root. Kai (Frontend) will create
this directory as part of the frontend vertical spec. The `package.json` must include:

```json
{
  "scripts": {
    "generate:schema": "cargo run -p control-api --bin generate-schema > openapi.json",
    "generate:types": "openapi-typescript openapi.json -o src/api/types.gen.ts",
    "generate:hooks": "orval --config orval.config.ts",
    "generate": "npm run generate:schema && npm run generate:types && npm run generate:hooks"
  },
  "devDependencies": {
    "openapi-typescript": "^7.0.0",
    "orval": "^7.0.0"
  },
  "dependencies": {
    "@tanstack/react-query": "^5.0.0",
    "openapi-fetch": "^0.12.0"
  }
}
```

---

## Handoff

**From Marcus to Dmitri (Backend):**
- Add `utoipa` + `utoipa-axum` to `control-api/Cargo.toml`
- Create `control-api/src/bin/generate_schema.rs`
- Update `ApiError` struct to match the standard error shape
- Annotate all route handlers with `#[utoipa::path(...)]`, priority order above
- All request/response types must implement `ToSchema`

**From Marcus to Kai (Frontend):**
- This document is the contract. No hand-written API calls.
- Write `src/api/client.ts` (custom fetch wrapper) — this is the only API file you write
- All other API files are generated by `npm run generate`
- React Query hooks come from `hooks.gen.ts` — use them directly in components

**From Marcus to Alex (PM):**
- OpenAPI annotation of all routes is a prerequisite for any Boards component
- Dmitri owns the annotation work — it should be the first backend task in the Boards sprint
- Estimated effort: 1–2 days for a developer familiar with the codebase

---

*Last reviewed: 2026-05-04 — Marcus Webb*
