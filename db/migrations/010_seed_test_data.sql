-- RealmForge Seed Data for UAT Testing
-- Applies after all migrations are run.
-- 
-- Seed tenants, actors, projects, roles, and login credentials
-- that the automated tests and manual UAT can use.

-- ── System tenant (already exists in 001 implicitly, ensure it's there) ──
INSERT INTO tenants (id, name, status)
VALUES ('system', 'System Tenant', 'active')
ON CONFLICT (id) DO NOTHING;

INSERT INTO projects (id, tenant_id, name, status)
VALUES ('project-uat-001', 'system', 'UAT Test Project', 'active')
ON CONFLICT (id) DO NOTHING;

INSERT INTO actors (id, tenant_id, display_name, actor_type, status)
VALUES
    ('actor-GARY-001', 'system', 'Gary Test User', 'human', 'active'),
    ('actor-ALICE-001', 'system', 'Alice Operator', 'human', 'active'),
    ('agent-BUILD-001', 'system', 'Build Agent', 'agent', 'active')
ON CONFLICT (id) DO NOTHING;

-- ── Seeded Credentials ──
-- SHA-256("test-password-123") = "7c13ab2e1d157284ccb11e9737f90cbb611b0acdb8f8dcd4e303a680e58a8b1b"
-- SHA-256("alice-secret") = "0c848abb03307b06cf70cd4e29c157dc81af5e94ab3eb1d0c59a120269572376"
INSERT INTO stored_credentials (actor_id, tenant_id, credential_hash)
VALUES
    ('actor-GARY-001', 'system', '7c13ab2e1d157284ccb11e9737f90cbb611b0acdb8f8dcd4e303a680e58a8b1b'),
    ('actor-ALICE-001', 'system', '0c848abb03307b06cf70cd4e29c157dc81af5e94ab3eb1d0c59a120269572376')
ON CONFLICT (actor_id, tenant_id) DO NOTHING;

-- ── Roles ──
INSERT INTO roles (id, tenant_id, name)
VALUES
    ('role-admin-001', 'system', 'admin'),
    ('role-operator-001', 'system', 'operator'),
    ('role-reader-001', 'system', 'reader')
ON CONFLICT (id) DO NOTHING;

-- ── Actor-Role Bindings ──
INSERT INTO actor_roles (actor_id, role_id, project_id)
VALUES
    ('actor-GARY-001', 'role-admin-001', 'project-uat-001'),
    ('actor-ALICE-001', 'role-operator-001', 'project-uat-001'),
    ('agent-BUILD-001', 'role-reader-001', 'project-uat-001')
ON CONFLICT (actor_id, role_id, project_id) DO NOTHING;

-- ── Default Login Policy (already in 009 - just ensure it exists) ──
INSERT INTO login_policies (tenant_id, config_json)
VALUES ('system', '{"max_failed_attempts": 5, "window_seconds": 60, "block_duration_seconds": 300, "max_requests_per_window": 10, "credential_validation_enabled": true}')
ON CONFLICT (tenant_id) DO UPDATE
SET config_json = EXCLUDED.config_json,
    updated_at = NOW();
