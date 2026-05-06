-- ────────────────────────────────────────────────
-- 013 Intake Decision Trees — structured intake
-- ────────────────────────────────────────────────
-- Supports the intake-engine decision tree system:
--   decision trees → sessions → observations → app types
--
-- Covers Phase 1 (3 starter trees) and Phase 2
-- (5 additional trees) of the intake rollout.
-- ────────────────────────────────────────────────

-- ── Application Types ──
CREATE TABLE IF NOT EXISTS intake_app_types (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    complexity      TEXT NOT NULL DEFAULT 'low'
                    CHECK (complexity IN ('low', 'medium', 'high', 'platform')),
    icon            TEXT NOT NULL DEFAULT '',
    requires_auth   BOOLEAN NOT NULL DEFAULT false,
    requires_db     BOOLEAN NOT NULL DEFAULT false,
    requires_hosting BOOLEAN NOT NULL DEFAULT false,
    default_tree_id TEXT NOT NULL DEFAULT '',
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Decision Trees ──
CREATE TABLE IF NOT EXISTS intake_trees (
    id              TEXT PRIMARY KEY,
    tree_id         TEXT NOT NULL,
    name            TEXT NOT NULL,
    version         TEXT NOT NULL DEFAULT '1.0.0',
    description     TEXT NOT NULL DEFAULT '',
    applies_to      TEXT[] NOT NULL DEFAULT '{}',
    max_depth       INTEGER NOT NULL DEFAULT 5,
    definition      JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_intake_trees_tree_id ON intake_trees (tree_id);
CREATE INDEX IF NOT EXISTS idx_intake_trees_applies_to ON intake_trees USING GIN (applies_to);

-- ── Intake Sessions ──
CREATE TABLE IF NOT EXISTS intake_sessions (
    id              TEXT PRIMARY KEY,
    app_type_id     TEXT NOT NULL REFERENCES intake_app_types(id) ON DELETE RESTRICT,
    tree_id         TEXT NOT NULL REFERENCES intake_trees(id) ON DELETE RESTRICT,
    status          TEXT NOT NULL DEFAULT 'in_progress'
                    CHECK (status IN ('in_progress', 'completed', 'abandoned')),
    answers         JSONB NOT NULL DEFAULT '{}'::jsonb,
    current_question_id TEXT NOT NULL DEFAULT '',
    walk_result     JSONB,
    owner           TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_intake_sessions_status ON intake_sessions (status);
CREATE INDEX IF NOT EXISTS idx_intake_sessions_owner ON intake_sessions (owner);
CREATE INDEX IF NOT EXISTS idx_intake_sessions_app_type ON intake_sessions (app_type_id);

-- ── Observations (AI Learning) ──
CREATE TABLE IF NOT EXISTS intake_observations (
    id              TEXT PRIMARY KEY,
    question_text   TEXT NOT NULL,
    context         TEXT NOT NULL DEFAULT '',
    app_type_id     TEXT REFERENCES intake_app_types(id) ON DELETE SET NULL,
    frequency       INTEGER NOT NULL DEFAULT 1,
    promotion_state TEXT NOT NULL DEFAULT 'observation'
                    CHECK (promotion_state IN ('observation', 'candidate', 'proposed', 'canonical')),
    suggested_tree_id TEXT NOT NULL DEFAULT '',
    admin_notes     TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_intake_observations_promotion ON intake_observations (promotion_state);
CREATE INDEX IF NOT EXISTS idx_intake_observations_frequency ON intake_observations (frequency DESC);

-- ── Seed Application Types ──
INSERT INTO intake_app_types (id, name, description, complexity, icon, default_tree_id)
VALUES
    ('static-site', 'Static Site', 'A static website with HTML, CSS, and client-side JavaScript only', 'low', 'globe', 'tree-static-site'),
    ('web-app', 'Web Application', 'A dynamic web application with server-side logic and database', 'medium', 'monitor', 'tree-web-app'),
    ('api-service', 'API Service', 'A REST, GraphQL, or gRPC API backend service', 'medium', 'server', 'tree-api-service')
ON CONFLICT (id) DO NOTHING;
