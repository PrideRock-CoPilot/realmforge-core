-- RealmForge Core snapshot and rollback-preview schema.

CREATE TABLE IF NOT EXISTS snapshot_manifests (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  status TEXT NOT NULL CHECK (status IN ('draft', 'known_good', 'invalid', 'rollback_previewed')),
  reason TEXT NOT NULL,
  parent_snapshot_id TEXT REFERENCES snapshot_manifests(id) ON DELETE SET NULL,
  manifest_json JSONB NOT NULL,
  previous_manifest_hash TEXT,
  manifest_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS snapshot_object_refs (
  id BIGSERIAL PRIMARY KEY,
  snapshot_id TEXT NOT NULL REFERENCES snapshot_manifests(id) ON DELETE CASCADE,
  logical_path TEXT NOT NULL,
  object_uri TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  size_bytes BIGINT NOT NULL CHECK (size_bytes >= 0)
);

CREATE TABLE IF NOT EXISTS snapshot_table_exports (
  id BIGSERIAL PRIMARY KEY,
  snapshot_id TEXT NOT NULL REFERENCES snapshot_manifests(id) ON DELETE CASCADE,
  table_name TEXT NOT NULL,
  export_uri TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  row_count BIGINT NOT NULL CHECK (row_count >= 0)
);

CREATE TABLE IF NOT EXISTS rollback_previews (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  from_snapshot_id TEXT NOT NULL REFERENCES snapshot_manifests(id) ON DELETE RESTRICT,
  to_snapshot_id TEXT NOT NULL REFERENCES snapshot_manifests(id) ON DELETE RESTRICT,
  status TEXT NOT NULL CHECK (status IN ('draft', 'validated', 'blocked')),
  preview_json JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_snapshot_manifests_project_time
  ON snapshot_manifests(project_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_snapshot_object_refs_snapshot
  ON snapshot_object_refs(snapshot_id);
