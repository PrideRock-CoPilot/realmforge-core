-- RealmForge Phase 5: Knowledge and Parquet Snapshots.
-- Creates the knowledge_datasets table for Postgres metadata side of Track B.

CREATE TABLE IF NOT EXISTS knowledge_datasets (
    id              TEXT PRIMARY KEY,
    scope           TEXT NOT NULL CHECK (scope IN ('global', 'tenant', 'app', 'work_path')),
    source_type     TEXT NOT NULL CHECK (source_type IN ('catalog', 'file', 'decision', 'evidence', 'trace')),
    source_id       TEXT NOT NULL,
    content_hash    TEXT NOT NULL DEFAULT '',
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    citations       JSONB NOT NULL DEFAULT '[]'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_knowledge_scope ON knowledge_datasets(scope);
CREATE INDEX IF NOT EXISTS idx_knowledge_source_type ON knowledge_datasets(source_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_indexed_at ON knowledge_datasets(indexed_at DESC);
