# RealmForge Core Foundation

## Bottom Layer

RealmForge Core starts with authority, not generation.

```text
actor scope
  -> policy decision
  -> bounded command
  -> transactional state write
  -> append-only audit event
  -> snapshot anchor
```

If this layer is wrong, every Builder, runtime, UI, and agent surface above it becomes unsafe.

## Control Plane Responsibilities

- Tenant, project, actor, role, session, skill, and skill-session identity.
- Typed policy decisions with corrective denial responses.
- Append-only event history with hash-chain support.
- PostgreSQL persistence through migrations applied before startup.
- Snapshot manifests that bind state exports and content-addressed objects.
- Thin API, MCP, and CLI adapters over shared Rust logic.

## Non-Responsibilities

- Code generation.
- Runtime bundle serving.
- Production deployment.
- Direct workspace file mutation.
- Arbitrary SQL or table browsing for agents.
