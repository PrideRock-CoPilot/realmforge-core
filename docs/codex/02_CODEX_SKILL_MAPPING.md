---
doc_id: DOC-CODEX-002
title: Codex Skill Mapping
status: draft
owner: realmforge-skill-creator
reviewers: [orchestrator, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: [FILE-DOCS-CODEX-SKILL-MAPPING]
visual_node_ids: [VN-CODEX-SKILL-MAPPING]
visual_edge_ids: [VE-CODEX-MAPS-CLAUDE-SKILLS]
approval_state: pending
---

# Codex Skill Mapping

## Purpose

This file maps Claude company skills to Codex skill sources.

| Claude skill | Codex skill source | Responsibility |
| --- | --- | --- |
| `skill-creator` | `realmforge-skill-creator` | RealmForge company skill governance |
| `ceo` | `ceo` | Strategy and investment approval |
| `pm` | `pm` | Planning, scope, handoffs |
| `cto` | `cto` | Architecture and technical risk |
| `domain-architect` | `domain-architect` | Domain language and bounded contexts |
| `security-architect` | `security-architect` | Threat model and hard boundaries |
| `api-architect` | `api-architect` | REST, MCP, CLI contracts |
| `infra-architect` | `infra-architect` | Postgres, deployment, observability |
| `data-architect` | `data-architect` | Parquet and data governance |
| `backend` | `backend` | Rust backend implementation |
| `frontend` | `frontend` | Human-facing interface |
| `data-engineer` | `data-engineer` | Parquet pipelines and datasets |
| `qa` | `qa` | Verification and release certification evidence |
| `accountant` | `accountant` | Cost model and variance review |
| `release-manager` | `release-manager` | Release, bundle, rollback readiness |
| `tech-writer` | `tech-writer` | ADRs and canonical docs |
| `biz-user` | `biz-user` | User stories and acceptance criteria |
| `orchestrator` | `orchestrator` | Handoff discipline |
| `council` | `council` | Cross-domain decision process |

## Trigger Rule

When a task touches a mapped domain, Codex should load the matching skill before acting. When a task touches three or more domains, load `orchestrator` first, then the specific skill docs.
