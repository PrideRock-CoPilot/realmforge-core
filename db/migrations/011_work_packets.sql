-- RealmForge agent work packet schema.
-- Depends on: 001_core_foundation.sql, 002_snapshot_foundation.sql

CREATE TABLE IF NOT EXISTS work_packets (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL REFERENCES actors(id) ON DELETE RESTRICT,
  work_path_node_id TEXT NOT NULL,
  objective TEXT NOT NULL,
  allowed_file_paths JSONB NOT NULL DEFAULT '[]',
  denied_file_paths JSONB NOT NULL DEFAULT '[]',
  required_contracts JSONB NOT NULL DEFAULT '[]',
  required_tests JSONB NOT NULL DEFAULT '[]',
  required_trace_points JSONB NOT NULL DEFAULT '[]',
  rollback_anchor TEXT REFERENCES snapshot_manifests(id) ON DELETE SET NULL,
  cost_budget JSONB,
  permission_scope JSONB NOT NULL,
  status JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_work_packets_agent ON work_packets(agent_id);
CREATE INDEX IF NOT EXISTS idx_work_packets_created ON work_packets(created_at DESC);
