# RealmForge

RealmForge is an AI-native software construction and runtime governance ecosystem.

**Core Thesis:** Git made human software collaboration scalable. RealmForge makes AI software execution governable.

---

## Current Milestone

**Docs-first buildout:** The product, metadata, work paths, agent grants, interfaces, data contracts, and acceptance tests are specified before implementation expands.

**Architecture Phase:** Storage layer refactor complete (`.rfsource` custom format), governance crates operational, 26 production crates.

---

## 🚀 Quick Start

**New to this workspace? Start here:**

1. **Read** `AGENTS.md` — Engineering rules and constraints
2. **Read** `docs/spec/00_INDEX.md` — Canonical specifications
3. **Read** `docs/architecture/rfsource-overview.md` — Storage architecture
4. **Read** `CHANGELOG.md` — Recent changes and migrations

---

## Architectural Status

### ✅ Storage Format Decision: RESOLVED

**Decision:** Custom `.rfsource` format  
**Rationale:** POC validated in <2 hours, Parquet-equivalent compression, git-like versioning, governance-first design  
**Status:** Production-ready, 8-crate modular implementation  
**Timeline:** 17 weeks → 6-10 weeks (revised after POC success)

**See:** 
* `docs/architecture/rfsource-overview.md` - Full architecture
* `docs/archive/README.md` - Decision history
* `CHANGELOG.md` - Migration guide from `parquet-store`

### Current Architecture

**26 Production Crates** organized into domains:

#### Storage Layer (8 crates)
* `rfsource-core` - Domain model, typed IDs
* `rfsource-format` - Binary frame format (`.rfsource` files)
* `rfsource-index` - Bloom filters, statistics
* `rfsource-store` - ACID operations, git-like commits
* `rfsource-governance` - Policy enforcement
* `rfsource-catalog` - Schema registry
* `rfsource-query` - SQL query engine
* `rfsource-materialize` - Materialized views
* `rfsource-service` - HTTP/gRPC API

#### Governance Layer (3 crates)
* `code-review` - Automated review engine
* `workflow-engine` - Stage orchestration
* `design-council` - Multi-stakeholder decisions

#### Authority & Control (8 crates)
* `authority-domain` - Identity, policy, commands
* `policy-engine` - Policy evaluation
* `audit-log` - Audit trail
* `snapshot-ledger` - State snapshots
* `control-service` - Control plane API
* `control-store` - Control data persistence
* `control-api` - HTTP endpoints
* `agent-gateway` - Operation gateway

#### Runtime & Monitoring (5 crates)
* `runtime-bundle` - Signed bundle creation
* `live-runtime` - Governed execution
* `live-watch` - Production monitoring
* `agent-mcp` - MCP protocol support
* `intake-engine` - Request intake system

#### Tooling (2 crates)
* `operator-cli` - Admin command-line interface

---

## The 13 Product Modules

These represent the complete RealmForge vision. Implementation is staged; see `docs/spec/00_INDEX.md` for detailed status.

1. **Authority Core** — Identity, policy, commands, audit, snapshots, rollback
2. **Catalogs** — Global, tenant, app catalogs (metadata layer)
3. **Skill Grants** — Zero-cap agents, hard capability grants
4. **Agent Gateway** — Enforced operation gateway with policy checks
5. **Work Paths** — Structured planning graph for construction tasks
6. **Boards** — Human command surface for agent orchestration
7. **Knowledge** — Scoped retrieval and context management
8. **Build Watch** — Real-time construction monitoring
9. **Runtime Bundle** — Signed, immutable deployment artifacts
10. **Live Runtime** — Governed agent execution environment
11. **Live Watch** — Production monitoring and observability
12. **Cost Ledger** — Token and compute cost tracking
13. **Visual Map** — Graph visualization of system metadata

**Current Focus (Q2 2026):** Modules 1, 4, 9, 10 (Authority, Gateway, Bundle, Runtime)

---

## Documentation

### Essential Reading
* `docs/spec/00_INDEX.md` — Canonical specifications (source of truth)
* `docs/architecture/rfsource-overview.md` — Storage architecture deep-dive
* `docs/realm_forge_ai_native_path_forward.md` — Original vision document
* `CHANGELOG.md` — Breaking changes and migration guides
* `AGENTS.md` — Required instructions for AI agents in this workspace

### Specialized Topics
* `docs/codex/` — Codex setup and skill system
* `docs/decisions/` — Architectural decision records (ADRs)
* `docs/archive/` — Historical design documents and progress tracking

### API & Integration
* `openapi.json` — OpenAPI 3.0 specification for HTTP APIs
* `docs/mcp/` — Model Context Protocol integration guides

---

## Development

