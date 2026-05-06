# ADR-001: Intake System Architecture

**Date:** 2025-01-30  
**Status:** PROPOSED (Pending Council Approval)  
**Deciders:** RealmForge Decision Council (Victor, Rena, Alex), CTO (Rena)  
**Related Documents:**
- `docs/INTAKE_SYSTEM_DESIGN.md` (full specification)
- `docs/BACKEND_GAP_ANALYSIS.md` Section 3.6
- `docs/decisions/DEC-COUNCIL-PENDING-intake-form-approval-authority.md`

---

## Context

RealmForge's Phase 0 (INTAKE & REQUIREMENTS) currently relies on unstructured agent-driven clarification, leading to:
- Inconsistent requirement quality
- Heavy AI token usage (~$5-10 per intake)
- No learning from past interactions
- No structured requirement capture

The Structured Intake System addresses this with a hybrid approach: 95% deterministic (rule-based forms) + 5% AI-assisted (edge cases), with a smart learning engine that auto-promotes frequent AI questions to canonical forms.

This ADR documents **four foundational architectural decisions** that must be made before implementation begins.

---

## Decision 1: Crate Architecture — Separate `intake-engine` Crate

### Status
**RECOMMENDED** (Pending CTO approval)

### Context
The intake system needs decision tree execution, conditional logic evaluation, and module/persona assignment. Two options:
1. **Monolithic:** Add intake logic to existing `control-service` crate
2. **Separated:** New `intake-engine` crate with clear boundaries

### Decision
Create a **separate `intake-engine` crate** responsible for:
- Decision tree parsing and validation
- Conditional logic evaluation (AND/OR/NOT, max depth 3)
- Session state management
- Module/persona derivation from responses
- Template versioning logic

**Layer boundaries:**
```
intake-engine (pure logic, no IO)
      ↓
control-service (orchestration)
      ↓
control-store (persistence)
      ↓
PostgreSQL
```

### Rationale

**Pros:**
* ✅ **Separation of Concerns:** Intake logic isolated from project/workflow logic
* ✅ **Testability:** Pure logic crate can be tested without database
* ✅ **Reusability:** Decision tree engine can be used outside intake (e.g., onboarding wizards)
* ✅ **Clear Ownership:** Single team owns intake domain
* ✅ **Independent Evolution:** Can change intake logic without touching workflow engine

**Cons:**
* ❌ Additional crate complexity
* ❌ Need to define clear interface between intake-engine and control-service
* ❌ Overhead of maintaining another crate

**Alternatives Considered:**
- **Add to control-service:** Simpler initially, but couples intake to project lifecycle logic, harder to test
- **Merge with workflow-engine:** Logical pairing, but violates single responsibility principle

### Consequences

**Positive:**
- Clean separation allows intake system to evolve independently
- Easier unit testing (pure logic, no IO)
- Can reuse decision tree engine for future wizards/forms
- Clear interface contract forces good design

**Negative:**
- Need to define stable interface between `intake-engine` ↔ `control-service`
- Slightly more complex dependency graph
- Need separate crate documentation

**Mitigation:**
- Document interface contract clearly (input: session state, output: next question + derived state)
- Use Rust traits to define boundaries
- Keep `intake-engine` pure (no database, no HTTP, no file IO)

### Implementation Notes

**Crate structure:**
```
crates/intake-engine/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── decision_tree.rs      # Tree structure and DSL
│   ├── evaluator.rs           # Conditional logic evaluation
│   ├── session.rs             # Session state management
│   ├── mapper.rs              # Feature → module/persona mapping
│   └── validator.rs           # Template validation
└── tests/
    └── integration_tests.rs
```

**Public interface:**
```rust
pub struct IntakeEngine;

impl IntakeEngine {
    pub fn evaluate_condition(
        condition: &Condition,
        responses: &HashMap<QuestionId, Answer>
    ) -> Result<bool, EvaluationError>;

    pub fn get_next_question(
        template: &DecisionTree,
        responses: &HashMap<QuestionId, Answer>
    ) -> Result<Option<Question>, TreeError>;

    pub fn derive_modules(
        template: &DecisionTree,
        responses: &HashMap<QuestionId, Answer>
    ) -> Vec<ModuleName>;

    pub fn validate_template(
        template: &DecisionTree
    ) -> Result<(), ValidationError>;
}
```

