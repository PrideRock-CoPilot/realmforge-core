# Structured Intake System — Council Approval Presentation

**Meeting Date:** [TBD]  
**Presenters:** Alex Rivera (PM), Dr. Rena Okafor (CTO)  
**Attendees:** Victor Chen (CEO), Alex Rivera (PM), Dr. Rena Okafor (CTO), Council Members  
**Duration:** 40 minutes  
**Objective:** Obtain Council approval to launch Phase 0a implementation

---

## Agenda

1. **Executive Summary** (5 min) — Business case and key metrics
2. **System Architecture** (10 min) — Technical design and decisions
3. **Policy Decisions** (10 min) — Governance framework (Q5-Q10)
4. **Implementation Plan** (5 min) — Timeline, resources, phases
5. **Risk Analysis** (5 min) — Threats and mitigations
6. **Vote & Approval** (5 min) — Decision framework

---

## 1. Executive Summary

### The Problem We're Solving

**Current State (Phase 0 — Unstructured):**
* Agent-driven clarification via unstructured conversation
* 20+ minutes per intake, highly variable quality
* ~30K tokens per intake = **$10/intake cost**
* No learning — same questions asked repeatedly
* No consistent requirement capture
* No machine-readable output for workflows

**Impact:**
* 100 intakes/month = **$1,000/month in AI costs**
* Inconsistent project requirements → rework, scope creep
* No audit trail for "how did we decide to build X?"

### The Solution: Structured Intake System

**95% Deterministic + 5% AI-Assisted**

* **Deterministic:** Pre-authored decision trees for 15 common app types
* **AI-Assisted:** Fallback for edge cases, unlisted app types
* **Self-Improving:** AI questions auto-promote to canonical forms (≥5 occurrences)

**Key Metrics:**
* ⏱️ **5-minute completion** (down from 20+ minutes)
* 💰 **$0.50/intake cost** (down from $10 — **95% reduction**)
* 📊 **100% capture rate** (structured, machine-readable)
* 🎯 **Self-improving** (AI learns → reduces AI usage over time)

### Business Case

**Cost Savings:**
* Current: 100 intakes/month × $10 = **$1,000/month**
* Target: 100 intakes/month × $0.50 = **$50/month**
* **Savings: $950/month = $11,400/year**

**ROI:**
* Investment: 12 weeks × 2.3 FTE = 27.6 person-weeks
* Payback: 126 intakes (~6 weeks post-launch at 100/month volume)
* **Break-even: 4.5 months** from project start

**Strategic Value:**
* Foundation for AI-native software construction governance
* Consistent requirements → better project scoping
* Machine-readable intake → automated workflow generation
* Learning system → efficiency improves over time

---

