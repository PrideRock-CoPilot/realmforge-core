-- RealmForge Intake System Foundation Schema
-- Implements structured intake with AI learning and admin review workflow.
-- References: INTAKE_SYSTEM_DESIGN.md, ADR-001-intake-system-architecture.md

-- Enable pg_trgm extension for similarity search (ADR-001 Decision 3)
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================================================
-- CORE TABLES
-- ============================================================================

-- Application type definitions (canonical + AI-generated drafts)
CREATE TABLE IF NOT EXISTS application_types (
    id TEXT PRIMARY KEY,                            -- '03_dynamic_web_app'
    name TEXT NOT NULL,                             -- 'Dynamic Web Application'
    description TEXT NOT NULL,
    complexity TEXT NOT NULL,                       -- 'medium', 'high', etc.
    timeline_weeks TEXT NOT NULL,                   -- '4-8 weeks'
    status TEXT NOT NULL DEFAULT 'active'           -- 'active', 'draft', 'deprecated'
        CHECK (status IN ('active', 'draft', 'deprecated')),
    is_ai_generated BOOLEAN NOT NULL DEFAULT FALSE, -- TRUE if AI created this
    created_by_actor_id TEXT,                       -- NULL for seed data
    approved_by_actor_id TEXT,                      -- NULL if still draft
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INT NOT NULL DEFAULT 1
);

-- Decision tree templates (versioned, JSONB decision trees)
CREATE TABLE IF NOT EXISTS intake_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_type_id TEXT NOT NULL REFERENCES application_types(id) ON DELETE CASCADE,
    version INT NOT NULL DEFAULT 1,
    decision_tree JSONB NOT NULL,                   -- Full decision tree structure
    feature_mappings JSONB NOT NULL,                -- Feature → module mappings
    status TEXT NOT NULL DEFAULT 'active'           -- 'active', 'draft', 'deprecated'
        CHECK (status IN ('active', 'draft', 'deprecated')),
    is_ai_generated BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_actor_id TEXT,
    approved_by_actor_id TEXT,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(application_type_id, version)
);

-- Active intake sessions (user flows, responses, AI interactions)
CREATE TABLE IF NOT EXISTS intake_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID REFERENCES intake_templates(id) ON DELETE SET NULL,
    application_type_id TEXT REFERENCES application_types(id) ON DELETE SET NULL,
    user_actor_id TEXT NOT NULL,                    -- No FK to allow demo/anonymous sessions
    session_type TEXT NOT NULL DEFAULT 'standard'   -- 'standard', 'ai_assisted', 'ai_generated'
        CHECK (session_type IN ('standard', 'ai_assisted', 'ai_generated')),
    responses JSONB NOT NULL DEFAULT '{}',          -- {question_id: answer}
    ai_questions JSONB DEFAULT '[]',                -- [{question, answer, timestamp}]
    derived_modules JSONB,                          -- [module1, module2, ...]
    derived_personas JSONB,                         -- [persona1, persona2, ...]
    status TEXT NOT NULL DEFAULT 'in_progress'      -- 'in_progress', 'completed', 'abandoned'
        CHECK (status IN ('in_progress', 'completed', 'abandoned')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    project_id TEXT                                 -- FK to projects table (after creation)
);

-- AI-asked questions (smart learning: questions not on canonical forms)
CREATE TABLE IF NOT EXISTS ai_intake_questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES intake_sessions(id) ON DELETE CASCADE,
    question_text TEXT NOT NULL,
    question_context JSONB NOT NULL,                -- {previous_answers, stuck_question_id}
    answer_text TEXT,
    application_type_id TEXT REFERENCES application_types(id) ON DELETE SET NULL,
    asked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    promoted_to_template BOOLEAN NOT NULL DEFAULT FALSE,
    promotion_date TIMESTAMPTZ
);

