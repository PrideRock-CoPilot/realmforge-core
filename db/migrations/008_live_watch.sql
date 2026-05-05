-- RealmForge Phase 8: Live Watch — Runtime health monitoring and remediation.
-- Creates tables for watch signals, remediation proposals, and watch profiles.

-- ── Watch Signals ──

CREATE TABLE IF NOT EXISTS watch_signals (
    id          TEXT PRIMARY KEY,
    app_id      TEXT NOT NULL,
    signal_type TEXT NOT NULL,
    value       DOUBLE PRECISION NOT NULL,
    threshold   DOUBLE PRECISION NOT NULL,
    severity    TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    timestamp   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_watch_signals_app_id ON watch_signals (app_id);
CREATE INDEX idx_watch_signals_signal_type ON watch_signals (signal_type);
CREATE INDEX idx_watch_signals_severity ON watch_signals (severity);
CREATE INDEX idx_watch_signals_timestamp ON watch_signals (timestamp DESC);

-- ── Remediation Proposals ──

CREATE TABLE IF NOT EXISTS remediation_proposals (
    id                    TEXT PRIMARY KEY,
    app_id                TEXT NOT NULL,
    triggering_signal_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    proposed_actions      JSONB NOT NULL DEFAULT '[]'::jsonb,
    impact_analysis       TEXT NOT NULL,
    status                TEXT NOT NULL DEFAULT 'proposed' CHECK (status IN ('proposed', 'approved', 'rejected', 'executed')),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_remediation_proposals_app_id ON remediation_proposals (app_id);
CREATE INDEX idx_remediation_proposals_status ON remediation_proposals (status);
CREATE INDEX idx_remediation_proposals_created_at ON remediation_proposals (created_at DESC);

-- ── Watch Profiles ──

CREATE TABLE IF NOT EXISTS watch_profiles (
    app_id                TEXT PRIMARY KEY,
    signal_thresholds     JSONB NOT NULL DEFAULT '[]'::jsonb,
    max_proposals_per_day BIGINT NOT NULL DEFAULT 10,
    poll_interval_secs    BIGINT NOT NULL DEFAULT 30
);
