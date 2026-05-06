# Decision Document: Intake System Roadmap Questions

**ID:** DEC-PRODUCT-PENDING-intake-roadmap-questions  
**Date:** 2025-01-30  
**Status:** PENDING (Requires PM + CTO approval)  
**Decision Owner:** Alex Rivera (PM), Dr. Rena Okafor (CTO)  
**Contributors:** RealmForge Development Team  
**Related Documents:**
- `docs/INTAKE_SYSTEM_DESIGN.md` Section 11.3
- `docs/MASTER_BUILD_PLAN.md` Phase 0a-0f
- `docs/decisions/ADR-001-intake-system-architecture.md`

---

## Overview

The Structured Intake System design (INTAKE_SYSTEM_DESIGN.md) identified three roadmap questions that require strategic decisions before full implementation. This document analyzes each question and provides recommendations.

---

## Question 8: Migration Strategy for Existing Projects

### Problem Statement

**Question:** How should RealmForge handle projects created BEFORE the structured intake system?

**Context:**
- Legacy projects have no intake session data
- No structured requirements captured
- Unknown feature derivations (modules/personas manually assigned)
- Can't rerun workflow engines without intake context

**Scenarios:**
1. **Pre-existing projects (N=0 initially, grows over time):**
   - Projects created via agent-driven clarification (current unstructured Phase 0)
   - No intake_sessions record
   - Requirements buried in chat logs, not machine-readable

2. **Ongoing projects:**
   - User wants to add features → needs intake context
   - User wants to regenerate requirements → no template reference

3. **Audit/compliance:**
   - "Show me how this project's requirements were captured"
   - No audit trail for pre-migration projects

### Options

#### Option A: Retro-Fit Intake (Manual Admin Process)

**Approach:**
- Admin manually creates intake_sessions for existing projects
- Fills in responses based on REQUIREMENTS.md analysis
- Links to closest matching template

**Pros:**
* ✅ Complete data consistency — all projects have intake records
* ✅ Future workflows work uniformly (no legacy special cases)
* ✅ Audit trail complete for all projects

**Cons:**
* ❌ High admin overhead (1-2 hours per project × N projects)
* ❌ Subjective interpretation of old requirements
* ❌ Risk of incorrect template matching

**Mitigation:**
- AI-assisted retro-fitting: analyze REQUIREMENTS.md → suggest template + responses
- Batch processing: prioritize active projects, defer archived projects
- Mark as `is_retro_fitted=true` to distinguish from organic intakes

#### Option B: Grandfather Clause (Legacy Flag)

**Approach:**
- Add `is_legacy_project` boolean to projects table
- Legacy projects bypass intake validation
- Workflows check flag and use fallback logic for missing intake data

**Pros:**
* ✅ Zero migration effort
* ✅ New system works independently
* ✅ No risk of corrupting old project data

**Cons:**
* ❌ Permanent technical debt (legacy code paths forever)
* ❌ Two different requirement capture systems (confusion)
* ❌ Cannot regenerate requirements for legacy projects

**Mitigation:**
- Sunset clause: legacy flag expires after 12 months, forcing retro-fit
- Dashboard warning: "This project uses legacy requirements (upgrade recommended)"

#### Option C: Shadow Intake (AI-Generated Backfill)

**Approach:**
- AI reads REQUIREMENTS.md + project files
- Generates synthetic intake_session with inferred responses
- Marks as `is_ai_generated=true`

**Pros:**
* ✅ Automated (scales to N projects)
* ✅ Consistent data model (all projects have intake records)
* ✅ Enables future workflows without special cases

**Cons:**
* ❌ AI inference may be wrong (garbage in → garbage out)
* ❌ No human validation of AI-generated intake
* ❌ Risk of circular logic (AI guesses modules based on REQUIREMENTS.md that was AI-generated)

**Mitigation:**
- Admin review queue for AI-generated intakes
- User notification: "We inferred your requirements — please review"
- Allow users to correct/complete AI-generated intake

### Recommendation

**✅ Option C (Shadow Intake with Admin Review) + Option B (Grandfather for <6 months)**

