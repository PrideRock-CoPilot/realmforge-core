---
doc_id: DOC-CODEX-001
title: Codex Session Protocol
status: draft
owner: orchestrator
reviewers: [realmforge-skill-creator, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: [FILE-DOCS-CODEX-SESSION-PROTOCOL]
visual_node_ids: [VN-CODEX-SESSION-PROTOCOL]
visual_edge_ids: [VE-CODEX-READS-SPEC]
approval_state: pending
---

# Codex Session Protocol

## Required Reading Order

Every Codex session working on RealmForge must read:

1. `CLAUDE.md`
2. `AGENTS.md`
3. `docs/realm_forge_ai_native_path_forward.md`
4. `docs/spec/00_INDEX.md`
5. Relevant skill docs from `docs/codex/skills-source/` or installed Codex skills

## Operating Rules

- Treat `docs/spec/` as the canonical source of truth.
- Do not implement code before the relevant spec, metadata, file registry entry, work path, tests, and grant rules exist.
- Use `realmforge-skill-creator` for RealmForge company skill creation or audit.
- Use `orchestrator` before multi-skill work.
- Use `council` for decisions listed in `docs/spec/22_OPEN_DECISIONS.md`.
- Record implementation-blocking uncertainty in `docs/spec/22_OPEN_DECISIONS.md`.

## Docs-First Rule

If asked to implement a feature and the required spec is missing, write or update the spec first. Then stop and report the spec gap closure unless the user explicitly asks to continue into buildout after documentation approval.

## Safety Rule

Codex must not use prompt-only instructions as a security boundary. Agent permissions are valid only when represented as skill grants, file scopes, command actions, evidence requirements, and audit events in the spec.
