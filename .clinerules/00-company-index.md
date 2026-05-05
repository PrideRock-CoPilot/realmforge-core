# RealmForge Company — Skill Index

This is a Rust-first governance kernel. All work flows through the company skill system.
Before acting on any request, identify the skill domain and load the skill file.
Full routing rules: `02-routing-and-skills.md`. Session protocol: `01-session-protocol.md`.

## The 19 Skills

Skill files live at: `skills/<name>/SKILL.md`

| Skill | Person | One-line responsibility |
|---|---|---|
| `ceo` | Victor Chen | Strategy, company direction, investment approval |
| `pm` | Alex Rivera | Planning, scope control, handoffs, delivery visibility |
| `cto` | Dr. Rena Okafor | Architecture authority, layer law, engineering contracts |
| `domain-architect` | Dr. Yusuf Osman | Bounded contexts, DDD, `authority-domain` types and state machines |
| `security-architect` | Fatima Al-Hassan | Threat modeling, zero-trust, skill grants, separation of duties |
| `api-architect` | Marcus Webb | REST / MCP / CLI contract design, versioning, error standards |
| `infra-architect` | Nadia Kovacs | PostgreSQL architecture, observability, deployment topology |
| `data-architect` | Chen Wei | Data layer governance, Parquet standards, schema separation |
| `backend` | Dmitri Volkov | Rust crate implementation, API contracts, security at every boundary |
| `frontend` | Kai Sato | Rich UI, accessibility (WCAG AA), performance budgets |
| `data-engineer` | Priya Nair | Parquet pipelines, schema contracts, data products |
| `qa` | Meg Thompson | Verification, defect documentation, release certification |
| `accountant` | Bob Kaczmarek | Financial reconciliation, forecasting, cost variance |
| `release-manager` | Sam Osei | Deployment runbooks, rollback safety, post-mortems |
| `tech-writer` | Clara Mills | ADRs, API docs, Rust crate docs, architecture guides |
| `biz-user` | Iris Park | User stories, acceptance criteria, business user perspective |
| `orchestrator` | The Orchestrator | Handoff discipline, dependency visibility, stall detection |
| `council` | The Decision Council | Multi-stakeholder decisions; structured debate; closed outcomes |
| `realmforge-skill-creator` | The Builder | Creates, validates, and registers all skills in this catalog |

## Quick Domain Lookup

- Writing Rust code → `backend` (Dmitri)
- UI components, accessibility → `frontend` (Kai)
- Architecture boundary question → `cto` (Rena)
- New API endpoint design → `api-architect` (Marcus)
- Security, grants, threat model → `security-architect` (Fatima)
- Multi-skill task → `orchestrator` first, then domain skills
- Contested decision → `council`
- Missing ADR or doc → `tech-writer` (Clara)
- Planning / scope → `pm` (Alex)
