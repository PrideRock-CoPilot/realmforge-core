-- RealmForge catalog and work path schema.
-- Depends on: 001_core_foundation.sql

-- ── Catalog entries ──

CREATE TABLE IF NOT EXISTS catalog_entries (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  scope TEXT NOT NULL CHECK (scope IN ('global', 'tenant', 'app')),
  scope_tenant_id TEXT REFERENCES tenants(id) ON DELETE RESTRICT,
  scope_app_id TEXT,
  parent_id TEXT REFERENCES catalog_entries(id) ON DELETE SET NULL,
  module_type TEXT NOT NULL CHECK (module_type IN ('module', 'contract', 'policy', 'handler', 'watch')),
  provenance_json JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_catalog_entries_scope ON catalog_entries(scope, scope_tenant_id);

-- ── Work path graphs ──

CREATE TABLE IF NOT EXISTS work_path_graphs (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Work path nodes ──

CREATE TABLE IF NOT EXISTS work_path_nodes (
  id TEXT PRIMARY KEY,
  work_path_id TEXT NOT NULL REFERENCES work_path_graphs(id) ON DELETE CASCADE,
  node_type TEXT NOT NULL CHECK (node_type IN ('module', 'runtime_contract', 'policy', 'service', 'watch_signal', 'data_contract', 'evidence')),
  name TEXT NOT NULL,
  file_ids JSONB NOT NULL DEFAULT '[]',
  contract_ids JSONB NOT NULL DEFAULT '[]',
  test_ids JSONB NOT NULL DEFAULT '[]',
  trace_point_ids JSONB NOT NULL DEFAULT '[]',
  children_ids JSONB NOT NULL DEFAULT '[]',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_work_path_nodes_graph ON work_path_nodes(work_path_id);
