-- Increase login policy rate limits for E2E testing
UPDATE login_policies 
SET config_json = '{"max_failed_attempts": 100, "window_seconds": 300, "max_requests_per_window": 200, "credential_validation_enabled": true, "block_duration_seconds": 60}' 
WHERE tenant_id = 'system';

-- Clear stale login attempts and blocks
DELETE FROM login_attempts WHERE actor_id = 'alice' AND tenant_id = 'system';
DELETE FROM login_blocks WHERE actor_id = 'alice' AND tenant_id = 'system';