**Phased approach:**
1. **Phase 1 (Weeks 1-12):** Launch intake system, flag existing projects as `is_legacy=true`
2. **Phase 2 (Weeks 13-24):** AI backfill intakes for active legacy projects, queue for review
3. **Phase 3 (Weeks 25+):** Sunset legacy flag — all projects must have intake_sessions

**Rationale:**
- Avoids blocking Phase 0a launch (no migration prerequisite)
- Scales to N projects (AI-driven, not manual)
- Human-in-loop (admin review queue catches AI errors)
- Time-bound (legacy flag expires, forcing convergence)

**Implementation:**
```sql
-- Add migration tracking to projects table
ALTER TABLE projects 
    ADD COLUMN is_legacy BOOLEAN DEFAULT FALSE,
    ADD COLUMN legacy_sunset_date TIMESTAMPTZ;

-- Backfill existing projects
UPDATE projects 
    SET is_legacy = TRUE, 
        legacy_sunset_date = now() + INTERVAL '6 months'
    WHERE intake_session_id IS NULL;
```

**Decision Required From:**
- [ ] **Alex Rivera (PM):** Approve phased rollout timeline
- [ ] **Rena Okafor (CTO):** Approve AI backfill architecture
- [ ] **Dmitri Volkov (Backend):** Confirm implementation feasibility

---

## Question 9: Multi-Language Support (i18n)

### Problem Statement

**Question:** Should intake forms support internationalization (multiple languages)?

**Context:**
- Current design assumes English-only templates
- RealmForge is English-first product (for now)
- Future expansion may require non-English support
- AI providers (Claude, OpenAI) support 95+ languages

**Scenarios:**
1. **Spanish-speaking user:** Selects "Aplicación Web Dinámica" → form renders in Spanish
2. **French Canadian client:** Needs bilingual forms (English/French) for compliance
3. **Global team:** Engineering in India, PM in Germany, users in Brazil

### Options

#### Option A: English-Only (Deferred i18n)

**Approach:**
- All templates, questions, help text in English
- No translation layer
- Defer i18n to Phase 2+ (>6 months out)

**Pros:**
* ✅ Simplest implementation (no i18n framework overhead)
* ✅ Faster time-to-market (no translation workflow)
* ✅ Single source of truth (no translation drift)

**Cons:**
* ❌ Excludes non-English speakers (limits market reach)
* ❌ Retro-fitting i18n later is expensive (template rewrites)
* ❌ AI clarification questions still in English (inconsistent UX)

**Risk:**
- Market research shows 35% of potential users prefer non-English intake
- Competitors offering multilingual forms (competitive disadvantage)

#### Option B: Full i18n (Day 1)

**Approach:**
- Templates store translations in JSONB: `{"en": "...", "es": "...", "fr": "..."}`
- User selects language preference → form renders in chosen language
- AI provider receives user language → responds in same language

**Pros:**
* ✅ Global-ready from launch
* ✅ Competitive advantage (most intake systems English-only)
* ✅ AI providers natively support multilingual (no extra work)

**Cons:**
* ❌ High initial overhead (translate 15 templates × 10-15 questions × 3 languages = 675 translations)
* ❌ Template authoring more complex (must provide translations)
* ❌ AI-generated forms need translation step (adds latency)

**Cost Estimate:**
- Professional translation: $0.10/word × 50 words/question × 15 templates × 12 questions × 2 languages = ~$1,800
- Maintenance: new templates require translation before approval

#### Option C: Hybrid (English default + AI translation on-demand)

**Approach:**
- Templates authored in English only
- User selects non-English → AI translates on-the-fly
- Translations cached in `intake_translations` table for reuse

**Pros:**
* ✅ No upfront translation cost (AI handles it)
* ✅ Supports 95+ languages instantly (AI capability)
* ✅ Template authoring stays simple (English only)
* ✅ Scales to rare languages without manual translation

**Cons:**
* ❌ Translation quality varies (AI may mistranslate domain terms)
* ❌ Adds latency (first-time translations take 2-3s)
* ❌ Token cost: ~500 tokens/template × $0.003/1K = $0.0015/translation (acceptable)

