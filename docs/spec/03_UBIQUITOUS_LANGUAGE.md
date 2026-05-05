---
doc_id: DOC-SPEC-003
title: Ubiquitous Language
status: draft
owner: domain-architect
reviewers: [cto, pm, api-architect, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: domain
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-DOMAIN-LANGUAGE]
visual_edge_ids: []
approval_state: pending
---

# Ubiquitous Language

## Naming Rules

- Use `organization`, `tenant`, `app`, `catalog`, `module`, `work path`, `skill grant`, `agent`, `packet`, `evidence`, `snapshot`, `bundle`, and `watch signal` exactly as defined here.
- Do not use `user` for every actor. Use `human actor`, `agent actor`, or `service actor`.
- Do not use `skill` as permission. A skill is a capability template; a `skill grant` is the active permission.
- Do not use `task` when the artifact is a governed execution unit. Use `work packet`.
- Do not use `rollback` for Git-only reversal. Rollback restores governed state.

## Domain Terms

| Term | Definition | ID prefix |
| --- | --- | --- |
| Organization | Legal or operational umbrella that owns one or more tenants | `org_` |
| Tenant | Governance and sharing boundary inside an organization | `ten_` |
| App | A governed application built or operated by the system | `app_` |
| Catalog | Versioned registry of modules, files, contracts, tests, traces, grants, and evidence | `cat_` |
| Global Catalog | Read-only approved reusable inventory available for copy into tenants | `gcat_` |
| Tenant Catalog | Tenant-owned sharing catalog | `tcat_` |
| App Catalog | App-specific catalog and snapshot stream | `acat_` |
| Module | Reusable capability package with contracts, files, tests, traces, and policies | `mod_` |
| Work Path | Directed graph that turns intent into executable scoped build steps | `wp_` |
| Work Path Node | Capability, screen, API, file, test, decision, risk, trace, or gate in a work path | `wpn_` |
| Skill | Role template defining responsibility and possible capabilities | `ski_` |
| Skill Grant | Time-bound hard permission binding a skill to an agent, app, and scope | `sgr_` |
| Agent | AI or service actor that can act only through grants | `agt_` |
| Work Packet | Smallest governed execution unit assigned to an agent | `pkt_` |
| Bounded Command | Intent request that must be authorized before mutation | `cmd_` |
| Evidence Record | Proof emitted by agents, tests, watchers, runtime, or release steps | `evd_` |
| Snapshot | Immutable point-in-time governed state anchor | `snp_` |
| Runtime Bundle | Signed executable/runtime state package | `bun_` |
| Build Watch Event | Construction-time observation or violation | `bwe_` |
| Live Watch Signal | Runtime observation, anomaly, or remediation proposal seed | `lws_` |

## State Names

Use snake_case for persisted states and PascalCase for Rust enum variants.

| Concept | Persisted states |
| --- | --- |
| Planning session | `draft`, `mapped`, `reviewed`, `approved`, `in_execution`, `blocked`, `completed`, `released`, `archived` |
| Skill grant | `draft`, `issued`, `active`, `suspended`, `expired`, `revoked` |
| Work packet | `draft`, `ready`, `assigned`, `in_progress`, `evidence_submitted`, `verified`, `blocked`, `rejected`, `completed`, `revoked` |
| Bounded command | `proposed`, `authorized`, `denied`, `applied`, `failed` |
| Snapshot | `draft`, `known_good`, `invalid`, `rollback_previewed`, `rollback_restored` |
| Runtime bundle | `draft`, `signed`, `verified`, `active`, `superseded`, `revoked` |
| Watch signal | `observed`, `triaged`, `packet_proposed`, `dismissed`, `resolved` |