-- Question frequency tracking (auto-promotion threshold tracking)
CREATE TABLE IF NOT EXISTS question_frequency (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_text_normalized TEXT NOT NULL,         -- Normalized for similarity matching
    application_type_id TEXT REFERENCES application_types(id) ON DELETE CASCADE,
    occurrence_count INT NOT NULL DEFAULT 1,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    status TEXT NOT NULL DEFAULT 'tracked'          -- 'tracked', 'under_review', 'promoted', 'rejected'
        CHECK (status IN ('tracked', 'under_review', 'promoted', 'rejected')),
    UNIQUE(question_text_normalized, application_type_id)
);

-- Admin review queue (approval workflow for AI-generated content)
CREATE TABLE IF NOT EXISTS admin_review_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_type TEXT NOT NULL                       -- 'new_question', 'new_form', 'form_edit'
        CHECK (review_type IN ('new_question', 'new_form', 'form_edit')),
    application_type_id TEXT REFERENCES application_types(id) ON DELETE CASCADE,
    template_id UUID REFERENCES intake_templates(id) ON DELETE CASCADE,
    proposed_change JSONB NOT NULL,                 -- Full proposed change
    justification TEXT,                             -- Why this change is needed
    source_type TEXT NOT NULL                       -- 'ai_learning', 'user_request', 'admin_initiated'
        CHECK (source_type IN ('ai_learning', 'user_request', 'admin_initiated')),
    status TEXT NOT NULL DEFAULT 'pending'          -- 'pending', 'approved', 'rejected'
        CHECK (status IN ('pending', 'approved', 'rejected')),
    created_by_actor_id TEXT,
    reviewed_by_actor_id TEXT,
    reviewed_at TIMESTAMPTZ,
    review_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Audit trail (compliance, governance, debugging)
CREATE TABLE IF NOT EXISTS intake_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL                       -- 'template', 'question', 'application_type'
        CHECK (entity_type IN ('template', 'question', 'application_type')),
    entity_id UUID NOT NULL,
    action TEXT NOT NULL                            -- 'created', 'approved', 'rejected', 'edited'
        CHECK (action IN ('created', 'approved', 'rejected', 'edited', 'deprecated')),
    actor_id TEXT NOT NULL,
    changes JSONB,                                  -- Full diff
    justification TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================================
-- PERFORMANCE INDEXES
-- ============================================================================

-- Session lookups
CREATE INDEX IF NOT EXISTS idx_intake_sessions_user 
    ON intake_sessions(user_actor_id);
CREATE INDEX IF NOT EXISTS idx_intake_sessions_status 
    ON intake_sessions(status);
CREATE INDEX IF NOT EXISTS idx_intake_sessions_created 
    ON intake_sessions(created_at DESC);

-- AI question tracking
CREATE INDEX IF NOT EXISTS idx_ai_questions_session 
    ON ai_intake_questions(session_id);
CREATE INDEX IF NOT EXISTS idx_ai_questions_promoted 
    ON ai_intake_questions(promoted_to_template);

-- Frequency-based promotion
CREATE INDEX IF NOT EXISTS idx_question_frequency_count 
    ON question_frequency(occurrence_count DESC);
CREATE INDEX IF NOT EXISTS idx_question_frequency_status 
    ON question_frequency(status);

-- Admin review queue
CREATE INDEX IF NOT EXISTS idx_admin_queue_status 
    ON admin_review_queue(status);
CREATE INDEX IF NOT EXISTS idx_admin_queue_type 
    ON admin_review_queue(review_type);
CREATE INDEX IF NOT EXISTS idx_admin_queue_created 
    ON admin_review_queue(created_at DESC);

-- Audit log
CREATE INDEX IF NOT EXISTS idx_audit_entity 
    ON intake_audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp 
    ON intake_audit_log(timestamp DESC);

-- ============================================================================
-- FULL-TEXT SEARCH (Similarity Matching)
-- ============================================================================

-- GIN index for similarity search on normalized questions (pg_trgm)
CREATE INDEX IF NOT EXISTS idx_question_text_trgm 
    ON question_frequency 
    USING gin(question_text_normalized gin_trgm_ops);

-- Full-text search index for natural language queries
CREATE INDEX IF NOT EXISTS idx_question_text_fts 
    ON question_frequency 
    USING gin(to_tsvector('english', question_text_normalized));
