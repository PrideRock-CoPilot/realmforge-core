# RealmForge Workspace

RealmForge is an AI-native software construction and runtime governance ecosystem.

**Core Thesis:** Git made human software collaboration scalable. RealmForge makes AI software execution governable.

The current milestone is docs-first buildout: the product, metadata, work paths, agent grants, interfaces, data contracts, and acceptance tests are specified before implementation expands.

## 🚨 Open Architectural Decisions

**✅ Storage Format POC Complete** - Custom `.rfsource` format validated in < 2 hours. Rust backend functional, compacts to Parquet-equivalent size, includes git-like commit/history functions. **Status:** Benchmarking phase (Week 1-3). Decision: Custom format vs. DuckDB vs. Delta Lake. See `/Users/pliekhus@outlook.com/docs/DEC-OPEN-storage-format-buildout.md` for full POC results and next steps. **Risk downgraded:** High → Low-Medium. **Timeline revised:** 17 weeks → 6-10 weeks for production.

---

## 🚀 Quick Start for Agents

**New to this workspace? Start here:**

1. **Read** `~/.assistant/QUICKSTART.md` — 60-second overview + first session checklist
2. **Read** `~/.assistant/REALMFORGE.md` — Core vision and architecture
3. **Read** `CLAUDE.md` — Workspace instructions and skill system
4. **Read** `AGENTS.md` — Engineering rules and constraints

**Complete index:** `~/.assistant/INDEX.md`

## Canonical Docs

- `docs/realm_forge_ai_native_path_forward.md` — source vision.
- `docs/spec/00_INDEX.md` — canonical specification entry point.
- `docs/codex/00_CODEX_SETUP.md` — Codex setup and skill installation notes.
- `AGENTS.md` — required agent instructions for this workspace.
- `CLAUDE.md` — workspace instructions and company skill system.

## Current Code

- `crates/` — Rust-first governance kernel and control-plane foundation.

Crate naming is accepted in `docs/decisions/DEC-COUNCIL-002-backend-crate-package-names.md`. Workspace flattening is now in progress under Phase 1.

## The 13 Product Modules

1. **Authority Core** — Identity, policy, commands, audit, snapshots, rollback
2. **Catalogs** — Global, tenant, app catalogs
3. **Skill Grants** — Zero-cap agents, hard grants
4. **Agent Gateway** — Enforced operation gateway
5. **Work Paths** — Structured planning graph
6. **Boards** — Human command surface
7. **Knowledge** — Scoped retrieval
8. **Build Watch** — Construction monitoring
9. **Runtime Bundle** — Signed bundle creation
10. **Live Runtime** — Governed execution
11. **Live Watch** — Production monitoring
12. **Cost Ledger** — Cost tracking
13. **Visual Map** — Graph metadata layer

See `~/.assistant/WORK_AREAS.md` for detailed breakdown.