## 2. System Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────┐
│                    User (PM/Developer)               │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│              MCP Tools (8 new tools)                 │
│  intake_start, intake_answer, intake_complete...     │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│             control-service (Orchestration)          │
│  • Session lifecycle management                      │
│  • AI provider coordination (Claude 3.5)            │
│  • Admin review workflow                             │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│          intake-engine (Pure Logic, No IO)           │
│  • Decision tree evaluation (AND/OR/NOT logic)      │
│  • Conditional navigation (show_if rules)           │
│  • Module/persona derivation                         │
│  • Template validation                               │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│              control-store (Persistence)             │
│  • intake_sessions, intake_templates                 │
│  • ai_intake_questions (pg_trgm indexed)            │
│  • admin_review_queue                                │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│                    PostgreSQL 15+                    │
│  • JSONB storage for decision trees                  │
│  • pg_trgm extension for similarity search           │
└─────────────────────────────────────────────────────┘
```

### Decision Trees Format

**Example: Static Website Template**
```json
{
  "questions": [
    {
      "id": "q1_purpose",
      "text": "What is the primary purpose?",
      "type": "single_select",
      "options": [
        {
          "value": "portfolio",
          "add_modules": ["static_hosting", "responsive_design"],
          "add_personas": ["frontend"]
        },
        ...
      ],
      "next_question": "q2_source"
    },
    {
      "id": "q2_source",
      "text": "How will you create content?",
      "type": "single_select",
      "show_if": { "question": "q1_purpose", "in": ["portfolio", "blog"] },
      ...
    }
  ]
}
```

**Conditional Logic:**
* `show_if` rules support AND/OR/NOT (max 3 levels deep)
* Questions skip if conditions not met
* Modules/personas accumulated as user progresses

---

## 3. Technical Decisions (ADR-001)

### Decision 1: Separate `intake-engine` Crate

**Decision:** Create standalone pure logic crate (no IO, no HTTP, no DB)

**Rationale:**
* ✅ **Separation of Concerns** — Intake logic isolated from workflow engine
* ✅ **Testability** — 100% unit testable without database/mocks
* ✅ **Reusability** — Decision tree engine reusable for future wizards
* ✅ **Layer Law Compliance** — Enforces api → service → engine → store

**Public Interface:**
```rust
pub fn evaluate_condition(condition, responses) -> bool;
pub fn get_next_question(tree, responses) -> Option<Question>;
pub fn derive_modules(tree, responses) -> Vec<Module>;
pub fn validate_template(tree) -> Result<()>;
```

**Council Approval Required:** ✅ **CTO (Rena)** sign-off on crate boundaries

---

### Decision 2: Pluggable AI Provider

**Decision:** Abstract AI behind trait, default to Claude 3.5 Sonnet

**Rationale:**
* ✅ **Vendor Independence** — No lock-in to Anthropic/OpenAI
* ✅ **Cost Optimization** — Can switch to cheaper models for simple tasks
* ✅ **Testing** — Mock provider for unit tests (no API calls)
* ✅ **Future-Proof** — New models added without code changes

**Provider Trait:**
```rust
pub trait AiProvider: Send + Sync {
    async fn ask_clarifying_question(...) -> AiQuestion;
    async fn generate_form(...) -> DecisionTree;
    async fn normalize_question(...) -> String;
}
```

**Implementations:**
* `ClaudeProvider` (default, temperature 0.3)
* `OpenAiProvider` (secondary, for cost comparison)
* `MockProvider` (testing only)

**Council Approval Required:** ✅ **CTO (Rena)** + **API Architect (Marcus)**

---

### Decision 3: pg_trgm Similarity (Phase 1)

**Decision:** Start with PostgreSQL pg_trgm, upgrade to embeddings if precision <60%

**Rationale:**
* ✅ **Fast Time-to-Value** — Zero external dependencies, instant similarity
* ✅ **Cost Control** — No embedding API costs ($0 vs $0.0001/1K tokens)
* ✅ **Operational Simplicity** — No background jobs, no vector DB sync
* ✅ **Clear Upgrade Path** — Add `embedding` column later if needed

**Similarity Query:**
```sql
SELECT normalized_question, similarity(normalized_question, $1) AS sim
FROM ai_intake_questions
WHERE similarity(normalized_question, $1) > 0.6
ORDER BY sim DESC LIMIT 5;
```

**Upgrade Trigger:**
* Precision <60% (too many false positives)
* Recall <40% (missing similar questions)
* Admin rejection rate >40%

**Council Approval Required:** ✅ **CTO (Rena)** + **Data Architect (Chen)**

---

### Decision 4: Database JSONB Storage

**Decision:** Store templates in PostgreSQL JSONB, not file-based

**Rationale:**
* ✅ **Transactional Consistency** — Template updates atomic with approval
* ✅ **Versioning Built-In** — Native version column + unique constraint
* ✅ **AI Integration** — AI-generated forms stored same as human-authored
* ✅ **Query Performance** — JSONB indexes support deep queries

**Schema:**
```sql
CREATE TABLE intake_templates (
    id UUID PRIMARY KEY,
    application_type_id TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1,
    decision_tree JSONB NOT NULL,
    feature_mappings JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    is_ai_generated BOOLEAN DEFAULT FALSE,
    ...
);
```

**Council Approval Required:** ✅ **CTO (Rena)** + **Data Architect (Chen)**

---

## 4. Policy Decisions (Q5-Q10)

### Q5: Form Approval Authority

**Question:** Who can approve AI-generated intake forms?

**Decision:** **Option A → Option B Transition** (Phased)
* **Phase 1 (First 5 forms):** Full Council approval required
* **Phase 2 (After 5 forms):** PM + CTO approval sufficient
* **Escalation:** PM/CTO can bring forms to Council if uncertain

**Rationale:**
* Early stage → need collective wisdom to establish quality bar
* After 5 successful approvals → delegate to PM/CTO for routine cases
* Council oversight without bottleneck

**Transition Criteria:**
* 5 AI-generated forms approved with no major issues
* PM and CTO demonstrate consistent judgment
* Council votes to delegate (recorded in minutes)

**Council Vote Required:** ✅ **This Decision (Q5)**

---

### Q6: Auto-Promotion Threshold

**Question:** Fixed or variable threshold for promoting AI questions?

**Decision:** **Fixed threshold of 5 occurrences** (3-month review)

**Rationale:**
* Simple to implement and explain
* Industry standard for "pattern detection"
* Review after 3 months: if precision <60%, switch to variable thresholds

**Alternative (deferred):** Variable thresholds by app type (Phase 2+)

**Council Vote Required:** ✅ **PM (Alex)** approval

---

### Q7: Draft Form Usage

**Question:** Can AI-generated forms be used immediately or require approval?

**Decision:** **Hybrid — One-time use with PII detection**

**Rationale:**
* User not blocked (can complete intake immediately)
* PII detection prevents sensitive data collection
* Admin review queue catches issues before form becomes canonical
* Rate limiting prevents abuse (5 AI forms/hour/user)

**Security Mitigations:**
* **T1 (Malicious Injection):** PII scanner, content validation
* **T2 (Quality Degradation):** Admin review before canonical
* **T3 (Resource Exhaustion):** Rate limiting (5/hour, $10/day cap)
* **T4 (PII Leakage):** Regex patterns for SSN, credit cards, passwords

**Council Vote Required:** ✅ **Security Architect (Fatima)** approval

---

### Q8-Q10: Roadmap Questions

**Q8: Migration Strategy**
* **Decision:** Shadow Intake (AI backfill) + 6-month grandfather clause
* **Rationale:** Avoids blocking Phase 0a launch, scales to N projects

**Q9: Multi-Language Support (i18n)**
* **Decision:** English-first, AI translation on-demand (Phase 2+)
* **Rationale:** Faster time-to-market, AI handles long tail of languages

**Q10: Template Versioning**
* **Decision:** Immutable templates (copy-on-write) + semantic versioning
* **Rationale:** No breaking changes, safe rollback, transparent history

**Council Vote Required:** ✅ **PM (Alex)** + **CTO (Rena)** approval

---

## 5. Implementation Plan

### 12-Week Timeline

```
Phase 0a (Weeks 1-2):  Core Engine + 3 App Types
  - intake-engine crate (evaluator, navigator, validator)
  - Migration 003 (7 tables)
  - Templates: Static Website, REST API, Dynamic Web App

