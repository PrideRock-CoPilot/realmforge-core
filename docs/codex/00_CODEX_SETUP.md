---
doc_id: DOC-CODEX-000
title: Codex Setup For RealmForge
status: draft
owner: realmforge-skill-creator
reviewers: [orchestrator, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: [FILE-DOCS-CODEX-SETUP]
visual_node_ids: [VN-CODEX-SETUP]
visual_edge_ids: [VE-CODEX-USES-SKILLS]
approval_state: pending
---

# Codex Setup For RealmForge

## Purpose

This folder defines how Codex should operate on RealmForge. It mirrors the RealmForge company skill discipline from `.claude/skills/` while using Codex-native skill folders.

## Current State

Codex has system skills installed under `C:\Users\amari\.codex\skills\.system`. The RealmForge company skills were installed from this source tree during Phase 0 setup.

Project-owned Codex skill sources live in:

```text
docs/codex/skills-source/
```

Approved skill folders are copied into:

```text
C:\Users\amari\.codex\skills\
```

Do not install `realmforge-skill-creator` as `skill-creator`; Codex already has a system skill with that name.

## Setup Sequence

1. Review every `docs/codex/skills-source/*/SKILL.md`.
2. Confirm each skill has front matter with only `name` and `description`.
3. Copy each approved folder to `C:\Users\amari\.codex\skills\`.
4. Start a fresh Codex session.
5. Confirm the skill list includes the RealmForge skills.

## Source Mapping

Claude source skills remain in `.claude/skills/`. Codex skill sources in `docs/codex/skills-source/` are concise operational versions derived from those files.
