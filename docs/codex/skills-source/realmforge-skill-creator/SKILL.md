---
name: realmforge-skill-creator
description: Create, audit, and maintain RealmForge company skills for Codex without colliding with Codex's system skill-creator. Use when a task involves RealmForge skill creation, skill review, skill mapping from Claude, role boundaries, handoff contracts, or installing project-owned skills into Codex.
---

# RealmForge Skill Creator

Use this skill to keep RealmForge's company skill deck sharp, bounded, and compatible with Codex.

## Rules

- Keep one responsibility per skill.
- Use concise Codex-native instructions.
- Put project-owned skill sources under `docs/codex/skills-source/<skill-name>/SKILL.md`.
- Use `realmforge-skill-creator` instead of `skill-creator` to avoid the Codex system skill name.
- Do not add auxiliary files unless the user explicitly asks.

## Skill Checklist

- Single responsibility.
- Clear trigger in front matter.
- Named upstream and downstream handoff.
- Owned deliverables.
- Hard limits.
- RealmForge docs-first behavior when applicable.

## Company Deck

Leadership: `ceo`, `pm`, `cto`.
Architects: `domain-architect`, `security-architect`, `api-architect`, `infra-architect`, `data-architect`.
Engineering: `backend`, `frontend`, `data-engineer`.
Quality and release: `qa`, `accountant`, `release-manager`.
Knowledge and voice: `tech-writer`, `biz-user`.
Process: `orchestrator`, `council`.