---

## Decision 2: AI Provider — Pluggable Architecture with Claude 3.5 Default

### Status
**RECOMMENDED** (Pending CTO approval)

### Context
AI is used for:
1. Clarifying questions when users stuck
2. Generating draft forms for unlisted app types
3. Normalizing questions for similarity matching

Hard-coding a single provider creates vendor lock-in and limits flexibility.

### Decision
Implement a **pluggable AI provider architecture** with Claude 3.5 Sonnet as the default provider.

**Architecture:**
```rust
pub trait AiProvider: Send + Sync {
    async fn ask_clarifying_question(
        &self,
        context: &IntakeContext,
        stuck_question_id: &str
    ) -> Result<AiQuestion, AiError>;

    async fn generate_form(
        &self,
        app_description: &str
    ) -> Result<DecisionTree, AiError>;

    async fn normalize_question(
        &self,
        question_text: &str
    ) -> Result<String, AiError>;
}

pub struct ClaudeProvider { /* ... */ }
pub struct OpenAiProvider { /* ... */ }
pub struct MockProvider { /* ... */ }  // For testing
```

**Configuration:**
```toml
[ai]
provider = "claude"  # or "openai", "mock"
model = "claude-3-5-sonnet-20241022"
max_tokens = 1000
temperature = 0.3
```

### Rationale

**Pros:**
* ✅ **Vendor Independence:** Can switch providers without code changes
* ✅ **Cost Optimization:** Can use cheaper models for simple tasks
* ✅ **Testing:** Mock provider for unit tests
* ✅ **Multi-Provider:** Can use different providers for different tasks
* ✅ **Future-Proof:** New models can be added easily

**Cons:**
* ❌ More complex implementation
* ❌ Need to abstract over provider-specific features
* ❌ Lowest-common-denominator API

**Alternatives Considered:**
- **Hard-code Claude:** Simpler, but vendor lock-in
- **Hard-code OpenAI:** Simpler, but vendor lock-in
- **Support multiple providers without abstraction:** Messy conditionals throughout codebase

### Consequences

**Positive:**
- Can evaluate different models for cost/quality trade-offs
- Can switch providers if pricing changes
- Can use local models for development/testing
- Clean separation between AI logic and provider

**Negative:**
- Need to maintain provider implementations
- Some provider-specific features may not be available
- Testing across multiple providers is more complex

**Mitigation:**
- Start with Claude provider only (defer other providers to Phase 2+)
- Document provider contract clearly
- Use feature flags to enable/disable providers

### Implementation Notes

**Provider selection precedence:**
1. Environment variable: `REALMFORGE_AI_PROVIDER`
2. Config file: `config.toml`
3. Default: `claude`

**Rate limiting per provider:**
```rust
pub struct RateLimitedProvider<P: AiProvider> {
    inner: P,
    rate_limiter: RateLimiter,
}
```

**Cost tracking:**
```rust
pub struct CostTrackedProvider<P: AiProvider> {
    inner: P,
    cost_tracker: Arc<Mutex<CostTracker>>,
}
```

---

## Decision 3: Question Similarity — pg_trgm with Embeddings Upgrade Path

### Status
**RECOMMENDED** (Pending Data Architect approval)

### Context
The learning engine needs to detect when AI asks similar questions across sessions to auto-promote them. Two approaches:
1. **Simple text matching:** PostgreSQL `pg_trgm` (trigram similarity)
2. **Semantic matching:** Embeddings + vector similarity (pgvector)

### Decision
Start with **pg_trgm** (trigram similarity), with clear upgrade path to embeddings if needed.

**Phase 1 (Weeks 1-10):**
```sql
-- Install extension
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Similarity search
SELECT 
    question_text_normalized,
    similarity(question_text_normalized, 'do you need offline mode') AS sim
FROM question_frequency
WHERE similarity(question_text_normalized, 'do you need offline mode') > 0.6
ORDER BY sim DESC
LIMIT 5;
```

