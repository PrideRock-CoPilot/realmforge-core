---
paths:
  - "crates/**"
  - "Cargo.toml"
  - "db/**"
---

# Architecture Laws

## Layer Law — Absolute, No Exceptions

```
api / mcp / cli  →  policy / domain service  →  store adapter  →  PostgreSQL
```

- Transport crates (`control-api`, `agent-mcp`, `operator-cli`) are thin dispatch only
- No business logic in transport crates
- No direct database access outside `control-store`
- No policy decisions outside `policy-engine`
- No IO in `authority-domain`

Violations must be escalated to `/cto` before any code is written.

## Crate Law — One Crate, One Responsibility

| Crate | Owns | Forbidden |
|---|---|---|
| `authority-domain` | Pure types, logic, state — no IO | Database, HTTP, file IO |
| `policy-engine` | Authorization decisions | Persistence, network |
| `control-store` | PostgreSQL persistence | Business logic, policy |
| `control-service` | Service orchestration | Direct DB access |
| `control-api` | HTTP dispatch | Business logic, direct DB |
| `agent-mcp` | MCP tool dispatch | Business logic |
| `operator-cli` | CLI dispatch | Business logic |

## Rust Rules

- No `unwrap()` or `expect()` in production code without a comment explaining the invariant
- No `unsafe` block without a `// SAFETY:` comment
- No bare `Uuid` or `String` as identifiers — use typed newtypes
- No stringly-typed errors — use `thiserror` with typed variants
- No `Mutex` held across an `.await`
- Every input to every public endpoint is validated before use

## File Size Law

- Target: ≤300 lines per file
- Hard cap: 500 lines (requires written justification in a doc comment)

## Package Naming

Follow `docs/decisions/DEC-COUNCIL-002-backend-crate-package-names.md` for all crate and package names.
