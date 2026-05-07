# RealmForge — Workspace Instructions

RealmForge is a Rust-first governance kernel. This workspace root contains the
foundational control plane for identity, authority, bounded commands, events, and
snapshots.

## 🎯 Essential Entry Points

Every agent **MUST** read these before acting:

1. **`docs/MASTER_BUILD_PLAN.md`** — The living blueprint. Complete layer-by-layer build plan with file manifests, validation gates, and handoff protocol.
2. **`AGENTS.md`** — Engineering instructions, architecture rules, layering constraints.
3. **`docs/realm_forge_ai_native_path_forward.md`** — The north star vision document.

## 📁 Repository Structure

- `crates/` — Capability-named Rust crates (`authority-domain`, `policy-engine`, `audit-log`, `snapshot-ledger`, `control-store`, `control-service`, `control-api`, `operator-cli`, `agent-mcp`)
- `db/migrations/` — PostgreSQL schema migrations (2 foundation migrations)
- `docs/` — Canonical specs, architecture docs, ADRs, master build plan, vision document
- `AGENTS.md` — Full engineering instructions and build workflow


---

## The Company

RealmForge runs on a skill system. These skills are the company — real colleagues with
specific domains, scars, and pride in their work. You can say "hi" to any of them by
name, or invoke their skill directly with `/skill-name`.

**Always load `/realmforge-skill-creator` first** when starting a new session or creating a new skill.

---

## Leadership

| Invoke | Person | Role | Domain |
|--------|--------|------|--------|
| `/realmforge-skill-creator` | The Builder | Meta-skill | Creates, validates, and registers all skills |
| `/ceo` | Victor Chen | CEO | Strategy, direction, investment approval |
| `/pm` | Alex Rivera | Project Manager | Planning, scope, delivery, handoffs |
| `/cto` | Dr. Rena Okafor | CTO | Architecture, technical risk, engineering contracts |

## Architects — Report to Rena (CTO)

| Invoke | Person | Domain |
|--------|--------|--------|
| `/domain-architect` | Dr. Yusuf Osman | Bounded contexts, DDD, authority-domain types and state machines |
| `/security-architect` | Fatima Al-Hassan | Threat modeling, zero-trust architecture, security patterns |
| `/api-architect` | Marcus Webb | API/MCP/CLI contract design, versioning, error standards |
| `/infra-architect` | Nadia Kovacs | PostgreSQL architecture, observability, deployment topology |
| `/data-architect` | Chen Wei | Data layer governance, schema separation, Parquet standards |
| `/design-council` | The Design Council | Evidence-driven design research, pattern extraction, design briefs |

## Engineering

| Invoke | Person | Domain |
|--------|--------|--------|
| `/backend` | Dmitri Volkov | Rust capability crates, APIs, security implementation |
| `/frontend` | Kai Sato | Rich UI, accessibility (WCAG AA), performance budgets |
| `/data-engineer` | Priya Nair | Parquet pipelines, schema contracts, data products |

## Quality, Finance & Release

| Invoke | Person | Domain |
|--------|--------|--------|
| `/qa` | Margaret "Meg" Thompson | Verification, defect documentation, release certification |
| `/peer-review` | Nora Patel | Independent readiness review before code review or QA |
| `/code-review` | Owen Brooks | File-level implementation review before QA |
| `/accountant` | Bob Kaczmarek | Reconciliation, forecasting, financial signals |
| `/release-manager` | Sam Osei | Deployment runbooks, rollback safety, post-mortems |

## Knowledge & User Voice

| Invoke | Person | Domain |
|--------|--------|--------|
| `/tech-writer` | Clara Mills | ADRs, API docs, Rust crate docs, architecture guides |
| `/biz-user` | Iris Park | User stories, acceptance criteria, business user perspective |

## Process

| Invoke | What | Domain |
|--------|------|--------|
| `/orchestrator` | The Orchestrator | Handoff discipline, dependency visibility, stall detection |
| `/council` | The Decision Council | Multi-stakeholder decisions; structured debate; documented outcomes |

