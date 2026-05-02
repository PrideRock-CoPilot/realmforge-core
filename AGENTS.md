# RealmForge Core Engineering Instructions

RealmForge Core is the Rust-first governance kernel for RealmForge. It is the bottom layer: identity, authority, bounded commands, events, snapshots, and rollback anchors.

## 🗺️ Required Reading — Before Any Code Changes

Every agent entering this workspace **MUST** read these documents in order before touching a single file:

1. **`docs/MASTER_BUILD_PLAN.md`** — The living blueprint. Layer-by-layer, file-by-file, phase-by-phase construction plan. All decisions flow from this document.
2. **`docs/AGENT_EXECUTION_GUIDE.md`** — The deep companion. Exact Rust type signatures, DB query shapes, test structures, and agent work packets for every phase. Read this after the build plan.
3. **`docs/realm_forge_ai_native_path_forward.md`** — The north star vision document. Defines *why* RealmForge exists and what it must become.
4. **`docs/architecture/foundation.md`** — Architectural foundation and control plane responsibilities.
5. **`docs/architecture/repo-boundaries.md`** — Repository boundaries and integration contracts.
6. **`docs/decisions/ADR-0001-rust-core-foundation.md`** — Architecture Decision Record for Rust-first approach.

## 🎯 Current Build Phase

See `docs/MASTER_BUILD_PLAN.md` for the current phase of the 11-phase build plan.
The complete file manifest, validation gates, and handoff protocol are defined there.

## ⚙️ Architecture Rules

- PostgreSQL is the system of record.
- Rust domain and policy crates are the authority layer.
- API, MCP, and CLI are thin entry surfaces backed by shared core logic.
- Agents receive bounded command-style operations only.
- No raw SQL exposure to agents.
- No unrestricted table browsing.
- No arbitrary writes to backend objects.
- Production deployment is out of scope for core V1.

## 🚧 Layering

The law is absolute. No agent may violate it.

**Allowed:**

```text
api/mcp/cli -> service layer -> policy engine -> domain model -> store/snapshot adapter -> PostgreSQL
```

**Forbidden (enforced by code review):**

```text
api -> store                 (bypasses policy)
mcp -> store                 (bypasses policy)
cli -> store                 (bypasses policy)
agent -> database            (no raw SQL to agents)
snapshot -> policy bypass    (snapshot may not authorize commands)
service -> db bypass         (service must go through store adapter)
```

## 📏 File Size

- Source files target under 300 lines.
- Files over 500 lines require architectural justification.
- Split domain contracts, policy rules, persistence, transport, and tests.
- Overrides require /cto (Rena Okafor) approval documented in code comments.

## 🔄 AI Agent Workflow

Every agent implementing a phase MUST follow this order:

1. **Read phase contract** from MASTER_BUILD_PLAN.md
2. **Load /skill-creator** to register any new skills needed
3. **Convene /cto** (Rena Okafor) for architecture review on any structural changes
4. **Implement in order:** Types (rf-domain) → Policy (rf-policy) → Service (rf-service) → Store (rf-store) → Transport (rf-api/rf-cli/rf-mcp)
5. **Add tests at every layer** — TDD where possible
6. **Pass validation gates** before handoff

## 🧪 Validation Gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

- No `unwrap()` without `// INVARIANT:` comment
- No `unsafe` without `// SAFETY:` comment
- All public items have doc comments
- All errors follow the error boundary pattern (wrap and annotate at each layer)
