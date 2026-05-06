---
doc_id: DOC-SPEC-004
title: Metadata Standard
status: draft
owner: tech-writer
reviewers: [cto, domain-architect, data-architect, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: metadata
work_path_ids: [WP-DOCS-000]
related_decision_ids: [DEC-COUNCIL-002]
related_file_ids: [FILE-ROOT-CARGO]
visual_node_ids: [VN-METADATA-STANDARD]
visual_edge_ids: []
approval_state: pending
---

# Metadata Standard

## Required Spec Document Front Matter

All spec docs must start with:

```yaml
---
doc_id:
title:
status: draft
owner:
reviewers:
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area:
work_path_ids: []
related_decision_ids: []
related_file_ids: []
visual_node_ids: []
visual_edge_ids: []
approval_state: pending
---
```

## Required Future File Registry Entry

Every implementation file must be registered before creation:

```yaml
file_id:
path:
artifact_class:
bounded_context:
owning_module:
risk_level:
created_by_work_path:
allowed_skill_grants: []
denied_skill_grants: []
public_interfaces: []
required_tests: []
required_trace_points: []
rollback_scope:
visual_node_ids: []
```

## Artifact Classes

`compiled_source`, `generated_source`, `managed_source`, `runtime_definition`, `runtime_policy`, `runtime_contract`, `runtime_trace_profile`, `static_asset`, `documentation`, `test_only`, `migration`, `parquet_dataset`, `operator_config`, `excluded`.

## Planned File Registry

These paths define the first implementation universe. File IDs are stable.

| File ID | Path | Artifact class | Module | Risk | Work path | Allowed grants | Required tests |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `FILE-ROOT-CARGO` | `Cargo.toml` | `operator_config` | workspace | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-WORKSPACE-METADATA-001` |
| `FILE-CRATE-DOMAIN-LIB` | `crates/authority-domain/src/lib.rs` | `compiled_source` | Authority Core | high | `WP-CORE-001` | `SGL-BACKEND-DOMAIN` | `TEST-DOMAIN-001` |
| `FILE-CRATE-DOMAIN-IDS` | `crates/authority-domain/src/ids.rs` | `compiled_source` | Authority Core | high | `WP-CORE-001` | `SGL-BACKEND-DOMAIN` | `TEST-DOMAIN-IDS-001` |
| `FILE-CRATE-DOMAIN-SCOPE` | `crates/authority-domain/src/scope.rs` | `compiled_source` | Skill Grants | high | `WP-SKILL-001` | `SGL-BACKEND-DOMAIN` | `TEST-SCOPE-001` |
| `FILE-CRATE-DOMAIN-WORKPATH` | `crates/authority-domain/src/work_path.rs` | `compiled_source` | Work Paths | high | `WP-WORKPATH-001` | `SGL-BACKEND-DOMAIN` | `TEST-WORKPATH-DOMAIN-001` |
| `FILE-CRATE-DOMAIN-CATALOG` | `crates/authority-domain/src/catalog.rs` | `compiled_source` | Catalogs | high | `WP-CATALOG-001` | `SGL-BACKEND-DOMAIN` | `TEST-CATALOG-DOMAIN-001` |
| `FILE-CRATE-DOMAIN-GRANT` | `crates/authority-domain/src/skill_grant.rs` | `compiled_source` | Skill Grants | critical | `WP-SKILL-001` | `SGL-BACKEND-DOMAIN` | `TEST-SKILL-GRANT-001` |
| `FILE-CRATE-DOMAIN-WORKFLOW` | `crates/authority-domain/src/workflow.rs` | `compiled_source` | Authority Core | high | `WP-CORE-001` | `SGL-BACKEND-DOMAIN` | `TEST-DOMAIN-WORKFLOW-001` |
| `FILE-CRATE-POLICY-LIB` | `crates/policy-engine/src/lib.rs` | `compiled_source` | Authority Core | critical | `WP-CORE-001` | `SGL-BACKEND-POLICY` | `TEST-POLICY-001` |
| `FILE-CRATE-POLICY-SOD` | `crates/policy-engine/src/separation_of_duties.rs` | `compiled_source` | Security | critical | `WP-SKILL-001` | `SGL-BACKEND-POLICY` | `TEST-SOD-001` |
| `FILE-CRATE-AUDIT-LIB` | `crates/audit-log/src/lib.rs` | `compiled_source` | Audit Ledger | high | `WP-CORE-001` | `SGL-BACKEND-AUDIT` | `TEST-AUDIT-CHAIN-001` |
| `FILE-CRATE-SNAPSHOT-LIB` | `crates/snapshot-ledger/src/lib.rs` | `compiled_source` | Snapshot Ledger | critical | `WP-SNAPSHOT-001` | `SGL-BACKEND-SNAPSHOT` | `TEST-SNAPSHOT-001` |
| `FILE-CRATE-STORE-LIB` | `crates/control-store/src/lib.rs` | `compiled_source` | Data Contracts | critical | `WP-DATA-001` | `SGL-BACKEND-STORE` | `TEST-STORE-001` |
| `FILE-CRATE-STORE-WORKPATH` | `crates/control-store/src/work_path.rs` | `compiled_source` | Work Paths | high | `WP-WORKPATH-001` | `SGL-BACKEND-STORE` | `TEST-WORKPATH-001` |
| `FILE-CRATE-SERVICE-LIB` | `crates/control-service/src/lib.rs` | `compiled_source` | Authority Core | critical | `WP-CORE-001` | `SGL-BACKEND-SERVICE` | `TEST-SERVICE-001` |
| `FILE-CRATE-GATEWAY-LIB` | `crates/agent-gateway/src/lib.rs` | `compiled_source` | Agent Gateway | critical | `WP-GATEWAY-001` | `SGL-BACKEND-GATEWAY` | `TEST-GATEWAY-001` |
| `FILE-CRATE-API-LIB` | `crates/control-api/src/lib.rs` | `compiled_source` | Interfaces | high | `WP-API-001` | `SGL-BACKEND-API` | `TEST-API-001` |
| `FILE-CRATE-MCP-LIB` | `crates/agent-mcp/src/lib.rs` | `compiled_source` | Interfaces | high | `WP-MCP-001` | `SGL-BACKEND-MCP` | `TEST-MCP-001` |
| `FILE-CRATE-CLI-MAIN` | `crates/operator-cli/src/main.rs` | `compiled_source` | Interfaces | medium | `WP-CLI-001` | `SGL-BACKEND-CLI` | `TEST-CLI-001` |
| `FILE-CRATE-PARQUET-LIB` | `crates/parquet-store/src/lib.rs` | `compiled_source` | Knowledge | high | `WP-KNOWLEDGE-001` | `SGL-DATA-PARQUET` | `TEST-PARQUET-001` |
| `FILE-CRATE-BUNDLE-LIB` | `crates/runtime-bundle/src/lib.rs` | `compiled_source` | Runtime Bundle | critical | `WP-RUNTIME-BUNDLE-001` | `SGL-BACKEND-RUNTIME` | `TEST-BUNDLE-001` |
| `FILE-CRATE-LIVE-RUNTIME-LIB` | `crates/live-runtime/src/lib.rs` | `compiled_source` | Live Runtime | critical | `WP-LIVE-RUNTIME-001` | `SGL-BACKEND-RUNTIME` | `TEST-LIVE-RUNTIME-001` |
| `FILE-CRATE-BUILD-WATCH-LIB` | `crates/build-watch/src/lib.rs` | `compiled_source` | Build Watch | high | `WP-BUILD-WATCH-001` | `SGL-BACKEND-WATCH` | `TEST-BUILD-WATCH-001` |
| `FILE-CRATE-LIVE-WATCH-LIB` | `crates/live-watch/src/lib.rs` | `compiled_source` | Live Watch | high | `WP-LIVE-WATCH-001` | `SGL-BACKEND-WATCH` | `TEST-LIVE-WATCH-001` |
| `FILE-DB-001` | `db/migrations/001_organization_tenant_app.sql` | `migration` | Data Contracts | critical | `WP-DATA-001` | `SGL-DATA-POSTGRES` | `TEST-MIGRATION-001` |
| `FILE-DB-002` | `db/migrations/002_catalogs_work_paths.sql` | `migration` | Catalogs | critical | `WP-CATALOG-001` | `SGL-DATA-POSTGRES` | `TEST-MIGRATION-002` |
| `FILE-DB-003` | `db/migrations/003_skill_grants_gateway.sql` | `migration` | Skill Grants | critical | `WP-SKILL-001` | `SGL-DATA-POSTGRES` | `TEST-MIGRATION-003` |
| `FILE-DB-004` | `db/migrations/004_evidence_watch_runtime.sql` | `migration` | Watch | critical | `WP-BUILD-WATCH-001` | `SGL-DATA-POSTGRES` | `TEST-MIGRATION-004` |
| `FILE-PARQUET-FILES` | `.realmforge/snapshots/{snapshot_id}/files.parquet` | `parquet_dataset` | Snapshot Ledger | high | `WP-SNAPSHOT-001` | `SGL-DATA-PARQUET` | `TEST-PARQUET-FILES-001` |
| `FILE-PARQUET-WORKPATHS` | `.realmforge/snapshots/{snapshot_id}/work_paths.parquet` | `parquet_dataset` | Work Paths | high | `WP-WORKPATH-001` | `SGL-DATA-PARQUET` | `TEST-PARQUET-WORKPATHS-001` |
| `FILE-PARQUET-EVIDENCE` | `.realmforge/snapshots/{snapshot_id}/evidence.parquet` | `parquet_dataset` | Evidence Ledger | high | `WP-BUILD-WATCH-001` | `SGL-DATA-PARQUET` | `TEST-PARQUET-EVIDENCE-001` |
| `FILE-CATALOG-LOGIN-MODULE` | `catalog/Login/catalog.json` | `runtime_definition` | Login Module | high | `WP-LOGIN-001` | `SGL-DATA-PARQUET` | `TEST-LOGIN-MODULE-001` |
| `FILE-CONTRACT-LOGIN` | `catalog/Login/contracts/login_request.json` | `runtime_contract` | Login Module | critical | `WP-LOGIN-001` | `SGL-BACKEND-RUNTIME` | `TEST-LOGIN-CONTRACT-001` |
| `FILE-CONTRACT-LOGIN-RESPONSE` | `catalog/Login/contracts/login_response.json` | `runtime_contract` | Login Module | critical | `WP-LOGIN-001` | `SGL-BACKEND-RUNTIME` | `TEST-LOGIN-CONTRACT-002` |
| `FILE-POLICY-LOGIN` | `catalog/Login/policy/login_policy.json` | `runtime_policy` | Login Module | critical | `WP-LOGIN-001` | `SGL-BACKEND-POLICY` | `TEST-LOGIN-POLICY-001` |
| `FILE-HANDLER-LOGIN` | `crates/control-service/src/login_handler.rs` | `compiled_source` | Login Module | critical | `WP-LOGIN-001` | `SGL-BACKEND-SERVICE` | `TEST-LOGIN-HANDLER-001` |
| `FILE-WATCH-LOGIN` | `catalog/Login/watch_profile.json` | `runtime_trace_profile` | Login Module | high | `WP-LOGIN-001` | `SGL-BACKEND-WATCH` | `TEST-LOGIN-WATCH-001` |
| `FILE-TEST-CATALOG-INTEGRATION` | `crates/control-service/tests/catalog_integration.rs` | `test_only` | Catalogs | medium | `WP-CATALOG-001` | `SGL-BACKEND-SERVICE` | `TEST-CATALOG-001` |
| `FILE-TEST-WORKPATH-INTEGRATION` | `crates/control-service/tests/work_path_integration.rs` | `test_only` | Work Paths | medium | `WP-WORKPATH-001` | `SGL-BACKEND-SERVICE` | `TEST-WORKPATH-001` |
| `FILE-DOCS-CODEX-SETUP` | `docs/codex/00_CODEX_SETUP.md` | `documentation` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-DOCS-CODEX-SESSION` | `docs/codex/01_CODEX_SESSION_PROTOCOL.md` | `documentation` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-DOCS-CODEX-MAPPING` | `docs/codex/02_CODEX_SKILL_MAPPING.md` | `documentation` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-DOCS-CODEX-SKILL-INSTALL` | `docs/codex/03_CODEX_SKILL_INSTALL_PLAN.md` | `documentation` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-DOCS-CLINE-NEXT-PHASE-HANDOFF` | `docs/codex/04_CLINE_NEXT_PHASE_PLANNING_HANDOFF.md` | `documentation` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-DOCS-CODE-REVIEW-PROCESS` | `docs/codex/05_CODE_REVIEW_PROCESS.md` | `documentation` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-DOCS-CODE-REVIEW-LEDGER` | `docs/codex/06_CODE_REVIEW_LEDGER.md` | `documentation` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-SKILL-PEER-REVIEW` | `skills/peer-review/SKILL.md` | `operator_config` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-CLAUDE-SKILL-PEER-REVIEW` | `.claude/skills/peer-review/SKILL.md` | `operator_config` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-SKILL-CODE-REVIEW` | `skills/code-review/SKILL.md` | `operator_config` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-CLAUDE-SKILL-CODE-REVIEW` | `.claude/skills/code-review/SKILL.md` | `operator_config` | Codex Operations | low | `WP-DOCS-000` | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `FILE-CRATE-INTAKE-LIB` | `crates/intake-engine/src/lib.rs` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-LIB-001` |
| `FILE-CRATE-INTAKE-TREE` | `crates/intake-engine/src/tree.rs` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-TREE-001` |
| `FILE-CRATE-INTAKE-CONDITION` | `crates/intake-engine/src/condition.rs` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-CONDITION-001` |
| `FILE-CRATE-INTAKE-ENGINE` | `crates/intake-engine/src/engine.rs` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-ENGINE-001` |
| `FILE-CRATE-INTAKE-MAPPER` | `crates/intake-engine/src/mapper.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-MAPPER-001` |
| `FILE-CRATE-INTAKE-VALIDATE` | `crates/intake-engine/src/validate.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-VALIDATE-001` |
| `FILE-CRATE-INTAKE-ERROR` | `crates/intake-engine/src/error.rs` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-ERROR-001` |
| `FILE-CRATE-STORE-INTAKE` | `crates/control-store/src/intake.rs` | `compiled_source` | Data Contracts | critical | `WP-CORE-001` | `SGL-BACKEND-STORE` | `TEST-STORE-INTAKE-001` |
| `FILE-DB-013` | `db/migrations/013_intake_decision_trees.sql` | `migration` | Data Contracts | critical | `WP-CORE-001` | `SGL-DATA-POSTGRES` | `TEST-MIGRATION-013` |
| `FILE-TREE-STATIC-SITE` | `catalog/intake-trees/tree-static-site.json` | `runtime_definition` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-TREE-001` |
| `FILE-TREE-WEB-APP` | `catalog/intake-trees/tree-web-app.json` | `runtime_definition` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-TREE-001` |
| `FILE-TREE-API-SERVICE` | `catalog/intake-trees/tree-api-service.json` | `runtime_definition` | Intake | medium | `WP-CORE-001` | `SGL-BACKEND-CORE` | `TEST-INTAKE-TREE-001` |
| `FILE-FE-INT-WIZARD-TYPES` | `frontend/src/features/intake-wizard/types.ts` | `compiled_source` | Intake | low | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-WIZARD-001` |
| `FILE-FE-INT-WIZARD-ENGINE` | `frontend/src/features/intake-wizard/tree-engine.ts` | `compiled_source` | Intake | medium | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-WIZARD-002` |
| `FILE-FE-INT-WIZARD-SELECTOR` | `frontend/src/features/intake-wizard/app-type-selector.tsx` | `compiled_source` | Intake | low | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-WIZARD-003` |
| `FILE-FE-INT-WIZARD-QUESTIONS` | `frontend/src/features/intake-wizard/question-walkthrough.tsx` | `compiled_source` | Intake | low | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-WIZARD-004` |
| `FILE-FE-INT-WIZARD-SUMMARY` | `frontend/src/features/intake-wizard/intake-summary.tsx` | `compiled_source` | Intake | low | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-WIZARD-005` |
| `FILE-FE-INT-WIZARD-MAIN` | `frontend/src/features/intake-wizard/intake-wizard.tsx` | `compiled_source` | Intake | high | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-WIZARD-006` |
| `FILE-FE-INT-WIZARD-INDEX` | `frontend/src/features/intake-wizard/index.ts` | `compiled_source` | Intake | low | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-WIZARD-007` |
| `FILE-FE-INT-WIZARD-CSS` | `frontend/src/index.css` | `managed_source` | Intake | medium | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-WIZARD-008` |
| `FILE-FE-INT-BOARD` | `frontend/src/features/boards/intake/intake-board.tsx` | `compiled_source` | Intake | low | `WP-CORE-001` | `SGL-FRONTEND-CORE` | `TEST-FE-INT-BOARD-001` |
| `FILE-CRATE-CODE-REVIEW-CARGO` | `crates/code-review/Cargo.toml` | `operator_config` | Code Review | medium | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-001` |
| `FILE-CRATE-CODE-REVIEW-LIB` | `crates/code-review/src/lib.rs` | `compiled_source` | Code Review | high | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-001` |
| `FILE-CRATE-CODE-REVIEW-CONFIG` | `crates/code-review/src/config.rs` | `compiled_source` | Code Review | high | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-008` |
| `FILE-CRATE-CODE-REVIEW-STANDARDS` | `crates/code-review/src/standards.rs` | `compiled_source` | Code Review | high | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-001`, `TEST-CODE-REVIEW-005` |
| `FILE-CRATE-CODE-REVIEW-CHECK` | `crates/code-review/src/check.rs` | `compiled_source` | Code Review | medium | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-001` |
| `FILE-CRATE-CODE-REVIEW-EXEC` | `crates/code-review/src/executable.rs` | `compiled_source` | Code Review | critical | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-002`, `TEST-CODE-REVIEW-009`, `TEST-CODE-REVIEW-010` |
| `FILE-CRATE-CODE-REVIEW-REGEX` | `crates/code-review/src/regex.rs` | `compiled_source` | Code Review | medium | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-003` |
| `FILE-CRATE-CODE-REVIEW-FILEPROP` | `crates/code-review/src/file_property.rs` | `compiled_source` | Code Review | medium | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-004` |
| `FILE-CRATE-CODE-REVIEW-REPORT` | `crates/code-review/src/report.rs` | `compiled_source` | Code Review | medium | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-006` |
| `FILE-CRATE-CODE-REVIEW-ERROR` | `crates/code-review/src/error.rs` | `compiled_source` | Code Review | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-001` |
| `FILE-CRATE-CODE-REVIEW-SEVERITY` | `crates/code-review/src/severity.rs` | `compiled_source` | Code Review | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-006` |
| `FILE-STANDARDS-AGNOSTIC` | `code-review-standards/language-agnostic.yaml` | `runtime_policy` | Code Review | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-001` |
| `FILE-STANDARDS-RUST` | `code-review-standards/rust.yaml` | `runtime_policy` | Code Review | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-001`, `TEST-CODE-REVIEW-005` |
| `FILE-CONFIG-CODE-REVIEW` | `.code-review.yaml` | `operator_config` | Code Review | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-CODE-REVIEW-007`, `TEST-CODE-REVIEW-008` |
| `FILE-CRATE-WORKFLOW-ENGINE-CARGO` | `crates/workflow-engine/Cargo.toml` | `operator_config` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-LIB` | `crates/workflow-engine/src/lib.rs` | `compiled_source` | Workflow Engine | medium | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-ERROR` | `crates/workflow-engine/src/error.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-TRAITS` | `crates/workflow-engine/src/traits.rs` | `compiled_source` | Workflow Engine | medium | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-ORCHESTRATOR` | `crates/workflow-engine/src/orchestrator.rs` | `compiled_source` | Workflow Engine | medium | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGES-MOD` | `crates/workflow-engine/src/stages/mod.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-IDEA` | `crates/workflow-engine/src/stages/idea.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-PLANNING` | `crates/workflow-engine/src/stages/planning.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-DESIGN` | `crates/workflow-engine/src/stages/design.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-ARCHITECTURE` | `crates/workflow-engine/src/stages/architecture.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-COUNCIL` | `crates/workflow-engine/src/stages/council.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-DEVELOPMENT` | `crates/workflow-engine/src/stages/development.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-PEER-REVIEW` | `crates/workflow-engine/src/stages/peer_review.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-CODE-REVIEW` | `crates/workflow-engine/src/stages/code_review.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-TESTING` | `crates/workflow-engine/src/stages/testing.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |
| `FILE-CRATE-WORKFLOW-ENGINE-STAGE-DOCUMENTATION` | `crates/workflow-engine/src/stages/documentation.rs` | `compiled_source` | Workflow Engine | low | `WP-DOCS-000` | `SGL-BACKEND-CORE` | `TEST-WORKFLOW-001` |

## Visual Metadata Contract

Every visual node ID must be stable, unique, and listed in at least one spec doc. Every visual edge ID must identify a source node, target node, and relationship type.
