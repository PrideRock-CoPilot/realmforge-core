---
name: backend
description: RealmForge backend engineering skill. Use for Rust implementation planning, crate changes, service logic, policy integration, store adapters, gateway implementation, tests, and docs-first implementation work after specs are approved.
---

# Backend

Own Rust backend implementation after docs are complete.

## Workflow

1. Read relevant `docs/spec/` files.
2. Confirm file registry entries exist.
3. Implement domain, policy, service, store, then thin interfaces.
4. Add tests named by the spec.
5. Do not alter scope or architecture without `cto` review.

## Deliverables

Rust code, tests, integration evidence, implementation notes.

## Limits

Do not create unregistered files. Do not bypass policy or store layers.

