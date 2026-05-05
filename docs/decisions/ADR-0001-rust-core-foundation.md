# ADR-0001: Rust-First Core Foundation

## Status

Accepted

## Decision

RealmForge Core is implemented as a Rust workspace. Python Builder code may remain useful as a prototype or higher-level adapter, but it is not the foundation.

## Rationale

The control plane is security-sensitive. It must enforce authorization, approval state, skill-session validity, event integrity, and rollback anchors deterministically. Rust provides strong typing, memory safety, predictable deployment, and a single-language base for CLI, API, MCP, snapshot, and runtime-adjacent code.

## Consequences

- Core V1 starts below Builder workflows.
- API/MCP/CLI surfaces stay thin.
- PostgreSQL remains the durable system of record.
- Higher-level repos will integrate with the control-service authority boundary instead of owning authority themselves.
