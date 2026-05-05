-- ────────────────────────────────────────────────
-- 012 Intake Pipeline — plans and work_packets
-- ────────────────────────────────────────────────
-- Supports the 6-stage intake pipeline:
--   intake → refinement → architecture → decomposition
--   → packetization → ready
--
-- Each plan stores its structured pipeline state as JSONB
-- columns for flexible stage-by-stage population.
-- ────────────────────────────────────────────────

-- ── Pipeline Stage Enum ──
DO $$ BEGIN
    CREATE TYPE pipeline_stage AS ENUM (
        'intake', 'refinement', 'architecture',
        'decomposition', 'packetization', 'ready'
    );
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- ── Intake Plans Table ──
CREATE TABLE IF NOT EXISTS intake_plans (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    goal            TEXT NOT NULL,
    scope           TEXT NOT NULL,
    constraints     TEXT[] NOT NULL DEFAULT '{}',
    assumptions     TEXT[] NOT NULL DEFAULT '{}',
    architecture_summary TEXT NOT NULL DEFAULT '',
    core_areas      JSONB NOT NULL DEFAULT '[]'::jsonb,
    decisions       JSONB NOT NULL DEFAULT '[]'::jsonb,
    risks           JSONB NOT NULL DEFAULT '[]'::jsonb,
    phases          JSONB NOT NULL DEFAULT '[]'::jsonb,
    work_packets    JSONB NOT NULL DEFAULT '[]'::jsonb,
    status          TEXT NOT NULL DEFAULT 'draft',
    current_stage   pipeline_stage NOT NULL DEFAULT 'intake',
    next_action     TEXT NOT NULL DEFAULT '',
    owner           TEXT NOT NULL DEFAULT '',
    audit_log       JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for listing by stage
CREATE INDEX IF NOT EXISTS idx_intake_plans_stage ON intake_plans (current_stage);
-- Index for listing by owner
CREATE INDEX IF NOT EXISTS idx_intake_plans_owner ON intake_plans (owner);
-- Index for listing by status
CREATE INDEX IF NOT EXISTS idx_intake_plans_status ON intake_plans (status);
-- Index for timestamp ordering
CREATE INDEX IF NOT EXISTS idx_intake_plans_updated_at ON intake_plans (updated_at DESC);