### Prerequisites
* **Rust**: 1.75+ (specified in `rust-toolchain.toml`)
* **Database**: PostgreSQL 16+ (via Docker: `docker-compose up -d`)
* **Tools**: `cargo`, `git`

### Quick Start

```bash
# Clone and setup
git clone <repo-url>
cd realmforge-core-local

# Start PostgreSQL
docker-compose up -d

# Build all crates
cargo build

# Run tests
cargo test

# Run control API
cargo run -p control-api
```

### Crate Naming Convention

Accepted in `docs/decisions/DEC-COUNCIL-002-backend-crate-package-names.md`:
* Domain names are singular: `authority-domain`, not `authorities-domain`
* Services are suffixed: `control-service`, `rfsource-service`
* Stores are suffixed: `control-store`, `rfsource-store`
* Engines are suffixed: `policy-engine`, `workflow-engine`

### Workspace Structure

```
realmforge-core-local/
├── crates/              # 26 production Rust crates
├── docs/
│   ├── spec/            # Canonical specifications
│   ├── architecture/    # Technical architecture docs
│   ├── decisions/       # ADRs
│   ├── archive/         # Historical docs
│   └── codex/           # Development guides
├── db/                  # Database migrations
├── frontend/            # UI (future)
├── catalog/             # Catalog definitions (future)
├── Cargo.toml           # Workspace root
├── README.md            # This file
├── AGENTS.md            # AI agent instructions
├── CHANGELOG.md         # Version history
└── docker-compose.yml   # Local PostgreSQL
```

---

## Contributing

### Before Starting Work

1. Read `AGENTS.md` - Engineering constraints
2. Check `docs/spec/00_INDEX.md` - Understand current scope
3. Review `docs/decisions/` - Know past architectural decisions
4. Run `cargo test` - Ensure baseline passes

### Development Workflow

1. **Design First** - Write spec in `docs/spec/` before coding
2. **Layered Changes** - Respect crate dependencies (bottom-up only)
3. **Test Coverage** - Add tests for new functionality
4. **Documentation** - Update relevant docs in `docs/`
5. **Code Review** - Automated via `code-review` crate

### Crate Layering Rules

**Bottom-Up Dependencies Only:**
* `rfsource-core` cannot depend on `rfsource-store`
* `rfsource-store` can depend on `rfsource-core`
* Services (top layer) can depend on anything below

**Enforcement:** Manual code review + planned automated checks

---

## Project Status

### Completed
* ✅ Storage format decision (custom `.rfsource`)
* ✅ 26 crates organized into coherent domains
* ✅ Governance crates (code-review, workflow-engine, design-council)
* ✅ Authority domain implementation
* ✅ MCP protocol support (agent-mcp)

### In Progress
* 🚧 Storage layer benchmarking (Q2 2026)
* 🚧 Intake engine refinement
* 🚧 Control API expansion

### Planned
* 📋 Distributed query execution (Q3 2026)
* 📋 Frontend UI (Q3 2026)
* 📋 Knowledge retrieval module (Q4 2026)

**See:** `docs/spec/00_INDEX.md` for detailed roadmap

---

## Architecture Highlights

### Storage: RFSource

**Why custom format?**
* Git-like version control built-in (commit, rollback, history)
* Governance enforced at storage layer (not bolted on)
* Parquet-equivalent compression
* Full control for AI-native optimizations

**Key insight:** AI construction requires rollback semantics. Can't bolt versioning onto existing formats without compromising governance.

### Governance: Self-Hosting

**Philosophy:** RealmForge should govern its own construction.

**Implementation:**
* `code-review` crate enforces standards automatically
* `workflow-engine` orchestrates development stages
* `design-council` facilitates multi-stakeholder decisions

**Result:** Product "eats its own dog food" from day one.

### Authority: Policy-Driven

**Model:** Capability-based security with zero-trust principles.

**Enforcement Points:**
* `agent-gateway` - All operations gated
* `rfsource-governance` - Storage layer policies
* `policy-engine` - Centralized evaluation

**Audit:** Every action logged in `audit-log` (immutable, cryptographically signed).

---

## Support & Resources

### Documentation
* **Architecture:** `docs/architecture/`
* **Specifications:** `docs/spec/00_INDEX.md`
* **Changes:** `CHANGELOG.md`

### Communication
* **Issues:** GitHub Issues (when public)
* **Discussions:** GitHub Discussions (when public)
* **Internal:** See `AGENTS.md` for team protocols

### External Links
* Vision: `docs/realm_forge_ai_native_path_forward.md`
* Codex Setup: `docs/codex/00_CODEX_SETUP.md`

---

**Last Updated:** 2026-05-07  
**License:** UNLICENSED (proprietary)  
**Maintainer:** RealmForge Core Team