**Phase 2+ (If similarity matching inadequate):**
```sql
-- Install pgvector
CREATE EXTENSION IF NOT EXISTS vector;

-- Add embedding column
ALTER TABLE question_frequency 
ADD COLUMN embedding vector(1536);

-- Similarity search
SELECT question_text_normalized
FROM question_frequency
ORDER BY embedding <-> '[0.1, 0.2, ...]'::vector
LIMIT 5;
```

### Rationale

**Why pg_trgm First:**
* ✅ **Simple:** No external API calls, no embeddings to generate
* ✅ **Fast:** Indexed trigram matching is very fast
* ✅ **Good Enough:** Works well for exact and near-exact matches
* ✅ **Zero Cost:** No AI API calls for similarity matching
* ✅ **Deterministic:** Same question always gets same similarity score

**Why Not Embeddings Initially:**
* ❌ **Complex:** Requires embedding generation, vector storage, vector indexing
* ❌ **Cost:** Every AI question requires embedding API call (~$0.0001 per question, adds up)
* ❌ **Latency:** Embedding generation adds 100-300ms per question
* ❌ **Overhead:** Need to manage embedding model versions

**When to Upgrade:**
Trigger embeddings upgrade if:
- Auto-promotion precision <60% (too many false positives)
- Admin rejection rate >40% for promoted questions
- User reports similar questions not being detected

### Consequences

**Positive:**
- Simple implementation for Phase 1
- Zero external dependencies
- Fast and deterministic
- Clear upgrade path when needed

**Negative:**
- May miss semantically similar questions with different wording
- Limited to lexical similarity
- Manual threshold tuning required

**Mitigation:**
- Track precision/recall metrics from Phase 1
- Document upgrade trigger criteria
- Pre-design embeddings schema for easy migration

### Implementation Notes

**Normalization function:**
```rust
pub fn normalize_question(question: &str) -> String {
    let mut q = question.to_lowercase();
    q = q.replace(|c: char| !c.is_alphanumeric() && c != ' ', "");
    q = q.split_whitespace().collect::<Vec<_>>().join(" ");
    // Remove common question prefixes
    for prefix in &["do you", "does", "will", "would", "should", "can", "is", "are"] {
        if q.starts_with(prefix) {
            q = q[prefix.len()..].trim_start().to_string();
            break;
        }
    }
    q
}
```

**Similarity threshold:**
- Initial: 0.6 (60% similarity)
- Adjustable via config
- Track false positive/negative rates

**Index:**
```sql
CREATE INDEX idx_question_trgm ON question_frequency 
USING gin(question_text_normalized gin_trgm_ops);
```

---

## Decision 4: Template Storage — Database JSONB (Not File-Based)

### Status
**RECOMMENDED** (Pending Data Architect approval)

### Context
Decision tree templates can be stored in:
1. **Database (JSONB):** Templates in `intake_templates.decision_tree` column
2. **Files:** Templates as JSON files in filesystem/object store
3. **Hybrid:** Metadata in DB, templates in files

### Decision
Store templates as **JSONB in PostgreSQL** (`intake_templates.decision_tree` column).

### Rationale

**Pros:**
* ✅ **Versioning:** Easy to track template versions in database
* ✅ **Transactions:** Template creation/approval is transactional
* ✅ **Querying:** Can query template structure (e.g., "which templates use SSO?")
* ✅ **Integrity:** Foreign key relationships maintained
* ✅ **Audit:** Template changes logged in audit table
* ✅ **Simplicity:** No filesystem/object store coordination

**Cons:**
* ❌ Large JSONB blobs in database
* ❌ Harder to edit templates (not plain text files)
* ❌ Database size increases

**Alternatives Considered:**
- **Files:** Simpler to edit, but versioning is harder, no transactional updates
- **Hybrid:** Complexity without clear benefit

### Consequences

**Positive:**
- Single source of truth (database)
- Transactional template updates
- Easy version history
- Can query template contents

