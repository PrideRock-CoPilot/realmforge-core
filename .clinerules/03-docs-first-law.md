# Docs-First Law and Metadata Requirements

## The Law

Do not create or edit any implementation file unless ALL of the following already exist:

1. **Spec document** — a file in `docs/spec/` covering this feature
2. **File registry entry** — a row in the `FILE-*` table in `docs/spec/04_METADATA_STANDARD.md`
3. **Work path ID** — a `WP-*` identifier referenced in the spec and the file entry
4. **Permission grants** — `allowed_skill_grants` defined in the file entry
5. **Acceptance tests** — `required_tests` entries defined in the file entry

If any of these are missing, write the missing documentation first, then stop.
Do not continue into implementation until the user approves the documentation.

## What to Do When Blocked

If a build request requires something that is not yet approved or specified:

1. Record the missing decision in `docs/spec/22_OPEN_DECISIONS.md` with status
   `USER_APPROVAL_REQUIRED` or `COUNCIL_DECISION_REQUIRED`
2. Report what is missing and what decision is needed
3. Do not work around the block

## Metadata Requirements for Docs Files

Every `docs/**/*.md` file must begin with a YAML frontmatter block:

```yaml
---
doc_id: DOC-XXX-NNN
title: "Human-readable title"
status: draft | active | accepted | deprecated
owner: <skill-name>
reviewers: [skill-name, ...]
created_at: YYYY-MM-DD
last_reviewed_at: YYYY-MM-DD
source_of_truth: true | false
product_area: <area>
work_path_ids: [WP-XXX-NNN]
related_decision_ids: []
related_file_ids: []
visual_node_ids: []
visual_edge_ids: []
approval_state: pending | accepted
---
```

Do not create a `docs/**/*.md` file without this block. This is enforced by the
pre-commit hook and the Claude Code `PreToolUse` hook in `.claude/hooks/`.

## Implementation File Registry

Every implementation file needs a registry entry before it is created.
See `docs/spec/04_METADATA_STANDARD.md` for the full table and schema.
The `artifact_class`, `bounded_context`, `owning_module`, `risk_level`,
`allowed_skill_grants`, and `required_tests` fields are all mandatory.
