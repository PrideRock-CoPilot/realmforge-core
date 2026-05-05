---
doc_id: DOC-CODEX-003
title: Codex Skill Install Plan
status: draft
owner: realmforge-skill-creator
reviewers: [orchestrator, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: [FILE-DOCS-CODEX-SKILL-INSTALL]
visual_node_ids: [VN-CODEX-SKILL-INSTALL]
visual_edge_ids: [VE-CODEX-INSTALLS-SKILLS]
approval_state: pending
---

# Codex Skill Install Plan

## Install Target

Approved RealmForge skills install to:

```text
C:\Users\amari\.codex\skills\
```

## Source

Project-owned source:

```text
docs/codex/skills-source/
```

## Manual Install Command

From `E:\realmforge`:

```powershell
$target = "C:\Users\amari\.codex\skills"
Get-ChildItem docs\codex\skills-source -Directory | ForEach-Object {
  Copy-Item $_.FullName (Join-Path $target $_.Name) -Recurse -Force
}
```

## Validation

After install, start a fresh Codex session and confirm these skill names appear: `realmforge-skill-creator`, `orchestrator`, `council`, `cto`, `pm`, `tech-writer`, `backend`, `qa`.

## Update Rule

Update `docs/codex/skills-source/` first. Then reinstall. Do not edit installed copies directly unless the change is copied back into source.
