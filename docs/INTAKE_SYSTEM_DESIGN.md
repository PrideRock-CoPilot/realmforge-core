# RealmForge Structured Intake System — Design Document

**Date:** 2025-01-30  
**Status:** Design Phase  
**Authors:** Council Review (Victor, Alex, Rena, Fatima, Dmitri, Kai, Meg)  
**Purpose:** Deterministic requirement capture with AI learning engine

---

## Executive Summary

**Problem:** Current Phase 0 (INTAKE & REQUIREMENTS) is unstructured agent-driven clarification with no formal requirements capture, leading to inconsistent quality, heavy AI token usage, and no learning from past intakes.

**Solution:** A hybrid intake system that is 95% deterministic (rule-based conditional forms) and <5% AI-assisted (edge cases only), with a smart learning engine that improves over time.

**Key Innovation:** The system learns from AI-assisted edge cases and automatically generates new forms, creating a self-improving intake process.

---

## 1. System Architecture

### 1.1 Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     USER INTERFACE                          │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Form Renderer │  │ AI Chat      │  │ Progress        │  │
│  │              │  │ Interface    │  │ Tracker         │  │
│  └──────────────┘  └──────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                   INTAKE ENGINE (Core)                      │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Decision Tree Executor                               │  │
│  │ • Loads templates from database                      │  │
│  │ • Evaluates conditional logic                        │  │
│  │ • Tracks session state                               │  │
│  │ • Generates next question                            │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓
              ┌─────────────┴──────────────┐
              ↓                            ↓
┌─────────────────────────┐  ┌────────────────────────────────┐
│ STANDARD PATH (95%)     │  │ AI-ASSISTED PATH (<5%)        │
│ • Pre-defined forms     │  │ • Smart Learning Engine       │
│ • Known app types       │  │ • Question tracking           │
│ • Validated questions   │  │ • Auto-promotion              │
│ • Instant next question │  │ • Form generation             │
└─────────────────────────┘  └────────────────────────────────┘
              ↓                            ↓
              └─────────────┬──────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              MODULE/PERSONA ASSIGNMENT ENGINE               │
│  • Maps features → modules                                  │
│  • Assigns personas/skills                                  │
│  • Generates project configuration                          │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                ADMIN REVIEW PORTAL                          │
│  • Review AI-generated questions                            │
│  • Approve/reject form proposals                            │
│  • Edit and refine templates                                │
│  • Promote drafts to canonical                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Data Flow

**Standard Intake (95% of cases):**
```
User → Select App Type → Load Template → Answer Q1 → 
  Load Next Question (based on Q1) → Answer Q2 → ... → 
  Complete → Map Features → Assign Modules/Personas → 
  Create Project
```

**AI-Assisted Intake (<5% of cases):**
```
User → Describe App → AI Suggests App Type → 
  User Stuck on Question → AI Asks Clarifying Question → 
  Log AI Question → Answer → Continue Form OR 
  No Matching App Type → AI Generates Draft Form → 
  Use Draft Form → Log for Review → Complete Intake
```

**Learning Loop:**
```
AI Question Logged → Track Frequency → 
  Threshold Reached (5+ occurrences) → 
  Admin Notified → Review → Approve → 
  Add to Canonical Form
```

---

## 2. Database Schema

### 2.1 Core Tables

