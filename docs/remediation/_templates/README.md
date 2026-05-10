# RealmForge Remediation Templates

**Purpose:** Reusable templates and guides for conducting area reviews and creating remediation plans.

---

## Contents

### Core Templates

**1. AREA_CONTEXT_TEMPLATE.md** 📋 NEW
* Template for capturing area intent and purpose
* Use BEFORE generating audit questions (Session 1)
* Captures: Purpose, responsibilities, boundaries, languages, architecture, key concepts, success criteria
* **Critical:** No implementation details — intent only
* Usage: Copy to `/docs/remediation/[area]/AREA_CONTEXT.md` and populate

**2. AUDIT_QUESTIONS_TEMPLATE.md** 📋 NEW
* Structure for 210 tailored audit questions (10 dimensions)
* Use AFTER creating AREA_CONTEXT.md (Session 2)
* Questions test production readiness, not current implementation
* Format: Question ID, text, priority, evidence, gap, risk
* Usage: Generate tailored version at `/docs/remediation/[area]/AUDIT_QUESTIONS.md`

**3. QUESTION_GENERATION_GUIDE.md** 📘 NEW
* **COMPREHENSIVE GUIDE** for creating area-specific questions
* Step-by-step process for tailoring 210 questions from AREA_CONTEXT.md
* **Critical constraint:** Never read implementation code during generation
* Dimension-by-dimension tailoring guidance with examples
* Usage: Read before generating AUDIT_QUESTIONS.md

**4. FILE_NAMING_CONVENTIONS.md** 📋 NEW
* Standard file locations and names for all 10 areas
* Two-session workflow documentation
* Common mistakes to avoid
* Usage: Reference when setting up area folders

**5. REMEDIATION_PLAN_TEMPLATE.md**
* Master template for area remediation plans
* Includes: Work streams, work items, handoff/sign-off sections, appendices
* Usage: Copy to `/docs/remediation/[area]/REMEDIATION_PLAN.md` and populate

**6. REVIEW_PROCESS_GUIDE.md** 📘
* **COMPREHENSIVE GUIDE** — Read this before conducting any audit
* 6 phases: Pre-audit prep → Audit → Report → Gap analysis → Plan → Kickoff
* 25KB of detailed instructions
* Includes skill invocation workflows, 210-question framework, templates

**7. QUICK_REFERENCE.md** 🎯
* One-page cheat sheet
* Essential skill invocations, workflow diagram, quality gates
* Use for quick lookup during reviews

---

## The Two-Session Workflow (NEW)

### Why Two Sessions?

Questions must test **what should exist for production** (intent), not **what currently exists** (implementation). Separating context capture from question generation prevents bias.

### Session 1: Capture Intent (AREA_CONTEXT.md)
**Goal:** Document what the area is supposed to do

**Process:**
1. Read `AREA_CONTEXT_TEMPLATE.md`
2. Create `/docs/remediation/[area]/AREA_CONTEXT.md`
3. Fill in: Purpose, responsibilities, boundaries, architecture, success criteria
4. **Do NOT include:** Current implementation details, file names, code structure
5. Get CTO (Rena) sign-off

**Output:** Intent-based context document

---

### Session 2: Generate Questions (AUDIT_QUESTIONS.md)
**Goal:** Create 210 questions testing production readiness

