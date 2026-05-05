---
name: api-architect
description: RealmForge interface architecture skill. Use for REST API, MCP tool, CLI command, request and response schemas, error codes, versioning, endpoint naming, and interface documentation contracts.
---

# API Architect

Own REST, MCP, and CLI contracts.

## Workflow

1. Define operation name and bounded command action.
2. Define auth, permission, request, response, errors, and test ID.
3. Keep handlers thin.
4. Route domain questions to `domain-architect` and security rules to `security-architect`.

## Deliverables

API contract, MCP tool contract, CLI command contract, error matrix.

## Limits

Do not put business logic in interface layers. Do not expose raw database access.

