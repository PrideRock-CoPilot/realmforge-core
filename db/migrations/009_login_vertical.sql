-- RealmForge Phase 9: Login Vertical — Authentication, rate limiting, and credentials.
-- Creates tables for login attempts, blocks, policies, and stored credentials.

-- ── Login Attempts ──

CREATE TABLE IF NOT EXISTS login_attempts (
    id          BIGSERIAL PRIMARY KEY,
    actor_id    TEXT NOT NULL,
    tenant_id   TEXT NOT NULL,
    timestamp   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    outcome     TEXT NOT NULL CHECK (outcome IN ('success', 'invalid_credentials', 'rate_limited', 'blocked'))
);

CREATE INDEX idx_login_attempts_actor_tenant ON login_attempts (actor_id, tenant_id);
CREATE INDEX idx_login_attempts_timestamp ON login_attempts (timestamp DESC);
CREATE INDEX idx_login_attempts_tenant_outcome ON login_attempts (tenant_id, outcome);

-- ── Login Blocks ──

CREATE TABLE IF NOT EXISTS login_blocks (
    actor_id        TEXT NOT NULL,
    tenant_id       TEXT NOT NULL,
    blocked_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blocked_until   TIMESTAMPTZ NOT NULL,
    reason          TEXT NOT NULL,
    PRIMARY KEY (actor_id, tenant_id)
);

CREATE INDEX idx_login_blocks_tenant ON login_blocks (tenant_id);
CREATE INDEX idx_login_blocks_active ON login_blocks (blocked_until);

-- ── Login Policies ──

CREATE TABLE IF NOT EXISTS login_policies (
    tenant_id   TEXT PRIMARY KEY,
    config_json JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Stored Credentials ──

CREATE TABLE IF NOT EXISTS stored_credentials (
    actor_id         TEXT NOT NULL,
    tenant_id        TEXT NOT NULL,
    credential_hash  TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (actor_id, tenant_id)
);

-- ── Seed: Default policy for the system tenant ──
INSERT INTO login_policies (tenant_id, config_json)
VALUES ('system', '{"max_failed_attempts": 5, "window_seconds": 60, "block_duration_seconds": 300, "max_requests_per_window": 10, "credential_validation_enabled": true}')
ON CONFLICT (tenant_id) DO NOTHING;
