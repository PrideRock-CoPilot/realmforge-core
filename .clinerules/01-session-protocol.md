# Session Start Protocol

This sequence is mandatory before any productive work. Do not skip steps.

## 1. Read Workspace Context

Read these three files first — every session, every time:

- `CLAUDE.md` — the company skill deck, routing rules, and architecture summary
- `AGENTS.md` — engineering constraints, docs-first law, Rust rules
- `docs/MASTER_BUILD_PLAN.md` — current build phase and what is and is not approved

## 2. Check for Blockers

Read `docs/spec/22_OPEN_DECISIONS.md`. Do not implement anything tagged
`COUNCIL_DECISION_REQUIRED` or `USER_APPROVAL_REQUIRED`. If the work you are
about to do depends on an unresolved decision, stop and report it.

## 3. Identify the Skill Domain

Match the user's request to a skill in `02-routing-and-skills.md`. Every request
belongs to at least one domain. If you cannot identify the domain, ask before acting.

## 4. Load the Skill

Read the matching skill file:

```
skills/<skill-name>/SKILL.md
```

Read it fully. Adopt the persona and follow its workflow. Do not act before loading
the skill that owns the domain.

## 5. Confirm No Docs-First Violations

See `03-docs-first-law.md`. If the requested work has no spec, no file registry
entry, or no approved work path, write the missing doc first and stop.

## Violation Recovery

If you realise mid-task that you skipped this protocol:
1. Acknowledge the violation explicitly in your response
2. Read the correct skill file now
3. Re-evaluate whether what you did was within that skill's bounds
4. If not, describe what needs to be undone or corrected
