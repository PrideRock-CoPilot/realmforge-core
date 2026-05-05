-- RealmForge Phase 6: Boards and Build Watch.
-- Creates tables for board plans, approvals, release commands, watch events, violations, and cost records.

-- ── Board Plans ──

CREATE TABLE IF NOT EXISTS board_plans (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    work_path_refs  JSONB NOT NULL DEFAULT '[]'::jsonb,
    status          TEXT NOT NULL CHECK (status IN ('draft', 'in_review', 'approved', 'in_progress', 'completed', 'blocked', 'archived')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_board_plans_status ON board_plans(status);
CREATE INDEX IF NOT EXISTS idx_board_plans_created_at ON board_plans(created_at DESC);

-- ── Board Approvals ──

CREATE TABLE IF NOT EXISTS board_approvals (
    id          TEXT PRIMARY KEY,
    plan_id     TEXT NOT NULL REFERENCES board_plans(id),
    approver    TEXT NOT NULL,
    decision    TEXT NOT NULL CHECK (decision IN ('approved', 'rejected', 'needs_changes')),
    comment     TEXT,
    timestamp   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_board_approvals_plan_id ON board_approvals(plan_id);

-- ── Release Commands ──

CREATE TABLE IF NOT EXISTS release_commands (
    id              TEXT PRIMARY KEY,
    plan_id         TEXT NOT NULL REFERENCES board_plans(id),
    bundle_ref      TEXT NOT NULL,
    approval_ref    TEXT NOT NULL,
    status          TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'deployed', 'rolled_back')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_release_commands_plan_id ON release_commands(plan_id);
CREATE INDEX IF NOT EXISTS idx_release_commands_status ON release_commands(status);

-- ── Watch Events ──

CREATE TABLE IF NOT EXISTS watch_events (
    id              TEXT PRIMARY KEY,
    scope           TEXT NOT NULL DEFAULT '',
    event_type      TEXT NOT NULL CHECK (event_type IN ('file_mutation', 'packet_submission', 'policy_violation', 'cost_anomaly', 'build_failure', 'test_failure', 'evidence_gap')),
    severity        TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'violation', 'critical')),
    detail          TEXT NOT NULL DEFAULT '',
    evidence_ref    TEXT,
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_watch_events_scope ON watch_events(scope);
CREATE INDEX IF NOT EXISTS idx_watch_events_severity ON watch_events(severity);
CREATE INDEX IF NOT EXISTS idx_watch_events_timestamp ON watch_events(timestamp DESC);

-- ── Violations ──

CREATE TABLE IF NOT EXISTS violations (
    id              TEXT PRIMARY KEY,
    rule            TEXT NOT NULL,
    severity        TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'violation', 'critical')),
    evidence_ref    TEXT,
    detail          TEXT NOT NULL DEFAULT '',
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_violations_severity ON violations(severity);
CREATE INDEX IF NOT EXISTS idx_violations_recorded_at ON violations(recorded_at DESC);

-- ── Cost Records ──

CREATE TABLE IF NOT EXISTS cost_records (
    id              TEXT PRIMARY KEY,
    scope           TEXT NOT NULL DEFAULT '',
    token_cost      BIGINT NOT NULL DEFAULT 0,
    build_time_ms   BIGINT NOT NULL DEFAULT 0,
    storage_bytes   BIGINT NOT NULL DEFAULT 0,
    rework_count    INTEGER NOT NULL DEFAULT 0,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cost_records_scope ON cost_records(scope);
CREATE INDEX IF NOT EXISTS idx_cost_records_recorded_at ON cost_records(recorded_at DESC);