```sql
-- Application type definitions
CREATE TABLE application_types (
    id TEXT PRIMARY KEY,                    -- '03_dynamic_web_app'
    name TEXT NOT NULL,                     -- 'Dynamic Web Application'
    description TEXT NOT NULL,
    complexity TEXT NOT NULL,               -- 'medium', 'high', etc.
    timeline_weeks TEXT NOT NULL,           -- '4-8 weeks'
    status TEXT NOT NULL DEFAULT 'active',  -- 'active', 'draft', 'deprecated'
    is_ai_generated BOOLEAN DEFAULT FALSE,  -- TRUE if AI created this
    created_by_actor_id UUID,
    approved_by_actor_id UUID,              -- NULL if still draft
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INT NOT NULL DEFAULT 1
);

-- Decision tree templates
CREATE TABLE intake_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_type_id TEXT NOT NULL REFERENCES application_types(id),
    version INT NOT NULL DEFAULT 1,
    decision_tree JSONB NOT NULL,           -- Full decision tree structure
    feature_mappings JSONB NOT NULL,        -- Feature → module mappings
    status TEXT NOT NULL DEFAULT 'active',  -- 'active', 'draft', 'deprecated'
    is_ai_generated BOOLEAN DEFAULT FALSE,
    created_by_actor_id UUID,
    approved_by_actor_id UUID,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(application_type_id, version)
);

-- Active intake sessions
CREATE TABLE intake_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID REFERENCES intake_templates(id),
    application_type_id TEXT REFERENCES application_types(id),
    user_actor_id UUID NOT NULL,
    session_type TEXT NOT NULL DEFAULT 'standard',  -- 'standard', 'ai_assisted', 'ai_generated'
    responses JSONB NOT NULL DEFAULT '{}',          -- {question_id: answer}
    ai_questions JSONB DEFAULT '[]',                -- [{question, answer, timestamp}]
    derived_modules JSONB,                          -- [module1, module2, ...]
    derived_personas JSONB,                         -- [persona1, persona2, ...]
    status TEXT NOT NULL DEFAULT 'in_progress',     -- 'in_progress', 'completed', 'abandoned'
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    project_id UUID                                 -- FK to projects table (after creation)
);

-- Smart learning: AI-asked questions
CREATE TABLE ai_intake_questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES intake_sessions(id),
    question_text TEXT NOT NULL,
    question_context JSONB NOT NULL,        -- {previous_answers, stuck_question_id}
    answer_text TEXT,
    application_type_id TEXT REFERENCES application_types(id),
    asked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    promoted_to_template BOOLEAN DEFAULT FALSE,
    promotion_date TIMESTAMPTZ
);

-- Question frequency tracking (for auto-promotion)
CREATE TABLE question_frequency (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_text_normalized TEXT NOT NULL, -- Normalized version for matching
    application_type_id TEXT REFERENCES application_types(id),
    occurrence_count INT NOT NULL DEFAULT 1,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    status TEXT NOT NULL DEFAULT 'tracked',  -- 'tracked', 'under_review', 'promoted', 'rejected'
    UNIQUE(question_text_normalized, application_type_id)
);

-- Admin review queue
CREATE TABLE admin_review_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_type TEXT NOT NULL,              -- 'new_question', 'new_form', 'form_edit'
    application_type_id TEXT REFERENCES application_types(id),
    template_id UUID REFERENCES intake_templates(id),
    proposed_change JSONB NOT NULL,         -- Full proposed change
    justification TEXT,                     -- Why this change is needed
    source_type TEXT NOT NULL,              -- 'ai_learning', 'user_request', 'admin_initiated'
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected'
    created_by_actor_id UUID,
    reviewed_by_actor_id UUID,
    reviewed_at TIMESTAMPTZ,
    review_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Audit trail
CREATE TABLE intake_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL,              -- 'template', 'question', 'application_type'
    entity_id UUID NOT NULL,
    action TEXT NOT NULL,                   -- 'created', 'approved', 'rejected', 'edited'
    actor_id UUID NOT NULL,
    changes JSONB,                          -- Full diff
    justification TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 2.2 Supporting Indexes

```sql
-- Performance indexes
CREATE INDEX idx_intake_sessions_user ON intake_sessions(user_actor_id);
CREATE INDEX idx_intake_sessions_status ON intake_sessions(status);
CREATE INDEX idx_ai_questions_promoted ON ai_intake_questions(promoted_to_template);
CREATE INDEX idx_question_frequency_count ON question_frequency(occurrence_count DESC);
CREATE INDEX idx_admin_queue_status ON admin_review_queue(status);

-- Full-text search for question matching
CREATE INDEX idx_question_text_search ON question_frequency 
    USING gin(to_tsvector('english', question_text_normalized));
