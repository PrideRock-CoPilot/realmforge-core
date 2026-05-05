# RealmForge Agent Instructions

## Required Reading

Every agent working in `E:\realmforge` must read these files before changing code or specs:

1. `CLAUDE.md`
2. `AGENTS.md`
3. `docs/realm_forge_ai_native_path_forward.md`
4. `docs/spec/00_INDEX.md`
5. Relevant docs under `docs/spec/`
6. Relevant skills under `docs/codex/skills-source/` or installed Codex skills

## Docs-First Law

RealmForge is in docs-first buildout. Do not create or edit implementation files unless the relevant spec, metadata entry, work path, permission grants, public contract, and acceptance tests already exist.

If a build request exposes a missing decision, record it in `docs/spec/22_OPEN_DECISIONS.md` as `USER_APPROVAL_REQUIRED` or `COUNCIL_DECISION_REQUIRED`.

## Skill Discipline

Use the RealmForge company skills installed in Codex when their domain applies:

- `realmforge-skill-creator` for skill creation or skill audits
- `orchestrator` for multi-skill work
- `council` for decisions marked `COUNCIL_DECISION_REQUIRED`
- `cto` for architecture and layer boundaries
- `security-architect` for grants, gateway, and separation of duties
- `domain-architect` for domain language and state models
- `api-architect` for REST, MCP, and CLI contracts
- `data-architect` and `data-engineer` for Postgres, Parquet, and object store contracts
- `backend`, `frontend`, `qa`, `release-manager`, `tech-writer`, `pm`, `ceo`, `biz-user`, and `accountant` for their documented domains

## Current Phase 0 Constraints

- Allowed: docs, instructions, skill setup, and unambiguous Rust hardening that already matches approved docs.
- Blocked pending Council: frontend stack, Parquet engine/library.
- Blocked pending user approval: modules beyond Login, exact brand-removal threshold, Live Watch auto-remediation.

## Rust Rules

- Crate/package names follow accepted decision `docs/decisions/DEC-COUNCIL-002-backend-crate-package-names.md`.
- Rust workspace files now live at the root (`Cargo.toml`, `crates/`, `db/`) per Phase 1 flattening.
- Do not add broad agent powers.
- Agents must default to zero capabilities without active skill grants.
- Avoid incomplete production stubs. If an operation cannot be implemented honestly yet, return a typed error and document the blocked spec decision.
- Keep API/MCP/CLI thin over service logic.
