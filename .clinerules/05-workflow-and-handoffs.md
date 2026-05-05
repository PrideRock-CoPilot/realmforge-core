# Workflow Chain and Handoff Discipline

For stage-by-stage execution, also follow `07-focused-workflow-lifecycle.md`.

## The Canonical Handoff Chain

Work must flow through this chain. Skipping a step is a violation.

```
Victor (CEO)           — strategy and investment approval
       ↓
Alex (PM)              — planning, scope, handoff coordination
       ↓
Rena (CTO)             — architecture contracts and technical guardrails
       ↓
Architects (as needed) — Yusuf, Fatima, Marcus, Nadia, Chen
       ↓
Engineering            — Dmitri (Rust), Kai (Frontend), Priya (Data)
       ↓
Nora (Peer Review)     — area readiness and evidence review
       ↓
Owen (Code Review)     — file-level implementation review
       ↓
Meg (QA)               — verification, defect docs, release certification
       ↓
Sam (Release Manager)  — deployment, rollback, post-deployment verification
```

Cross-cutting (triggered at their own points):
- **Bob (Accountant)** — financial signals to CEO and PM
- **Clara (Tech Writer)** — ADRs and docs triggered by every significant decision
- **Iris (Biz User)** — user stories at sprint start, acceptance criteria
- **Orchestrator** — handoff discipline across all skills
- **Council** — decisions that exceed single-skill authority

## Orchestrator Rules

- Load `orchestrator` at the start of every multi-skill task
- Before spanning skills: describe the handoff chain and confirm no dependencies are blocked
- After each skill completes: confirm delivery and update workflow state
- A handoff is not complete until the receiver confirms receipt — a status change is not a handoff

## Multi-Skill Work Protocol

1. Load `orchestrator` first
2. Map the full handoff chain for the task
3. Surface any blocked dependencies before starting
4. Execute each skill in chain order — do not parallelize across skill boundaries
5. Confirm each handoff explicitly before moving to the next skill

## Hard Limits

- Engineering → Release without QA sign-off is a violation
- Any handoff without a named deliverable and named receiver is a violation
- Implicit handoffs ("someone will handle it") are violations
- Scope additions mid-task require the PM to name what moves or waits