```

---

## 3. Decision Tree Format

### 3.1 JSON Structure (Example)

```json
{
  "application_type_id": "03_dynamic_web_app",
  "version": 1,
  "metadata": {
    "created_by": "system",
    "last_updated": "2025-01-30T00:00:00Z",
    "total_questions": 12,
    "average_completion_time_minutes": 8
  },
  "questions": [
    {
      "id": "q1_auth",
      "text": "Does your application require user authentication?",
      "type": "boolean",
      "help_text": "Authentication allows users to log in and access protected features.",
      "default": true,
      "validation": {
        "required": true
      },
      "on_true": {
        "add_modules": ["authentication", "session_management", "database"],
        "next_question": "q2_auth_type"
      },
      "on_false": {
        "next_question": "q3_data_storage"
      }
    },
    {
      "id": "q2_auth_type",
      "text": "What type of authentication do you need?",
      "type": "multi_select",
      "help_text": "Select all authentication methods you want to support.",
      "options": [
        {
          "value": "email_password",
          "label": "Email/Password - Standard username/password",
          "add_modules": ["password_auth", "password_reset"],
          "add_personas": ["backend"]
        },
        {
          "value": "social",
          "label": "Social Login - Google, Apple, GitHub",
          "add_modules": ["oauth_provider", "social_auth"],
          "add_personas": ["backend"]
        }
      ],
      "validation": {
        "required": true,
        "min_selections": 1
      },
      "next_question": "q3_data_storage"
    }
  ],
  "feature_mappings": {
    "authentication": {
      "modules": ["authentication", "session_management", "database"],
      "personas": ["backend", "security-architect"],
      "deliverables": ["auth_api", "session_store", "login_ui"]
    }
  },
  "completion_rules": {
    "required_questions": ["q1_auth", "q3_data_storage", "q4_hosting"],
    "minimum_modules": 3
  }
}
```

### 3.2 Conditional Logic DSL

**Supported Operators:**
- `AND`, `OR`, `NOT`
- `EQUALS`, `CONTAINS`, `IN`
- Max nesting depth: 3 levels

**Example:**
```json
{
  "id": "q5_advanced",
  "text": "Do you need advanced features?",
  "show_if": {
    "operator": "AND",
    "conditions": [
      {"question": "q1_auth", "equals": true},
      {
        "operator": "OR",
        "conditions": [
          {"question": "q2_auth_type", "contains": "sso"},
          {"question": "q2_auth_type", "contains": "mfa"}
        ]
      }
    ]
  }
}
```

---

## 4. AI Learning Engine

### 4.1 Question Tracking

**When AI asks a question not on the form:**
1. Log to `ai_intake_questions` with full context
2. Normalize question text (lowercase, remove punctuation)
3. Check `question_frequency` for similar questions
4. If exists: increment count, update last_seen
5. If new: create entry with count=1
6. If count ≥ 5: Create `admin_review_queue` entry

**Normalization Algorithm:**
```python
def normalize_question(question: str) -> str:
    """Normalize question for similarity matching."""
    q = question.lower()
    q = re.sub(r'[^\w\s?]', '', q)
    q = ' '.join(q.split())
    q = re.sub(r'^(do you|does|will|would|should|can|is|are)\s+', '', q)
    return q
```

### 4.2 AI-Generated Forms

**Trigger:** User selects "My app type isn't listed" OR AI detects no matching template

**Workflow:**
1. AI asks user to describe their app in 2-3 sentences
2. AI analyzes description and generates decision tree (10-15 questions)
3. Store as `application_types.status='draft'` and `intake_templates.status='draft'`
4. User completes intake using draft form
5. Create `admin_review_queue` entry (type='new_form')
6. Admin reviews and approves/rejects/edits

### 4.3 Auto-Promotion Logic

**Threshold Rules:**
- Question asked ≥5 times across different sessions → Queue for review
- New form used ≥3 times with completion → Queue for promotion
- AI-generated question has >80% answer consistency → Auto-suggest

---

## 5. Admin Review Portal

### 5.1 Review Interface

**Dashboard:**
```
┌─────────────────────────────────────────────────────────┐
│ ADMIN REVIEW QUEUE                    [Filter: Pending] │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  🟡 NEW QUESTION (5 occurrences)                       │
│     "Do you need offline mode support?"                │
│     App Type: Progressive Web App                      │
│     First seen: 2025-01-15                             │
│     [Review] [Approve] [Reject]                        │
│                                                         │
│  🟢 NEW FORM (used 3 times, 100% completion)           │
│     "IoT Device Management Platform"                    │
│     Questions: 14 | Modules: 8 | Personas: 5           │
│     [Review Form] [Approve] [Reject] [Edit]            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Review Actions:**
1. **Approve** → Promotes to canonical, updates references
2. **Reject** → Marks rejected, adds to blocklist
3. **Edit** → Opens editor, saves as new draft version
4. **Request Changes** → Sends back to AI with notes

### 5.2 Form Editor

**Visual Decision Tree Editor:**
- Drag-and-drop question ordering
- Visual conditional logic builder
- Feature mapping editor
- Preview mode (test the form)
- Version diff viewer

**Validation Rules:**
- All questions must have unique IDs
- Conditional logic must reference valid questions
- Feature mappings must reference valid modules
- No circular dependencies
- All paths must lead to completion

---

## 6. Security & Governance

### 6.1 Threat Model

**T1: Malicious Form Injection**
- **Mitigation:** All AI-generated forms require admin approval, input validation, rate limiting

**T2: Question Poisoning**
- **Mitigation:** Frequency threshold requires multiple sessions, admin review before promotion

**T3: Data Leakage via AI Questions**
- **Mitigation:** AI questions logged separately, PII detection, separate encryption