**Mitigation:**
- Human review for top 3 languages (Spanish, French, German)
- Translation cache eliminates repeated translation costs
- User feedback: "Report translation issue" button

### Recommendation

**✅ Option C (Hybrid: English default + AI translation on-demand)**

**Phased approach:**
1. **Phase 0a-0f (Weeks 1-12):** English-only (no translation layer)
2. **Phase 2 (Weeks 13-20):** Add AI translation for top 5 languages (Spanish, French, German, Portuguese, Chinese)
3. **Phase 3 (Weeks 21+):** Human-reviewed translations for top 3 languages

**Rationale:**
- Doesn't block Phase 0a launch (English-first is acceptable MVP)
- Scalable (AI handles long tail of languages)
- Cost-effective (no $1,800 upfront translation, <$0.01/intake for translations)
- Quality improves over time (human review for high-volume languages)

**Implementation:**
```sql
-- Translation cache table
CREATE TABLE intake_translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES intake_templates(id),
    question_id TEXT NOT NULL,
    field_type TEXT NOT NULL,  -- 'text', 'help_text', 'option_label'
    source_language TEXT NOT NULL DEFAULT 'en',
    target_language TEXT NOT NULL,
    source_text TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    is_ai_generated BOOLEAN DEFAULT TRUE,
    is_human_reviewed BOOLEAN DEFAULT FALSE,
    reviewed_by_actor_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(template_id, question_id, field_type, target_language)
);
```

**Decision Required From:**
- [ ] **Alex Rivera (PM):** Approve deferred i18n roadmap
- [ ] **Victor Chen (CEO):** Approve market strategy (English-first acceptable?)

---

## Question 10: Template Versioning Strategy

### Problem Statement

**Question:** How should RealmForge handle breaking changes to intake templates?

**Context:**
- Templates evolve (new questions, changed logic, deprecated features)
- Existing projects reference template version N
- Template updated to version N+1 with breaking changes
- Users may want to restart intake with new template

**Breaking change examples:**
1. **Question removed:** Q5 "Do you need caching?" → removed (now always included)
2. **Question ID changed:** `q2_auth_type` → `q2_authentication_method`
3. **Logic changed:** Conditional `show_if: Q1=true AND Q2=social` → `show_if: Q1=true`
4. **Module mapping changed:** `authentication` → splits into `auth_core` + `auth_providers`

**Scenarios:**
- **Existing project:** Uses template v1 → template updated to v2 → project still references v1
- **User restarts intake:** Wants to use latest template → old responses may not map to new questions
- **Admin edits template:** Changes question IDs → breaks existing sessions mid-flight

### Options

#### Option A: Immutable Templates (Copy-on-Write)

**Approach:**
- Templates are immutable once approved
- Edits create new version (v1, v2, v3...)
- Old versions remain queryable forever
- Projects reference specific version (never auto-upgrade)

**Pros:**
* ✅ No breaking changes (old projects always work)
* ✅ Deterministic behavior (version 1 always produces same results)
* ✅ Audit trail (historical versions preserved)
* ✅ Safe rollback (can revert to previous version)

**Cons:**
* ❌ Version sprawl (15 templates × 10 versions = 150 DB records)
* ❌ Users may use outdated templates (miss new features)
* ❌ Duplicate content (v2 is 95% same as v1, but full copy)

**Mitigation:**
- Template diff viewer: "What changed in v2?"
- User notification: "A new version is available — restart intake?"
- Deprecation policy: versions >12 months old marked deprecated

#### Option B: Semantic Versioning (MAJOR.MINOR.PATCH)

**Approach:**
- Version format: `1.0.0` (MAJOR.MINOR.PATCH)
- **MAJOR:** Breaking changes (question removed, logic changed)
- **MINOR:** Additive changes (new question added, help text improved)
- **PATCH:** Fixes (typo correction, clarification)
- Projects auto-upgrade for MINOR/PATCH, manual opt-in for MAJOR

**Pros:**
* ✅ Clear compatibility contract (semantic versioning widely understood)
* ✅ Non-breaking changes auto-applied (users get improvements)
* ✅ Breaking changes explicit (MAJOR bump signals "review required")

