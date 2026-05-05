# ADR-0002: React + TypeScript Frontend Stack

**Date:** 2026-05-04
**Status:** Accepted
**Deciders:** Dr. Rena Okafor (CTO — DRI), Victor Chen (CEO), Alex Rivera (PM)
**Consulted:** Kai Sato (Frontend), Dmitri Volkov (Backend), Chen Wei (Data Architect),
              Margaret Thompson (QA), Fatima Al-Hassan (Security Architect)
**Council session:** DEC-COUNCIL-001 (2026-05-04)

---

## Context

RealmForge Core (Phases 0–11 + Login Vertical) is complete. The governance kernel is fully
implemented in Rust: authority domain, policy engine, audit trail, snapshot ledger, service
layer, REST API, CLI, and MCP surface. The next phase of construction requires a human-facing
frontend — the **Boards** product and all subsequent UI surfaces.

Boards is the human command surface for planning, approvals, work paths, packet status,
evidence, watch signals, costs, and release readiness. It requires:

- Six complex views (Intake, Work Path, Packet, Evidence, Release, Cost Boards)
- Interactive graph rendering for the Visual Map module (`VN-*` nodes, `VE-*` edges)
- Real-time signal surfaces for Build Watch and Live Watch events
- Every interaction emits a bounded command, audit event, and visual node link
- WCAG AA accessibility (hard requirement)
- Paginated and streaming data surfaces backed by Parquet and PostgreSQL

The frontend stack decision was formally blocked (`DEC-COUNCIL-001` in
`docs/spec/22_OPEN_DECISIONS.md`) pending Council resolution. This ADR documents the
Council's decision and closes that lockout.

---

## Decision Drivers

- **Visual Map is not deferrable.** `MOD-VISUAL-MAP` is a first-class product module.
  Every Board view references visual node IDs and edge IDs. The stack must support
  production-ready interactive graph rendering on day one.
- **WCAG AA is a hard requirement.** Accessibility is not a "nice to have" — it is an
  acceptance criterion for every UI surface.
- **Real-time signal surfaces.** Build Watch and Live Watch produce streaming events that
  must appear in the Evidence Board and Release Board without page refresh.
- **Paginated, high-volume data tables.** The Evidence Board and Cost Board surface
  Parquet-backed data that may span thousands of rows. Cache invalidation and pagination
  management must be solved, not built from scratch.
- **Hiring and long-term maintainability.** The team is small. The stack must be findable
  in the talent market.
- **Security posture.** The shell strategy determines the threat surface. Tighter is better
  for a governance tool.

---

## Options Considered

### Option A: React (Vite + React 19 + TypeScript) — Web-first shell

**Description:** React with Vite for build tooling, TypeScript throughout. Component
foundation: shadcn/ui primitives (internal) wrapped by a named `ui/` package (public API).
Graph rendering: React Flow. Data fetching: React Query. Real-time: SSE/WebSocket via
React Query subscriptions. Shell: browser SPA, served from the Axum backend or a
separate static host.

**Proponent:** Kai Sato (Frontend)

- Pro: React Flow is the only production-ready, accessible, TypeScript-native graph
  rendering library in this class. No equivalent exists in Svelte or Leptos.
- Pro: Radix UI and Axe provide a mature WCAG AA testing and component ecosystem.
  Testing Library + Playwright cover unit, component, and E2E layers.
- Pro: React Query's `useInfiniteQuery`, cache invalidation, and retry logic solve
  the Parquet-backed data table problem without building infrastructure.
- Pro: Largest hiring pool of the three options.
- Con: Language boundary between Rust backend and TypeScript frontend. API contract
  must be maintained via OpenAPI schema generation — drift is a real risk without CI
  enforcement.
- Con: shadcn/ui is copy-paste, not a versioned library. Component layer requires
  architectural discipline (the `ui/` package contract) to remain maintainable.
- Con: React 19's ecosystem is in transition (concurrent features, compiler). Dependency
  pinning and upgrade testing are required.

---

### Option B: SvelteKit (TypeScript)

