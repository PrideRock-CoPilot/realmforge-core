-- RealmForge Phase 4: Agent Gateway and Skill Grants.
-- Creates skill_grants and work_packets tables.

CREATE TABLE IF NOT EXISTS skill_grants (
    id                TEXT PRIMARY KEY,
    actor_id          TEXT NOT NULL REFERENCES actors(id) ON DELETE RESTRICT,
    tenant_id         TEXT NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
    allowed_actions   JSONB NOT NULL DEFAULT '[]'::jsonb,
    denied_actions    JSONB NOT NULL DEFAULT '[]'::jsonb,
    allowed_file_patterns JSONB NOT NULL DEFAULT '[]'::jsonb,
    denied_file_patterns  JSONB NOT NULL DEFAULT '[]'::jsonb,
    budget_tokens     BIGINT,
    budget_operations BIGINT,
    separation_group  TEXT NOT NULL DEFAULT 'default',
    state             TEXT NOT NULL DEFAULT 'active'
                          CHECK (state IN ('active', 'expired', 'revoked', 'suspended')),
    expires_at        TIMESTAMPTZ NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_skill_grants_actor_id ON skill_grants(actor_id);
CREATE INDEX IF NOT EXISTS idx_skill_grants_tenant_id ON skill_grants(tenant_id);
CREATE INDEX IF NOT EXISTS idx_skill_grants_state ON skill_grants(state);

CREATE TABLE IF NOT EXISTS work_packets (
    id                  TEXT PRIMARY KEY,
    agent_id            TEXT NOT NULL REFERENCES actors(id) ON DELETE RESTRICT,
    work_path_node_id   TEXT NOT NULL,
    objective           TEXT NOT NULL,
    allowed_file_paths  JSONB NOT NULL DEFAULT '[]'::jsonb,
    denied_file_paths   JSONB NOT NULL DEFAULT '[]'::jsonb,
    required_contracts  JSONB NOT NULL DEFAULT '[]'::jsonb,
    required_tests      JSONB NOT NULL DEFAULT '[]'::jsonb,
    required_trace_points JSONB NOT NULL DEFAULT '[]'::jsonb,
    rollback_anchor     TEXT REFERENCES snapshot_manifests(id) ON DELETE SET NULL,
    cost_budget         JSONB,
    permission_scope    JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    status              TEXT NOT NULL DEFAULT 'pending'
                          CHECK (status IN ('pending', 'active', 'completed', 'failed', 'revoked'))
);

CREATE INDEX IF NOT EXISTS idx_work_packets_agent_id ON work_packets(agent_id);
CREATE INDEX IF NOT EXISTS idx_work_packets_status ON work_packets(status);
