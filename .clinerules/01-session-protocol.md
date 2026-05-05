# Session Start Protocol

This sequence is mandatory before any productive work. Do not skip steps.

## 1. Read Workspace Context

Read these workspace files and rule files first — every session, every time:

- `CLAUDE.md` — the company skill deck, routing rules, and architecture summary
- `AGENTS.md` — engineering constraints, docs-first law, Rust rules
- `docs/MASTER_BUILD_PLAN.md` — current build phase and what is and is not approved
- `.clinerules/03-docs-first-law.md` — specs, metadata, work paths, grants, and tests before implementation
- `.clinerules/04-architecture-laws.md` — layer law, crate law, Rust rules, and file-size limits
- `.clinerules/05-workflow-and-handoffs.md` — canonical handoff chain and orchestrator discipline
- `.clinerules/06-repo-hygiene-and-verification.md` — clean repo, docs freshness, and verification rules
- `.clinerules/07-focused-workflow-lifecycle.md` — Idea → Planning → Architecture → Development → Testing → Documentation flow

## 2. Check for Blockers

Read `docs/spec/22_OPEN_DECISIONS.md`. Do not implement anything tagged
`COUNCIL_DECISION_REQUIRED` or `USER_APPROVAL_REQUIRED`. If the work you are
about to do depends on an unresolved decision, stop and report it.

## 3. Check Repository Hygiene

Follow `06-repo-hygiene-and-verification.md` before edits:

- Run `git status --short`
- Run `git ls-files --others --exclude-standard`
- Separate pre-existing dirty files from files this task will touch
- Do not clean or revert user work unless explicitly requested

## 4. Identify the Lifecycle Stage

Follow `07-focused-workflow-lifecycle.md`:

- Name the current stage: Idea, Planning, Architecture, Development, Testing, or Documentation
- Identify the required artifact for that stage
- Do not move to a later stage until the current stage's exit gate is satisfied

## 5. Identify the Skill Domain

Match the user's request to a skill in `02-routing-and-skills.md`. Every request
belongs to at least one domain. If you cannot identify the domain, ask before acting.

## 6. Load the Skill

Read the matching skill file:

```
skills/<skill-name>/SKILL.md
```

Read it fully. Adopt the persona and follow its workflow. Do not act before loading
the skill that owns the domain.

## 7. Confirm No Governance Violations

Check `03-docs-first-law.md`, `04-architecture-laws.md`, and `05-workflow-and-handoffs.md`.
If the requested work has no spec, file registry entry, approved work path, grant, or test, write the missing doc first and stop. If it would violate layer law or skip a required handoff, route to `cto` or `orchestrator` before acting.

## Violation Recovery

If you realise mid-task that you skipped this protocol:
1. Acknowledge the violation explicitly in your response
2. Read the correct skill file now
3. Re-evaluate whether what you did was within that skill's bounds
4. If not, describe what needs to be undone or corrected
