-- RealmForge Phase 7: Runtime Bundle and Live Runtime.
-- Creates tables for bundle manifests, runtime instances, and bundle artifact tracking.

-- ── Bundle Manifests ──

CREATE TABLE IF NOT EXISTS bundle_manifests (
    id                    TEXT PRIMARY KEY,
    version               TEXT NOT NULL,
    app_id                TEXT NOT NULL,
    artifact_hashes       JSONB NOT NULL DEFAULT '[]'::jsonb,
    governance_signature  TEXT NOT NULL,
    release_approval_ref  TEXT,
    built_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status                TEXT NOT NULL CHECK (status IN ('building', 'signed', 'verified', 'deployed', 'running', 'failed'))
);

CREATE INDEX idx_bundle_manifests_app_id ON bundle_manifests (app_id);
CREATE INDEX idx_bundle_manifests_status ON bundle_manifests (status);

-- ── Runtime Instances ──

CREATE TABLE IF NOT EXISTS runtime_instances (
    id              TEXT PRIMARY KEY,
    bundle_id       TEXT NOT NULL REFERENCES bundle_manifests(id),
    status          TEXT NOT NULL DEFAULT 'loading' CHECK (status IN ('loading', 'running', 'paused', 'stopped', 'failed')),
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    active_sessions BIGINT NOT NULL DEFAULT 0,
    action_count    BIGINT NOT NULL DEFAULT 0,
    error_count     BIGINT NOT NULL DEFAULT 0,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX idx_runtime_instances_bundle_id ON runtime_instances (bundle_id);
CREATE INDEX idx_runtime_instances_status ON runtime_instances (status);
