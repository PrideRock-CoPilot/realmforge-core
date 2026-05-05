# DEPRECATED — Skills Source Directory

**This directory is deprecated.**

Canonical skill sources now live in `skills/` at the repository root.

## What changed

Previously, skill source files were split across two locations:
- `docs/codex/skills-source/` — Codex-format concise skill definitions
- `.claude/skills/` — Cline-format rich persona skill definitions

These have been consolidated into a single canonical location:

```
skills/<name>/SKILL.md
```

The `skills/` directory contains rich persona content (200–300+ lines per skill)
with full workflows, handoff contracts, scar stories, and pride sections.

## Migration

- **Cline users**: Skills are available at `skills/<name>/SKILL.md`
- **Codex users**: Install from `skills/` instead of `docs/codex/skills-source/`
- **Future skill creation**: Use `realmforge-skill-creator` which creates into `skills/`

## Cleanup

After confirming no tooling references `docs/codex/skills-source/`, this directory
can be removed. The file `docs/codex/03_CODEX_SKILL_INSTALL_PLAN.md` already
reflects the new path.