**T4: Unauthorized Form Modification**
- **Mitigation:** Form editing requires admin role, all changes audited, version control

### 6.2 Approval Workflow

**Who Can Approve:**
- New Questions: PM (Alex) or CTO (Rena)
- New Forms: Council Decision Required
- Form Edits: PM (Alex)

**Approval Criteria:**
1. **Completeness** — Captures all necessary requirements?
2. **Clarity** — Questions unambiguous?
3. **Non-redundancy** — No overlap with existing questions?
4. **Relevance** — Applicable to ≥80% of app type?
5. **Neutrality** — Avoids bias toward specific solutions?

---

## 7. Workflows

### 7.1 Standard Intake (95%)

```
User lands on intake page
    ↓
Select application type (dropdown)
    ↓
Load canonical template
    ↓
Render first question
    ↓
User answers → Evaluate logic → Load next question
    ↓
Repeat until complete
    ↓
Show summary → User confirms → Create project
```

### 7.2 AI-Assisted Intake (<5%)

**Scenario 1: User stuck**
```
User clicks "Ask AI for help"
    ↓
AI asks clarifying question (logged)
    ↓
User answers → AI translates to form answer
    ↓
Continue form
```

**Scenario 2: App type not listed**
```
User selects "My app isn't listed"
    ↓
AI: "Describe your app"
    ↓
AI generates draft form (10-15 questions)
    ↓
Store as draft → Use for session → Queue for review
```

### 7.3 Learning Loop

```
AI asks question not on form
    ↓
Log to ai_intake_questions
    ↓
Normalize, check frequency
    ↓
If count ≥ 5: Create admin review entry
    ↓
Admin reviews → If approved: Add to canonical template
```

---

## 8. MCP Tools

### 8.1 Intake Session Management

```typescript
// Start new intake session
intake_start(
  actor_id: UUID,
  application_type_id?: string
) -> {
  session_id: UUID,
  template_id: UUID,
  first_question: Question
}

// Answer question
intake_answer(
  session_id: UUID,
  question_id: string,
  answer: any
) -> {
  next_question?: Question,
  derived_modules: string[],
  completion_percentage: number
}

// Complete intake
intake_complete(
  session_id: UUID
) -> {
  project_id: UUID,
  requirements_artifact_id: UUID
}
```

### 8.2 AI Assistance

```typescript
// Request AI help
intake_ai_assist(
  session_id: UUID,
  stuck_question_id: string,
  user_description: string
) -> {
  ai_question: string,
  context: string
}

// Generate draft form
intake_generate_form(
  actor_id: UUID,
  app_description: string
) -> {
  application_type: ApplicationType,  // draft
  template: IntakeTemplate,           // draft
  session_id: UUID
}
```

### 8.3 Admin Tools

```typescript
// Get review queue
admin_get_review_queue(
  status?: 'pending' | 'approved' | 'rejected'
) -> {
  reviews: ReviewQueueItem[],
  summary: {
    pending_count: number,
    new_questions: number,
    new_forms: number
  }
}

// Review decision
admin_review_decision(
  review_id: UUID,
  decision: 'approve' | 'reject' | 'edit',
  notes?: string
) -> {
  success: boolean,
  promoted_entity_id?: UUID
}

// Edit template
admin_edit_template(
  template_id: UUID,
  changes: TemplateChanges
) -> {
  new_version: number,
  review_queue_id: UUID
}
```

---

## 9. Implementation Phases

### Phase 1: Core Intake Engine (Weeks 1-2)

**Deliverables:**
1. `intake-engine` crate (decision tree executor)
2. Database migrations
3. Seed data: 3 application types (Static Website, REST API, Dynamic Web App)
4. MCP tools: `intake_start`, `intake_answer`, `intake_complete`
5. Basic form renderer UI

**Acceptance Criteria:**
- Can start intake for 3 app types
- Conditional logic works
- Module/persona assignment accurate

### Phase 2: AI Assistance (Weeks 3-4)

**Deliverables:**
1. AI assistance integration
2. Question tracking system
3. MCP tools: `intake_ai_assist`, `intake_ai_answer`
4. "Ask AI" button in UI

**Acceptance Criteria:**
- User can request AI help
- AI questions logged
- Similarity matching works

### Phase 3: Smart Learning (Week 5)

**Deliverables:**
1. Question frequency tracking
2. Auto-promotion logic (threshold: 5)
3. Admin notification system

**Acceptance Criteria:**
- Questions tracked across sessions
- Admin notified at threshold
- No false positives

### Phase 4: AI Form Generation (Weeks 6-7)