**Description:** SvelteKit with TypeScript. Reactive stores for real-time signals.
SSR out of the box. Graph rendering via svelvet or D3 integration.

**Proponent:** Alex Rivera (PM)

- Pro: Svelte's reactivity model is closer to the metal than React for real-time
  signal surfaces. Less boilerplate.
- Pro: SSR built in. Smaller bundle sizes than React.
- Pro: Good developer ergonomics; faster authoring velocity for straightforward views.
- Con: Graph rendering gap is a wall, not a gap. svelvet is a toy library compared
  to React Flow. D3 in Svelte is unergonomic — D3 wants imperative DOM mutation;
  Svelte wants declarative reactive templates. Building a graph renderer is 3–6 weeks
  of infrastructure work before a single visual node renders.
- Con: Thinner a11y ecosystem. Radix and Axe have no direct Svelte equivalents.
- Con: Smaller talent pool than React. Staffing risk for a small team.

*Note: Alex Rivera withdrew strong advocacy for this option during Phase 3 (Structured
Challenge) after acknowledging the graph rendering gap was blocking, not deferrable.*

---

### Option C: Leptos (Rust/WASM)

**Description:** Full-stack Rust. Leptos compiles to WebAssembly. Shared types between
`authority-domain` and UI components. Single language across the entire codebase.

**Proponent:** Dmitri Volkov (Backend)

- Pro: Shared types between Rust backend and WASM frontend. The compiler enforces the
  API boundary — a renamed field in `authority-domain` breaks the UI at compile time,
  not at runtime.
- Pro: Single language. Engineers who know Rust can contribute to the full stack.
- Pro: Signals-based reactivity. Fast WASM execution.
- Con: No graph rendering library. `MOD-VISUAL-MAP` is blocked — permanently, not
  temporarily. No React Flow equivalent exists or is planned.
- Con: No Testing Library equivalent. WCAG AA component testing is not addressable
  with available tooling.
- Con: No React Query equivalent. Paginated, streaming, cache-invalidated data tables
  require 2–3 weeks of infrastructure before a single data row renders.
- Con: Ecosystem measured in hundreds of engineers globally, not hundreds of thousands.
  Library bugs may require upstream contributions rather than workarounds.

*Note: Dmitri Volkov acknowledged all challenges as valid during Phase 3.*

---

### Shell Strategy: Web-first vs. Desktop (Tauri)

Three shell positions were on the table:

- **Web-first (browser SPA):** Maximum reach. No install. Works everywhere. Simplest
  to ship. Tightest browser sandbox security posture.
- **Desktop-first (Tauri):** Native integration, system tray, local file access,
  offline capability. Adds complexity and limits web distribution.
- **Web-first, Tauri-wrapper later:** Ship web app first. Wrap in Tauri when a specific
  native capability is needed. No commitment now.

Fatima Al-Hassan (Security Architect) raised: desktop shell changes the threat model.
Local file access in a compromised component can read the filesystem. A browser-sandboxed
web app is more constrained. Web-first is the correct security posture for a governance
tool. No challenge was raised to this position.

---

## Decision

**React (Vite + React 19 + TypeScript)** for all RealmForge frontend surfaces.
**Web-first shell** (browser SPA). No Tauri commitment.

Component foundation: **shadcn/ui** (internal implementation detail) wrapped by a named
**`ui/` package** (the public component API — `<BoardCard>`, `<PacketStatusBadge>`,
`<AuditEventRow>`, etc.).

Graph rendering: **React Flow**.

API consumption: **OpenAPI-generated TypeScript client** from the Axum REST API.
Generation must be automated in CI — no manual drift.

---

## Rationale

The Visual Map requirement is the decisive factor. `MOD-VISUAL-MAP` is a first-class
product module referenced by every Board view through visual node IDs (`VN-*`) and edge
IDs (`VE-*`). React Flow is the only production-ready, accessible, TypeScript-native graph
rendering library in this class. Svelte has no equivalent. Leptos has nothing. No other
factor in the decision overrides this.

The secondary factors reinforce the same conclusion: Radix UI + Axe give React the most
mature WCAG AA testing ecosystem without building from scratch. React Query solves
paginated/streaming data tables. The talent pool is the largest.