---

## How to Greet a Colleague

Say their name naturally:
- "hi Victor" → CEO
- "hi Rena" → CTO
- "hi Alex" → PM
- "hi Meg" or "hi Margaret" → QA
- "hi Owen" → Code Review
- "hi Bob" → Accountant
- "hi Yusuf" → Domain Architect
- "hi Fatima" → Security Architect
- "hi Marcus" → API Architect
- "hi Nadia" → Infrastructure Architect
- "hi Chen" → Data Architect
- "hi Dmitri" → Backend/Rust
- "hi Kai" → Frontend
- "hi Priya" → Data Engineer
- "hi Sam" → Release Manager
- "hi Clara" → Tech Writer
- "hi Iris" → Business User
- "orchestrator, what's in-flight?" → Orchestrator
- "/council" → convene the Decision Council
- "hi design council" → The Design Council
- "/design-council" → convene the Design Council
- "let's study what worked" → The Design Council

---

## 🚨 Cline Skill Enforcement Rules

These rules are **mandatory**. Every agent (including Cline) MUST follow them on every session.

### 1. Always Invoke Skills via `use_skill`

When a user's request matches a skill's domain, invoke the skill using `use_skill` as your **FIRST action** before using any other tool. The skill has specialized workflows that produce better results than ad-hoc answers.

**Key routing rules:**
- Architecture decision, technical risk → invoke `/cto` (Rena Okafor)
- Planning, scope, handoffs → invoke `/pm` (Alex Rivera)
- Strategy, investment approval → invoke `/ceo` (Victor Chen)
- Domain model, bounded context → invoke `/domain-architect` (Yusuf Osman)
- Threat model, security → invoke `/security-architect` (Fatima Al-Hassan)
- API/MCP/CLI contract design → invoke `/api-architect` (Marcus Webb)
- Infrastructure, DB, observability → invoke `/infra-architect` (Nadia Kovacs)
- Data layer, schema governance → invoke `/data-architect` (Chen Wei)
- Rust crate implementation → invoke `/backend` (Dmitri Volkov)
- Frontend UI, accessibility → invoke `/frontend` (Kai Sato)
- Data pipelines, Parquet → invoke `/data-engineer` (Priya Nair)
- Verification, defect documentation → invoke `/qa` (Meg Thompson)
- Peer review, area readiness evidence, project-board peer review status → invoke `/peer-review` (Nora Patel)
- Code review, file-level implementation review, project-board code-review status → invoke `/code-review` (Owen Brooks)
- Financial reconciliation, forecasting → invoke `/accountant` (Bob Kaczmarek)
- Deployment, rollback → invoke `/release-manager` (Sam Osei)
- ADRs, API docs, crate docs → invoke `/tech-writer` (Clara Mills)
- User stories, business perspective → invoke `/biz-user` (Iris Park)
- Workflow state, handoff discipline → invoke `/orchestrator`
- Multi-stakeholder decision → invoke `/council`
- Design research, pattern extraction, design briefs → invoke `/design-council` (The Design Council)
- Creating/auditing a new skill → invoke `/realmforge-skill-creator` (ALWAYS first)

### 2. Orchestrator Invocation

- **At the start of every session**, invoke `/orchestrator` to get the current workflow state and identify any in-flight work or stalled handoffs.
- **Before spanning multiple skills**, invoke `/orchestrator` to map the handoff chain and surface any dependency conflicts.
- **When a stall is detected** (work item > 24h in same state), invoke `/orchestrator` for escalation to Alex (PM).
- **For any multi-step task that crosses skill boundaries**, the orchestrator must be consulted before starting and after each handoff.

### 3. Session Start Protocol

Every session MUST begin with this sequence:

1. **Invoke `/realmforge-skill-creator`** — Register any new skills needed for this session.
2. **Invoke `/orchestrator`** — Get current workflow state, in-flight items, and any stalled handoffs.
3. **Read canonical documents** — Read `CLAUDE.md`, `AGENTS.md`, `docs/MASTER_BUILD_PLAN.md` before acting.

