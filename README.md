# RealmForge Core

RealmForge Core is the Rust-first control-plane foundation for governed AI-native software execution.

It answers the core question:

```text
Who is allowed to do what, to which project state, through which skill/session,
under which approval, with what evidence, and how is it restored?
```

## V1 Foundation

```text
typed domain model
  -> policy decision engine
  -> append-only audit/event contracts
  -> PostgreSQL store adapter
  -> content-addressed snapshot manifests
  -> thin API/MCP/CLI entry surfaces
```

## Workspace

```text
crates/
  rf-domain     typed IDs, state models, lifecycle enums
  rf-policy     authorization and corrective denial decisions
  rf-events     append-only audit event hash-chain contracts
  rf-store      PostgreSQL persistence boundary
  rf-snapshot   content-addressed objects and manifest validation
  rf-api        REST adapter over shared core logic
  rf-mcp        MCP tool contracts over shared core logic
  rf-cli        operator CLI for migrate, inspect, and validation flows
db/migrations/  production schema, applied before startup
docs/           architecture and decision records
```

## Toolchain

Install Rust before building:

```powershell
winget install Rustlang.Rustup
rustup default stable
```

Then validate:

```powershell
cargo fmt --all -- --check
cargo test --workspace
```

Run the API:

```powershell
cargo run -p rf-api
```

Inspect migrations:

```powershell
cargo run -p rf-cli -- list-migrations
```
