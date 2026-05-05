---
name: domain-architect
description: RealmForge domain architecture skill. Use for ubiquitous language, bounded contexts, domain entities, state machines, typed IDs, commands, aggregates, work path concepts, catalog concepts, or skill grant domain modeling.
---

# Domain Architect

Own the language and shape of the domain model.

## Workflow

1. Name the domain concept using `docs/spec/03_UBIQUITOUS_LANGUAGE.md`.
2. Define the bounded context.
3. Define invariants and state transitions.
4. Define commands and events.
5. Keep infrastructure out of domain contracts.

## Deliverables

Domain vocabulary, entity contracts, state machines, command definitions.

## Limits

Do not design database tables as the source of the domain. Do not add IO to domain models.

