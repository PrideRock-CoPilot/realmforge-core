# ─────────────────────────────────────────────
# Seed Test Data for E2E Tests
# ─────────────────────────────────────────────
# Seeds the database with test tenants, actors, credentials,
# board plans, and work paths needed by the frontend e2e tests.
# Requires PostgreSQL to be running on localhost:5432.

param(
  [string]$DatabaseUrl = "postgres://postgres:postgres@localhost:5432/realmforge"
)

$ErrorActionPreference = "Stop"

Write-Host "Seeding test data..." -ForegroundColor Cyan
Write-Host "Database: $DatabaseUrl" -ForegroundColor DarkGray

# Use the backend's existing seed logic via the PostgreSQL connection
# We seed the database directly with SQL since the login flow expects
# seeded credentials.

$Env:DATABASE_URL = $DatabaseUrl

# Step 1: Ensure tenant exists
Write-Host "Inserting tenant 'system'..." -ForegroundColor Gray
psql $DatabaseUrl -c @"
INSERT INTO tenants (id, name, status, created_at, updated_at)
VALUES ('system', 'System Tenant', 'active', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
"@ 2>&1 | Out-Null

# Step 2: Ensure project exists
Write-Host "Inserting project 'project-uat-001'..." -ForegroundColor Gray
psql $DatabaseUrl -c @"
INSERT INTO projects (id, tenant_id, name, status, created_at, updated_at)
VALUES ('project-uat-001', 'system', 'UAT Project', 'active', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
"@ 2>&1 | Out-Null

# Step 3: Ensure actor 'alice' exists
Write-Host "Inserting actor 'alice'..." -ForegroundColor Gray
psql $DatabaseUrl -c @"
INSERT INTO actors (id, tenant_id, display_name, actor_type, status, created_at, updated_at)
VALUES ('alice', 'system', 'Alice', 'human', 'active', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
"@ 2>&1 | Out-Null

# Step 4: Insert credential for alice (s3cr3t)
# The credential is stored as a SHA-256 hash. In production this uses
# login_policy::hash_credential(). For test purposes, we pre-compute.
Write-Host "Inserting credential for 'alice'..." -ForegroundColor Gray
$hash = (Write-Host "s3cr3t" -NoNewline | Get-FileHash -Algorithm SHA256).Hash
$hashBytes = [Text.Encoding]::UTF8.GetBytes($hash.ToLower())
$hashHex = [System.BitConverter]::ToString($hashBytes).Replace("-","").ToLower()

psql $DatabaseUrl -c @"
INSERT INTO actor_credentials (actor_id, tenant_id, credential_hash, created_at)
VALUES ('alice', 'system', decode('$hashHex', 'hex'), NOW())
ON CONFLICT (actor_id, tenant_id) DO UPDATE SET credential_hash = decode('$hashHex', 'hex');
"@ 2>&1 | Out-Null

# Step 5: Create sample board plans
Write-Host "Inserting sample board plans..." -ForegroundColor Gray
psql $DatabaseUrl -c @"
INSERT INTO board_plans (id, title, status, work_path_refs, created_at, updated_at)
VALUES
  ('plan-001', 'Login Module Implementation', 'draft', ARRAY['wp_01'], NOW(), NOW()),
  ('plan-002', 'Audit Dashboard Enhancement', 'in_review', ARRAY['wp_02'], NOW(), NOW()),
  ('plan-003', 'Work Path Cost Analysis', 'approved', ARRAY['wp_03'], NOW(), NOW()),
  ('plan-004', 'Live Watch Integration', 'in_progress', ARRAY['wp_04'], NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
"@ 2>&1 | Out-Null

# Step 6: Create sample work paths
Write-Host "Inserting sample work paths..." -ForegroundColor Gray
psql $DatabaseUrl -c @"
INSERT INTO work_paths (id, tenant_id, name, description, graph_json, created_at, updated_at)
VALUES
  ('wp_01', 'system', 'Login Module', 'End-to-end work path for Login module implementation', '{"nodes":[],"edges":[]}', NOW(), NOW()),
  ('wp_02', 'system', 'Audit Dashboard', 'Work path for audit dashboard enhancement', '{"nodes":[],"edges":[]}', NOW(), NOW()),
  ('wp_03', 'system', 'Cost Analysis', 'Work path for cost analysis feature', '{"nodes":[],"edges":[]}', NOW(), NOW()),
  ('wp_04', 'system', 'Live Watch', 'Work path for live watch integration', '{"nodes":[],"edges":[]}', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
"@ 2>&1 | Out-Null

# Step 7: Create sample bundles for release board
Write-Host "Inserting sample bundles..." -ForegroundColor Gray
psql $DatabaseUrl -c @"
INSERT INTO bundles (id, name, version, status, tenant_id, created_at)
VALUES
  ('bundle-001', 'Login Module', '1.0.0', 'draft', 'system', NOW()),
  ('bundle-002', 'Audit Dashboard', '2.1.0', 'approved', 'system', NOW())
ON CONFLICT (id) DO NOTHING;
"@ 2>&1 | Out-Null

# Step 8: Insert login policy for system tenant (relaxed for testing)
Write-Host "Inserting login policy for system tenant..." -ForegroundColor Gray
psql $DatabaseUrl -c @"
INSERT INTO login_policies (tenant_id, config, created_at, updated_at)
VALUES ('system', '{"max_failed_attempts": 10, "window_seconds": 300, "max_attempts": 20, "block_duration_seconds": 60}', NOW(), NOW())
ON CONFLICT (tenant_id) DO NOTHING;
"@ 2>&1 | Out-Null

Write-Host "✓ Seed complete!" -ForegroundColor Green
Write-Host "  Tenant: system" -ForegroundColor DarkGray
Write-Host "  Actor: alice / s3cr3t" -ForegroundColor DarkGray
Write-Host "  Plans: 4 board plans" -ForegroundColor DarkGray
Write-Host "  Work Paths: 4 work paths" -ForegroundColor DarkGray
Write-Host "  Bundles: 2 bundles" -ForegroundColor DarkGray