**Cons:**
* ❌ Complex upgrade logic (need to diff v1.0.0 → v1.1.0 and apply delta)
* ❌ Edge cases: what if question added in v1.1, then removed in v2.0? (responses orphaned)
* ❌ User confusion: "Why did my form change mid-intake?"

**Mitigation:**
- Auto-upgrade only for completed intakes (not in-progress)
- Changelog: "Template updated — here's what changed"
- Admin review required for MAJOR bumps

#### Option C: Schema Migration (Django-style)

**Approach:**
- Templates have version number + migration scripts
- Migration script defines how to upgrade responses from v1 → v2
- Example migration: `Q5_caching` removed → set `default_caching=true` for all projects

**Pros:**
* ✅ Lossless upgrades (migrations preserve data)
* ✅ Flexible (can handle complex transformations)
* ✅ Testable (run migration on copy, verify results)

**Cons:**
* ❌ High complexity (each template version needs migration script)
* ❌ Migration bugs risk data corruption
* ❌ Admin burden (must write/test migrations for every change)

**Mitigation:**
- Migration DSL (simple declarative syntax, not custom code)
- Dry-run mode (preview migration without applying)
- Rollback support (migrations reversible)

### Recommendation

**✅ Option A (Immutable Templates) + Option B (Semantic Versioning for guidance only)**

**Approach:**
- Templates immutable (copy-on-write for new versions)
- Use semantic versioning for communication (not enforcement)
- Users notified of new versions, opt-in to upgrade

**Rationale:**
- Simplest implementation (no migration scripts, no auto-upgrade logic)
- Safe (old projects never break)
- Transparent (version history visible to users)
- Defer complexity (can add migrations later if needed)

**Implementation:**
```sql
-- Template versioning (already in schema)
CREATE TABLE intake_templates (
    id UUID PRIMARY KEY,
    application_type_id TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1,  -- v1, v2, v3...
    semver TEXT,                     -- '1.0.0' (informational only)
    changelog TEXT,                  -- What changed in this version
    supersedes_template_id UUID REFERENCES intake_templates(id),  -- Previous version
    status TEXT NOT NULL DEFAULT 'active',  -- 'active', 'deprecated'
    ...
);

-- User notifications
CREATE TABLE template_upgrade_notifications (
    id UUID PRIMARY KEY,
    user_actor_id UUID NOT NULL,
    project_id UUID NOT NULL,
    current_template_version INT NOT NULL,
    latest_template_version INT NOT NULL,
    notification_sent_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    user_acknowledged BOOLEAN DEFAULT FALSE
);
```

**Decision Required From:**
- [ ] **Alex Rivera (PM):** Approve versioning policy + notification strategy
- [ ] **Rena Okafor (CTO):** Approve immutable template architecture

---

## Summary & Recommendations

| Question | Recommendation | Decision Owner | Timeline |
|----------|----------------|----------------|----------|
| **Q8: Migration Strategy** | Shadow Intake (AI backfill) + 6-month grandfather clause | PM + CTO | Phase 2 (Weeks 13-24) |
| **Q9: Multi-Language (i18n)** | English-first, AI translation on-demand (Phase 2+) | PM + CEO | Phase 2 (Weeks 13-20) |
| **Q10: Versioning Strategy** | Immutable templates + semantic versioning (guidance) | PM + CTO | Phase 0a (Week 1) |

### Critical Path Impact

- **Q10 (Versioning):** MUST decide before Phase 0a Week 1 (affects DB schema design)
- **Q8 (Migration):** Can defer to Phase 2 (doesn't block Phase 0a launch)
- **Q9 (i18n):** Can defer to Phase 2 (English-first acceptable for MVP)

### Next Steps

1. **PM (Alex) + CTO (Rena):** Review and approve recommendations above
2. **Update INTAKE_SYSTEM_DESIGN.md:** Replace Q8-Q10 with approved decisions
3. **Update ADR-001:** Add versioning decision (Option A approved)
4. **Update MASTER_BUILD_PLAN.md:** Add Phase 2 tasks for i18n + migration

---

**Status:** ⏳ PENDING PM + CTO APPROVAL  
**Target Decision Date:** Week 1, Day 1 (before Phase 0a kickoff)
