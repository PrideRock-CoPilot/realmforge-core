# RealmForge Repository Boundaries

## Parent Workspace

`E:\realmforge` is the parent workspace for RealmForge repositories.

## Core Repo

The control workspace owns the foundation:

- identity and typed scope
- authorization decisions
- skill-session validity checks
- append-only audit contracts
- PostgreSQL persistence contracts
- snapshot manifests and rollback anchors
- thin API, MCP, and CLI surfaces

## Future Repos

Future Builder, runtime, UI, module-library, and ingestion repos must treat Core as the authority source. They may request commands, submit evidence, and read approved state, but they must not duplicate authorization, approval, or rollback logic.

```text
human/agent/client
  -> future repo surface
  -> control-service command authorization
  -> state write + audit event + snapshot anchor
```