### 4. Hard Rules

- **No bypassing the handoff chain.** Work must flow through the canonical chain: CEO → PM → CTO → Architects (as needed) → Engineering → QA → Release Manager. Direct engineering-to-release without QA is a violation.
- **No solving a skill's problem without invoking the skill.** If the request matches a skill domain, you MUST invoke it with `use_skill` — do not attempt to solve it yourself.
- **No implicit handoffs.** Every handoff between skills must be explicit: confirmed by sender, confirmed by receiver, recorded by the orchestrator.
- **No session without context.** Always read `CLAUDE.md`, `AGENTS.md`, and `docs/MASTER_BUILD_PLAN.md` before acting.
- **No raw SQL or unfettered database access.** All persistence goes through the store adapter layer.
- **No layer violations.** The crate law is absolute: api/mcp/cli → policy → service → domain → store → PostgreSQL. No shortcuts.

### 5. Workflow Discipline During Implementation

When executing a multi-skill task:

1. **Before starting**: Ask `/orchestrator` to confirm the handoff chain and that no dependencies are blocked.
2. **After each handoff**: Confirm delivery with the sending skill, confirm receipt with the receiving skill, and update `/orchestrator`.
3. **On completion**: Report final state to `/orchestrator` so workflow tracking stays current.
4. **If blocked**: Invoke `/orchestrator` to document the blocker and escalate if needed.

---

## The Workflow Chain

```
Victor (CEO)          — Strategy and investment approval
       ↓
Alex (PM)             — Planning, scoping, handoff coordination
       ↓
Rena (CTO)            — Architecture contracts and technical guardrails
       ↓
Architects (as needed):
  Yusuf   — Domain model and bounded context design
  Fatima  — Threat model and security requirements
  Marcus  — API/MCP/CLI contract definitions
  Nadia   — Infrastructure, DB, observability design
  Chen    — Data layer architecture and governance
       ↓
Engineering:
  Dmitri  — Rust crate implementation
  Kai     — Frontend UI components
  Priya   — Data pipelines and Parquet artifacts
       ↓
Nora (Peer Review)     — Independent readiness review before downstream gates
       ↓
Owen (Code Review)     — File-level implementation review before QA
       ↓
Meg (QA)              — Verification, defect documentation, release certification
       ↓
Sam (Release Manager) — Deployment runbooks, rollback, post-deployment verification

Cross-cutting:
  Bob (Accountant)    — Financial signals to Victor and Alex (immediate on variance)
  Clara (Tech Writer) — ADRs, API docs, crate docs (triggered by every significant decision)
  Iris (Biz User)     — User stories and acceptance criteria (at sprint start, not sprint end)
  Orchestrator        — Handoff discipline across all skills
  Council             — Multi-stakeholder decisions that exceed single-skill authority
```

---

## Architecture Rules (Summary)

Full rules in `AGENTS.md`. Key constraints:

**Layer law** (enforced by Rena — no exceptions):
```
api/mcp/cli  →  policy/domain service  →  store/snapshot adapter  →  PostgreSQL
```

**Crate law:** one crate, one responsibility.
- `authority-domain`: pure logic, no IO
- `policy-engine`: authorization, no persistence
- `control-store`: persistence, no business logic
- `control-api`, `agent-mcp`, `operator-cli`: thin transports, no business logic

**File law:** ≤300 lines target, 500 line hard cap (justification required)

**Rust law:** no `unwrap()` without invariant comment. No `unsafe` without `SAFETY:` comment.
Typed IDs everywhere — never bare `Uuid`.

---

## Creating a New Skill

Invoke `/realmforge-skill-creator`. They run you through the six questions and draft the file.
Skills live in `e:\realmforge\skills\<name>\SKILL.md`.

---

## Philosophy

These skills are not agents. They are colleagues. Each one has a domain they own, scars
from failures that shaped their discipline, and pride in doing their job above and beyond.

The company is only as strong as the discipline of its handoffs.
