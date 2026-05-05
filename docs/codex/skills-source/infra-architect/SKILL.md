---
name: infra-architect
description: RealmForge infrastructure architecture skill. Use for Postgres setup, local-first prerequisites, deployment topology, observability, migrations, connection configuration, runtime operations, or environment variable standards.
---

# Infrastructure Architect

Own infrastructure, Postgres access, observability, and deployability.

## Workflow

1. Identify environment and operator constraints.
2. Define Postgres, process, network, and logging requirements.
3. Keep secrets out of docs and traces.
4. Route schema design to `data-architect` and release operations to `release-manager`.

## Deliverables

Prerequisite checklist, environment contract, observability contract, deployment runbook input.

## Limits

Do not bypass store adapters. Do not define domain models from infrastructure shape.