**Negative:**
- Large rows if templates grow (mitigated by TOAST)
- Need admin UI for editing (can't just edit JSON file)

**Mitigation:**
- PostgreSQL TOAST handles large JSONB efficiently
- Build visual template editor (Phase 5)
- Export/import functionality for version control

### Implementation Notes

**Schema:**
```sql
CREATE TABLE intake_templates (
    id UUID PRIMARY KEY,
    application_type_id TEXT NOT NULL,
    version INT NOT NULL,
    decision_tree JSONB NOT NULL,      -- Full tree here
    feature_mappings JSONB NOT NULL,   -- Feature → module mappings
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(application_type_id, version)
);
```

**Validation:**
```rust
pub fn validate_template(tree: &serde_json::Value) -> Result<(), ValidationError> {
    // 1. Check all question IDs are unique
    // 2. Check conditional logic references valid questions
    // 3. Check feature mappings reference valid modules
    // 4. Check no circular dependencies
    // 5. Check all paths lead to completion
    // ...
}
```

**Size limits:**
- Max 100 questions per template
- Max depth 5 for conditional logic
- Max 50 modules per feature mapping

**Export for version control:**
```bash
# Export template as JSON file
realmforge-cli intake export-template --id=03_dynamic_web_app --version=1 > template.json

# Import template from JSON file
realmforge-cli intake import-template --file=template.json
```

---

## Integration & Dependencies

### Crate Dependencies
```
intake-engine (no external deps beyond serde)
      ↓
control-service (orchestrates intake + AI provider)
      ↓
control-store (persists templates, sessions, questions)
```

### AI Provider Integration
```
control-service
  ├── intake_engine (decision tree logic)
  └── ai_provider (pluggable trait)
        ├── claude_provider (default)
        ├── openai_provider (future)
        └── mock_provider (testing)
```

### Database Schema
- `intake_templates` stores JSONB decision trees
- `question_frequency` has `pg_trgm` index for similarity
- Clear upgrade path to `pgvector` if needed

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Separate crate adds complexity | Medium | Clear interface contract, good documentation |
| AI provider API changes | High | Abstraction layer isolates changes |
| pg_trgm insufficient for similarity | Medium | Track metrics, documented upgrade path to embeddings |
| JSONB templates too large | Low | PostgreSQL TOAST, size limits, export functionality |
| Template editing UX poor | Medium | Build visual editor in Phase 5, export/import for power users |

---

## Validation Criteria

Before accepting this ADR:
1. **CTO (Rena)** approves crate boundaries and layer contracts
2. **API Architect (Marcus)** approves AI provider trait interface
3. **Data Architect (Chen)** approves JSONB storage and pg_trgm approach
4. **Security Architect (Fatima)** approves no new attack vectors introduced

---

## Timeline

- **Week 1:** Implement `intake-engine` crate with decision tree evaluator
- **Week 2:** Integrate with `control-service`, add database persistence
- **Weeks 3-4:** Add Claude provider implementation
- **Week 5:** Add question similarity matching (pg_trgm)
- **Weeks 8-9:** Build admin UI for template editing

---

## Appendix: Open Questions

These remain AFTER this ADR is approved:

**From INTAKE_SYSTEM_DESIGN.md Section 11.2:**
- **Q5:** Who can approve new forms? (Council decision required — see DEC-COUNCIL-PENDING)
- **Q6:** Auto-promotion threshold fixed or variable? (PM decision required — see DEC-PM-PENDING)
- **Q7:** AI-generated forms immediate use or approval-first? (Security decision required — see DEC-SECURITY-PENDING)

**From INTAKE_SYSTEM_DESIGN.md Section 11.3:**
- **Q8:** Migration strategy for existing projects
- **Q9:** Multi-language (i18n) support for forms
- **Q10:** Versioning strategy for breaking template changes

---

## References

- `docs/INTAKE_SYSTEM_DESIGN.md` — Full specification (1,468 lines)
- `docs/BACKEND_GAP_ANALYSIS.md` Section 3.6 — Gap analysis
- `docs/decisions/DEC-COUNCIL-PENDING-intake-form-approval-authority.md`
- `docs/decisions/DEC-PM-PENDING-auto-promotion-threshold.md`
- `docs/decisions/DEC-SECURITY-PENDING-draft-form-usage.md`

---

**Status:** PROPOSED  
**Next Steps:**
1. CTO (Rena) reviews and approves crate architecture
2. API Architect (Marcus) reviews AI provider trait
3. Data Architect (Chen) reviews JSONB + pg_trgm approach
4. Security Architect (Fatima) reviews for vulnerabilities
5. Council votes on approval
6. If approved → ADR-001 status becomes ACCEPTED