**Process:**
1. Read `QUESTION_GENERATION_GUIDE.md` (comprehensive instructions)
2. Read `/docs/remediation/[area]/AREA_CONTEXT.md` (the area's intent)
3. Read `AUDIT_QUESTIONS_TEMPLATE.md` (structure reference)
4. **Do NOT read:** Implementation code, crate files, current source
5. Tailor all 210 questions to the area's context
6. Generate `/docs/remediation/[area]/AUDIT_QUESTIONS.md`
7. Get area owner confirmation

**Output:** 210 tailored, intent-based audit questions

---

## How to Use These Templates

### For Area Owners (Starting an Audit)

**Step 0: Two-Session Preparation (NEW)**
1. **Session 1:** Create `AREA_CONTEXT.md` using `AREA_CONTEXT_TEMPLATE.md`
   * Document area intent without implementation details
   * Get CTO review and sign-off
2. **Session 2:** Generate `AUDIT_QUESTIONS.md` using `QUESTION_GENERATION_GUIDE.md`
   * Read only AREA_CONTEXT.md and templates
   * Never read implementation code
   * Tailor all 210 questions to area context

**Step 1: Read the guides**
1. Read `QUICK_REFERENCE.md` first (10 minutes)
2. Read `REVIEW_PROCESS_GUIDE.md` fully (30-60 minutes)
3. Bookmark both for reference during audit

**Step 2: Prepare**
1. Invoke orchestrator for workflow state
2. Invoke CTO for architecture context
3. Gather existing documentation

**Step 3: Execute audit**
1. Load domain-audit skill
2. Invoke relevant skills (architects, engineers)
3. Answer all 210 questions from `/docs/remediation/[area]/AUDIT_QUESTIONS.md`
4. Document evidence for each question
5. Calculate coverage metrics

**Step 4: Create reports**
1. Create `AUDIT_REPORT.md` (use REVIEW_PROCESS_GUIDE.md Phase 3)
2. Create `GAP_ANALYSIS.md` (use REVIEW_PROCESS_GUIDE.md Phase 4)
3. Route for CTO review

**Step 5: Create remediation plan**
1. Copy `REMEDIATION_PLAN_TEMPLATE.md` to your area folder
2. Organize gaps into work streams
3. Break down into work items
4. Populate all sections (use REVIEW_PROCESS_GUIDE.md Phase 5)
5. Route for multi-skill validation and sign-off

**Step 6: Execute**
1. Invoke orchestrator for workflow initialization
2. Invoke PM for first work item assignment
3. Setup weekly checkpoints

---

### For Reviewers (Signing Off on Plans)

**Architecture Review (Rena, CTO):**
* Verify AREA_CONTEXT.md accurately reflects area intent
* Verify layer boundaries respected
* Validate work breakdown covers all architectural concerns
* Check acceptance criteria are testable
* Sign off on AREA_CONTEXT.md, AUDIT_REPORT.md, and REMEDIATION_PLAN.md

**Security Review (Fatima):**
* Verify threat model completeness
* Validate security work items
* Check audit trail requirements
* Sign off on security work streams

**Implementation Review (Dmitri, Backend):**
* Verify effort estimates realistic
* Validate technical approach
* Check for missing work items
* Sign off on implementation work streams

**Planning Review (Alex, PM):**
* Verify prioritization correct
* Validate timeline and resource allocation
* Check handoff chain complete
* Sign off on overall plan

---

## Template Matrix

| Template | Purpose | When to Use | Size |
|----------|---------|-------------|------|
| **AREA_CONTEXT_TEMPLATE.md** | Capture area intent | Session 1: Before audit | 5 KB |
| **AUDIT_QUESTIONS_TEMPLATE.md** | Question structure | Session 2: Question generation | 50 KB |
| **QUESTION_GENERATION_GUIDE.md** | How to generate questions | Session 2: Before generating | 15 KB |
| **FILE_NAMING_CONVENTIONS.md** | File locations and naming | Setup: Area folder creation | 10 KB |
| **REVIEW_PROCESS_GUIDE.md** | Complete workflow | Before starting any audit | 25 KB |
| **QUICK_REFERENCE.md** | One-page cheat sheet | During audit for quick lookup | 9 KB |
| **REMEDIATION_PLAN_TEMPLATE.md** | Work item structure | After gap analysis | 7 KB |

---

## Supporting Documents (Referenced in Guide)

**Template 1: Evidence Collection Checklist**
* Found in: REVIEW_PROCESS_GUIDE.md, Supporting Templates section
* Use during: Phase 2 (Audit Execution)

**Template 2: Critical Issue Template**
* Found in: REVIEW_PROCESS_GUIDE.md, Supporting Templates section
* Use during: Phase 3 (Audit Report Creation)

**Template 3: Skill Invocation Matrix**
* Found in: REVIEW_PROCESS_GUIDE.md, Supporting Templates section
* Also in: QUICK_REFERENCE.md
* Use during: All phases

---

## Quality Standards

Every remediation plan created from these templates must:

**Completeness:**
- [ ] All work items have: Priority, Context, Scope, Acceptance Criteria, Dependencies, Deliverables
- [ ] Every work item has HANDOFF and SIGN-OFF sections
- [ ] All appendices populated (Dependency Graph, Risk Register, Go/No-Go, Checkpoints, Escalation)

**Traceability:**
- [ ] Every work item traces back to a gap in GAP_ANALYSIS.md
- [ ] Every gap traces back to an audit question in AUDIT_QUESTIONS.md
- [ ] Every question traces back to AREA_CONTEXT.md intent
- [ ] Every critical issue has remediation work items

**Intent-Based Questions:**
- [ ] AREA_CONTEXT.md contains no implementation details
- [ ] AUDIT_QUESTIONS.md generated without reading implementation code
- [ ] Questions test production readiness, not implementation validation

**Feasibility:**
- [ ] Effort estimates validated by skill owners
- [ ] Dependencies mapped and realistic
- [ ] Parallelization opportunities identified
- [ ] Timeline includes 20-30% buffer

**Reviewability:**
- [ ] Acceptance criteria are specific and testable
- [ ] Handoff recipients named explicitly
- [ ] Sign-off checklists are actionable
- [ ] Next dependencies clear

---

## Example: RFSource (Reference Implementation)

The RFSource remediation plan is the **reference implementation** of this template system:

**Location:** `/docs/remediation/rfsource/REMEDIATION_PLAN.md`

**What makes it exemplary:**
* 96 work items across 8 work streams
* Every item has complete Context, Scope, Acceptance Criteria
* All handoffs and sign-offs explicitly defined
* Full appendices (dependency graph, risk register, go/no-go criteria)
* Estimated 13-18 weeks with realistic buffer

**Use as example when:**
* Creating your first remediation plan
* Unsure how detailed work items should be
* Need reference for appendix content
* Validating your own plan structure

**Next step:** Create AREA_CONTEXT.md and AUDIT_QUESTIONS.md for RFSource as reference implementations of the two-session workflow

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-XX | Initial creation with 3 templates |
| 2.0 | 2024-01-XX | Added 4 new templates: AREA_CONTEXT, AUDIT_QUESTIONS, QUESTION_GENERATION_GUIDE, FILE_NAMING_CONVENTIONS. Introduced two-session workflow for intent-based questioning. |

---

## References

* **Master Index:** `/docs/remediation/00_INDEX.md`
* **RFSource Example:** `/docs/remediation/rfsource/REMEDIATION_PLAN.md`
* **Domain Audit Skill:** `.assistant/skills/domain-audit/SKILL.md`
* **Orchestrator Skill:** `.assistant/skills/orchestrator/SKILL.md`
* **Council Skill:** `.assistant/skills/council/SKILL.md`

---

**Questions or feedback?** Discuss with Alex (PM) or Rena (CTO)
