# Skill Routing

When a request matches a domain below, read the skill file before acting.
Skill files live in `skills/<name>/SKILL.md`.

## Routing Table

| Request type | Skill to load | File path |
|---|---|---|
| Architecture decision, technical risk, layer boundary | `cto` | `skills/cto/SKILL.md` |
| Sprint planning, scope, handoffs, priorities | `pm` | `skills/pm/SKILL.md` |
| Strategy, investment, company direction | `ceo` | `skills/ceo/SKILL.md` |
| Domain model, bounded context, state machines | `domain-architect` | `skills/domain-architect/SKILL.md` |
| Threat model, security patterns, zero-trust | `security-architect` | `skills/security-architect/SKILL.md` |
| REST / MCP / CLI contract design | `api-architect` | `skills/api-architect/SKILL.md` |
| PostgreSQL, observability, deployment | `infra-architect` | `skills/infra-architect/SKILL.md` |
| Data governance, Parquet, schema | `data-architect` | `skills/data-architect/SKILL.md` |
| Rust crate implementation, API, policy, store | `backend` | `skills/backend/SKILL.md` |
| Frontend UI, accessibility, performance | `frontend` | `skills/frontend/SKILL.md` |
| Parquet pipelines, data products | `data-engineer` | `skills/data-engineer/SKILL.md` |
| Verification, defect docs, release certification | `qa` | `skills/qa/SKILL.md` |
| Deployment runbooks, rollback, post-mortems | `release-manager` | `skills/release-manager/SKILL.md` |
| ADRs, API docs, Rust crate docs | `tech-writer` | `skills/tech-writer/SKILL.md` |
| User stories, acceptance criteria | `biz-user` | `skills/biz-user/SKILL.md` |
| Financial reconciliation, forecasting | `accountant` | `skills/accountant/SKILL.md` |
| Workflow state, handoff tracking, stall detection | `orchestrator` | `skills/orchestrator/SKILL.md` |
| Multi-stakeholder contested decisions | `council` | `skills/council/SKILL.md` |
| Creating or auditing skills | `realmforge-skill-creator` | `skills/realmforge-skill-creator/SKILL.md` |

## Rules

- Read the skill file **before** writing any code, creating any file, or making any decision
- If the request spans three or more domains, load `orchestrator` first
- Do not solve a skill's problem without loading that skill — this is wandering
- A skill's "What You Don't Touch" section is a hard limit — escalate, do not cross it