Leptos is architecturally elegant — the shared-type story across `authority-domain` and the
WASM frontend is genuinely superior — but the ecosystem is not mature enough today for a
product of this surface area. Decisions are made on ecosystem evidence, not ecosystem
promises.

SvelteKit's reactivity advantages are real but insufficient to overcome the graph rendering
wall when Visual Map is not deferrable.

The web-first shell decision follows Fatima's threat model analysis: a browser sandbox is
a tighter security posture than a Tauri native shell for a governance product. Tauri may
be introduced later, subject to a new Council session and Fatima's delta threat model review.

---

## Consequences

**Positive:**
- `MOD-VISUAL-MAP` is unblocked. Boards vertical, graph node/edge rendering, and the
  visual map spec can all begin immediately.
- WCAG AA testing infrastructure is available from day one (Radix, Axe, Testing Library).
- React Query handles all Parquet-backed paginated/streaming data views without
  building cache infrastructure.
- Largest hiring pool for frontend talent.
- React ecosystem has well-established patterns for real-time SSE/WebSocket integration
  covering Build Watch and Live Watch signal surfaces.

**Negative / Trade-offs:**
- Language boundary between Rust backend and TypeScript frontend. The API contract
  must be maintained via OpenAPI schema generation — if CI does not enforce this,
  the types will drift.
- No shared types between `authority-domain` and the UI layer. A renamed domain field
  is a silent API break unless caught by contract tests or schema generation.
- shadcn/ui copy-paste model requires disciplined `ui/` package ownership. The public
  component API must be documented and enforced before implementation begins or it
  becomes informal.

**Risks:**
- **React 19 transition risk.** The React 19 ecosystem (compiler, concurrent features)
  is in flux. Pin React and its ecosystem dependencies; test upgrades explicitly.
- **OpenAPI drift.** If TypeScript client generation is not automated in CI, the frontend
  types will diverge from the Rust API within weeks. Marcus (API Architect) owns this
  pipeline as a hard dependency for all frontend work.
- **`ui/` package erosion.** If engineers bypass the `ui/` package and use shadcn/ui
  primitives directly in feature code, the component layer becomes unmaintainable.
  Code review must enforce the boundary.
- **Tauri creep.** If the team adds Tauri incrementally without a formal Council decision,
  the threat model delta never gets reviewed. Fatima must be notified before any Tauri
  dependency is introduced.

---

## Dissenting Opinions

**Dmitri Volkov (Backend):**
> "I maintain that full-stack Rust is architecturally superior. The shared-type approach
> across `authority-domain` and a Leptos WASM frontend avoids the language boundary entirely.
> Leptos will be mature enough within 18 months. This decision introduces a type gap that
> will cost debugging time at the API boundary. I accept the decision. My dissent is on record."

**Alex Rivera (PM):**
> "SvelteKit's developer experience advantages and reactivity model are real and we are
> leaving them on the table. The graph rendering gap was the deciding factor and I accept
> that assessment. My preference for Svelte's reactivity model is noted but not sufficient
> to override the Visual Map requirement."

---

## Follow-up Actions

| Owner | Action | Dependency for |
|-------|--------|----------------|
| Marcus (API Architect) | Define and implement OpenAPI → TypeScript client generation pipeline, automated in CI | All frontend API consumption |
| Kai (Frontend) | Draft frontend vertical spec: stack, folder structure, `ui/` package contract, React Flow integration plan, accessibility baseline | Boards implementation |
| Clara (Tech Writer) | Document `ui/` package component contract once Kai's spec is approved | Frontend code review gate |
| Fatima (Security Architect) | Record web-first shell decision in threat model; flag any future Tauri proposal for new Council session | Security posture |
| Orchestrator | Mark DEC-COUNCIL-001 CLOSED; unblock Boards vertical, Visual Map spec, Knowledge UI | All frontend work items |

---

## Review Date

2027-05-04. Revisit if: Leptos ecosystem matures significantly, a native capability
requires Tauri evaluation, or the React 19 transition creates ecosystem instability.