**Deliverables:**
1. AI form generation module
2. Draft template storage
3. MCP tool: `intake_generate_form`
4. Basic template editor

**Acceptance Criteria:**
- AI generates valid decision trees
- Draft forms usable

### Phase 5: Admin Review Portal (Weeks 8-9)

**Deliverables:**
1. Admin review dashboard UI
2. Review queue management
3. Visual template editor
4. Version diff viewer

**Acceptance Criteria:**
- Admin can review pending items
- Can approve/reject with audit trail
- Can edit templates visually

### Phase 6: Expansion (Weeks 10-12)

**Deliverables:**
1. 12 more application types (total 15)
2. Comprehensive testing (40+ scenarios)
3. Performance optimization
4. Documentation

**Acceptance Criteria:**
- 15 app types with tested forms
- <5 min completion time
- <1% AI assistance usage (mature state)

---

## 10. Metrics & Success Criteria

### 10.1 System Metrics

**Performance:**
- Average completion time: <5 minutes
- Form rendering: <500ms
- AI response: <3 seconds

**Quality:**
- Form abandonment: <10%
- AI assistance usage: <5%
- Question promotion acceptance: >80%
- Generated form approval: >60%

**Learning:**
- New questions identified/week: 5-10
- Questions promoted/month: 2-4
- New forms approved/quarter: 1-2

### 10.2 Business Metrics

**Efficiency:**
- Requirement quality: 8/10+
- Clarification loops: <2
- Time to project creation: <10 min
- Token cost per intake: <$0.50

**User Satisfaction:**
- Intake satisfaction: 4.5/5+
- "Would use again": >90%
- Support tickets per 100 intakes: <5

### 10.3 Monitoring

**Real-time Dashboards:**
1. Intake funnel (drop-off analysis)
2. AI assistance usage
3. Question frequency trends
4. Form performance by app type
5. Admin queue metrics

**Alerts:**
- Completion rate <85%
- AI assistance >10%
- Form abandonment >20%
- Admin queue backlog >20

---

## 11. Open Questions & Decisions

### 11.1 Technical Decisions

**Q1: Crate Architecture**
- **Recommendation:** Separate `intake-engine` crate (separation of concerns)

**Q2: AI Provider**
- **Recommendation:** Pluggable with default to Claude 3.5

**Q3: Question Similarity**
- **Recommendation:** Start with pg_trgm, upgrade to embeddings if needed

**Q4: Template Storage**
- **Recommendation:** Database (JSONB) for easier versioning

### 11.2 Policy Decisions

**Q5: Who Can Approve New Forms?**
- **Decision Required:** Council

**Q6: Auto-Promotion Threshold**
- **Decision Required:** PM (Alex)

**Q7: AI-Generated Form Usage**
- **Decision Required:** Security Architect (Fatima)

### 11.3 Roadmap Questions

**Q8: Migration Strategy**
- How to handle existing projects?

**Q9: Multi-Language Support**
- Should forms support i18n?

**Q10: Versioning Strategy**
- How to handle breaking changes?

---

## 12. Appendix

### 12.1 Example: Full Intake Session

**User:** Builds a SaaS product (dynamic web app)

```
1. Select App Type: "Dynamic Web Application"
2. Q1: Authentication? → Yes
   Modules: [authentication, session_management, database]
3. Q2: Auth type? → [email_password, social]
   Modules: [password_auth, oauth_provider, social_auth]
4. Q3: Data storage? → [structured, files]
   Modules: [relational_db, object_store, file_upload]
5. [AI Assist] "Not sure about caching"
   AI: "Will you have >10K concurrent users?"
   User: "Not initially"
   AI translates: cache_strategy = "optional, plan for later"

Final:
- Modules: [authentication, password_auth, social_auth, database, 
           object_store, file_upload, cloud_deployment, frontend, backend_api]
- Personas: [backend, frontend, data-architect, devops]
- Creates: Project + REQUIREMENTS.md + Team Assignments
```

### 12.2 Example: AI-Generated Form

**User:** "I need an operating system for embedded IoT devices"

```
AI Analysis:
- No matching app types
- Keywords: "operating system", "embedded", "IoT"

AI Generates Draft:
{
  "application_type_id": "ai_generated_iot_os",
  "name": "IoT Operating System",
  "complexity": "very_high",
  "timeline_weeks": "24-48",
  "questions": [
    {"id": "q1_rtos", "text": "Real-time guarantees?", ...},
    {"id": "q2_arch", "text": "Target architecture?", ...},
    // ... 11 more questions
  ]
}

Stored as draft → User completes → Queued for admin review
```

---

**END OF DESIGN DOCUMENT**