Phase 0b (Weeks 3-4):  AI Assistance
  - ClaudeProvider implementation
  - AI fallback for unlisted app types
  - Edge-case clarification

Phase 0c (Week 5):     Smart Learning Engine
  - Background job: detect promotion candidates
  - pg_trgm similarity search (threshold 0.6)
  - Admin review queue integration

Phase 0d (Weeks 6-7):  AI Form Generation
  - generate_form() method
  - PII detection validation
  - One-time draft form usage

Phase 0e (Weeks 8-9):  Admin Review Portal
  - Review queue UI (React/Vue)
  - Form editor (JSON with preview)
  - Approval workflow

Phase 0f (Weeks 10-12): 15 Application Types
  - 12 additional templates (total 15)
  - QA validation (40+ test scenarios)
  - Production-ready system
```

### Resource Allocation

| Resource | Allocation | Duration | Notes |
|----------|-----------|----------|-------|
| **Dmitri Volkov** (Backend) | 80% | Weeks 1-12 | intake-engine, control-service, AI provider |
| **Priya Mehta** (Data) | 60% | Weeks 1-12 | Schema design, migrations, pg_trgm indexing |
| **Kai Larson** (Frontend) | 50% | Weeks 8-12 | Admin portal UI, form editor |
| **Meg Thompson** (QA) | 40% | Weeks 10-12 | Test plans, acceptance criteria |

**Total Effort:** 27.6 person-weeks

---

## 6. Risk Analysis

### Critical Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **AI API outage** | High (5% fallback blocked) | Medium | Mock provider for dev/test, fallback to manual clarification |
| **pg_trgm low precision** | Medium (bad auto-promotions) | Low | Admin review queue catches false positives, clear upgrade trigger |
| **Template quality issues** | High (bad requirements) | Low | 3 seed templates QA-validated, admin approval before canonical |
| **Resource contention** | Medium (delays) | Medium | Dmitri 80% dedicated, clear phase dependencies |
| **Scope creep** | Medium (timeline slip) | Low | Fixed 12-week timeline, deferred features to Phase 2+ |

### Security Threats (T1-T4)

**T1: Malicious Form Injection**
* Attacker crafts harmful AI-generated form
* **Mitigation:** Admin review queue, PII detection, content validation

**T2: Quality Degradation**
* Bad auto-promotions pollute canonical forms
* **Mitigation:** ≥5 occurrence threshold, human-in-loop approval

**T3: Resource Exhaustion**
* AI API abuse via repeated form generation
* **Mitigation:** Rate limiting (5 calls/hour/user), cost caps ($10/day)

**T4: PII Leakage**
* AI-generated forms request sensitive data
* **Mitigation:** Regex-based PII scanner, reject forms with SSN/credit cards

**Security Review Required:** ✅ **Fatima Al-Hassan** sign-off

---

## 7. Success Metrics

### Phase 0a-0f Validation (Weeks 1-12)

**Quantitative:**
* ✅ 100% of 3 seed templates pass validation
* ✅ Intake completion time <5 minutes (target: 4 min avg)
* ✅ Token usage <2K per intake (target: 1.5K avg)
* ✅ AI fallback <5% of intakes
* ✅ Zero template validation failures

**Qualitative:**
* ✅ PM (Alex) approves 3 templates for user experience
* ✅ CTO (Rena) approves architecture and crate boundaries
* ✅ QA (Meg) signs off on test coverage (40+ scenarios)

### Phase 1+ (Months 4-6)

**Learning System:**
* Auto-promotion precision >60%
* Admin rejection rate <40%
* 15 templates cover >95% of intakes

**Business Metrics:**
* Cost per intake <$0.50 (95% reduction achieved)
* User satisfaction >4/5 stars
* Requirement quality score >85%

---

## 8. Deliverables Completed

### Documentation (2,418+ lines)

| Document | Lines | Status |
|----------|-------|--------|
| INTAKE_SYSTEM_DESIGN.md | 1,468 | ✅ Complete |
| ADR-001-intake-system-architecture.md | 351 | ✅ Complete |
| MASTER_BUILD_PLAN.md (Phase 0a-0f) | 338 | ✅ Integrated |
| DEC-COUNCIL-PENDING (Q5) | 227 | ✅ Ready for Vote |
| DEC-PM-PENDING (Q6) | 162 | ✅ Ready for Vote |
| DEC-SECURITY-PENDING (Q7) | 198 | ✅ Ready for Vote |
| DEC-PRODUCT-PENDING (Q8-Q10) | 354 | ✅ Ready for Vote |
| APPROVAL-REQUEST-CTO-ADR-001 | 473 | ✅ Ready for CTO |
| INTAKE_SYSTEM_SIGNOFF_TRACKER | 389 | ✅ Status Tracker |

### Application Type Templates (3)

| Template | Questions | Modules | Status |
|----------|-----------|---------|--------|
| 01_static_website.json | 8 | 8 | ✅ QA Ready |
| 02_rest_api.json | 11 | 6 | ✅ QA Ready |
| 03_dynamic_web_app.json | 14 | 9 | ✅ QA Ready |

---

## 9. Voting Framework

### Approval Structure

**Primary Vote: Launch Authorization**
* **Motion:** "Authorize Phase 0a launch of Structured Intake System as specified in INTAKE_SYSTEM_DESIGN.md and ADR-001"
* **Voting Members:** Victor (CEO), Rena (CTO), Alex (PM)
* **Threshold:** Majority (2 of 3)

**Secondary Votes: Policy Decisions**
1. **Q5 (Form Approval Authority):** Council decision on approval delegation
2. **Q6 (Auto-Promotion Threshold):** PM (Alex) decision, Council advisory
3. **Q7 (Draft Form Usage):** Security (Fatima) decision, Council advisory
4. **Q8-Q10 (Roadmap Questions):** PM + CTO decision, Council advisory

### Vote Options

**Option 1: APPROVED**
* Phase 0a authorized to begin Week 1
* Dmitri and Priya start implementation immediately
* All 4 technical decisions (ADR-001) approved as specified
* All policy decisions (Q5-Q10) approved as recommended

**Option 2: APPROVED WITH MODIFICATIONS**
* Approval contingent on changes (specify below)
* Changes documented in amendment
* Re-vote if changes are substantive

**Option 3: DEFERRED**
* Additional review or information needed
* Specify concerns and next steps
* Reschedule vote for [date]

**Option 4: REJECTED**
* Do not proceed with current design
* Document reasons
* Alternative approach required

---

## 10. Voting Record

### Motion: Launch Authorization

**Motion:** "I move to approve the Structured Intake System for Phase 0a implementation as specified in INTAKE_SYSTEM_DESIGN.md and ADR-001-intake-system-architecture.md."

**Moved by:** _________________  
**Seconded by:** _________________  

**Vote:**
* **Victor Chen (CEO):** [ ] Approve [ ] Approve with Changes [ ] Defer [ ] Reject
* **Dr. Rena Okafor (CTO):** [ ] Approve [ ] Approve with Changes [ ] Defer [ ] Reject
* **Alex Rivera (PM):** [ ] Approve [ ] Approve with Changes [ ] Defer [ ] Reject

**Result:** _________________  
**Date:** _________________  

---

### Decision Q5: Form Approval Authority

**Motion:** "I move to approve the phased approach (Full Council for first 5 forms, then PM+CTO delegation) for AI-generated form approval."

**Vote:**
* **Victor Chen (CEO):** [ ] Approve [ ] Approve with Changes [ ] Defer [ ] Reject
* **Dr. Rena Okafor (CTO):** [ ] Approve [ ] Approve with Changes [ ] Defer [ ] Reject
* **Alex Rivera (PM):** [ ] Approve [ ] Approve with Changes [ ] Defer [ ] Reject

**Result:** _________________  

---

### Technical Decisions (ADR-001)

**Motion:** "I move to approve all four architecture decisions in ADR-001 as specified."

**Vote:**
* **Dr. Rena Okafor (CTO):** [ ] Approve [ ] Approve with Changes [ ] Defer [ ] Reject

**Decisions:**
* [ ] Decision 1: Separate `intake-engine` crate
* [ ] Decision 2: Pluggable AI provider (Claude 3.5 default)
* [ ] Decision 3: pg_trgm similarity (embeddings upgrade path)
* [ ] Decision 4: Database JSONB storage

**Result:** _________________  

---

## 11. Next Steps (Upon Approval)

### Immediate (Week 1, Day 1)
1. **Dmitri:** Create `intake-engine` crate skeleton
2. **Priya:** Write migration 003 (7 tables)
3. **Alex:** Load 3 seed templates into database
4. **Rena:** Review initial crate structure

### Week 1-2 (Phase 0a)
1. Implement decision tree evaluator
2. Implement conditional navigator
3. Implement module derivation
4. Run 3 test intakes (1 per template)
5. Validation gate: all tests pass

### Week 3 (Phase 0b Prep)
1. Review Phase 0a deliverables
2. Begin ClaudeProvider implementation
3. Test AI fallback flow

---

## 12. Questions & Discussion

**Open for Council Questions:**
* Technical architecture concerns?
* Policy decision clarifications?
* Resource allocation concerns?
* Timeline feasibility?
* Risk mitigation sufficiency?

---

## Appendix A: Reference Documents

**Core Design:**
* [INTAKE_SYSTEM_DESIGN.md](docs/INTAKE_SYSTEM_DESIGN.md) — Full specification (1,468 lines)
* [ADR-001-intake-system-architecture.md](docs/decisions/ADR-001-intake-system-architecture.md) — Architecture decisions (351 lines)
* [BACKEND_GAP_ANALYSIS.md](docs/BACKEND_GAP_ANALYSIS.md) — Section 3.6 (lines 608-754)
* [MASTER_BUILD_PLAN.md](docs/MASTER_BUILD_PLAN.md) — Phase 0a-0f (lines 135-473)

**Policy Decisions:**
* [DEC-COUNCIL-PENDING-intake-form-approval-authority.md](docs/decisions/DEC-COUNCIL-PENDING-intake-form-approval-authority.md)
* [DEC-PM-PENDING-auto-promotion-threshold.md](docs/decisions/DEC-PM-PENDING-auto-promotion-threshold.md)
* [DEC-SECURITY-PENDING-draft-form-usage.md](docs/decisions/DEC-SECURITY-PENDING-draft-form-usage.md)
* [DEC-PRODUCT-PENDING-intake-roadmap-questions.md](docs/decisions/DEC-PRODUCT-PENDING-intake-roadmap-questions.md)

**Approvals:**
* [APPROVAL-REQUEST-CTO-ADR-001.md](decisions/APPROVAL-REQUEST-CTO-ADR-001.md) — CTO sign-off (473 lines)
* [INTAKE_SYSTEM_SIGNOFF_TRACKER.md](docs/INTAKE_SYSTEM_SIGNOFF_TRACKER.md) — Central status tracker (389 lines)

**Application Templates:**
* [01_static_website.json](db/seed_data/templates/01_static_website.json)
* [02_rest_api.json](db/seed_data/templates/02_rest_api.json)
* [03_dynamic_web_app.json](db/seed_data/templates/03_dynamic_web_app.json)

---

**END OF PRESENTATION**

**Meeting Status:** ⏳ PENDING COUNCIL VOTE  
**Prepared by:** Alex Rivera (PM), Dr. Rena Okafor (CTO)  
**Date Prepared:** 2025-01-30  
**Version:** 1.0
