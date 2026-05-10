# Question Generation Guide — Intent-Based Audit Questions

**Purpose:** Instructions for generating area-specific audit questions from AREA_CONTEXT.md

**Status:** 📋 GENERATION GUIDE

**Critical Constraint:** Questions MUST be based on intent (what should exist for production readiness), NOT on current implementation (what does exist).

---

## Overview

This guide explains how to generate 210 tailored audit questions for a RealmForge area based on its AREA_CONTEXT.md document. The questions test whether the area meets production readiness standards, not whether it matches current implementation.

---

## The Two-Session Workflow

### Session 1: Capture Intent (AREA_CONTEXT.md)
* **Goal:** Document what the area is **supposed** to do
* **Input:** Area owner knowledge, architecture docs, requirements
* **Output:** `/docs/remediation/[area]/AREA_CONTEXT.md`
* **Constraint:** No implementation details, no current state

### Session 2: Generate Questions (AUDIT_QUESTIONS.md)
* **Goal:** Create 210 questions testing production readiness against standards
* **Input:** AREA_CONTEXT.md + AUDIT_QUESTIONS_TEMPLATE.md + This guide
* **Output:** `/docs/remediation/[area]/AUDIT_QUESTIONS.md`
* **Constraint:** **Never read implementation code, crate files, or current source**

---

## Critical Constraint: Intent-Based Questions Only

### ✅ DO Read:
* `/docs/remediation/[area]/AREA_CONTEXT.md` (the area's intent)
* `/docs/remediation/_templates/AUDIT_QUESTIONS_TEMPLATE.md` (structure)
* `/docs/remediation/_templates/QUESTION_GENERATION_GUIDE.md` (this file)
* `/docs/remediation/_templates/REVIEW_PROCESS_GUIDE.md` (process context)

### ❌ DO NOT Read:
* Implementation code in `/crates/[area]/`
* Current Rust files (`.rs`)
* Existing test files
* Database migration files (unless explicitly referenced in AREA_CONTEXT.md as part of intent)
* Any artifact that describes **current state** rather than **intended purpose**

### Why This Constraint Matters

If you read implementation code while generating questions:
1. **Validation theater:** Questions become "does the code match itself?" rather than "does this meet production standards?"
2. **Blind spots:** You won't ask about missing functionality because you don't see it in the code
3. **Bias:** Questions get shaped around what exists rather than what should exist
4. **Low bar:** The audit becomes a sanity check rather than a production readiness test

**Example of the problem:**
* **Without constraint:** "Does the grant model support subject types?" (because you saw Subject enum in code)
* **With constraint:** "Does the authorization system support role-based, attribute-based, and time-based access control?" (because AREA_CONTEXT.md said it should)

The first question validates existing code. The second question tests completeness against requirements.

---

## Step-by-Step Process

### Step 0: Prerequisites

Before starting, ensure:
* [ ] AREA_CONTEXT.md exists and is complete
* [ ] AREA_CONTEXT.md has been reviewed by CTO (Rena)
* [ ] You have **not** read any implementation code for this area
* [ ] You have the template and this guide open

---

### Step 1: Read AREA_CONTEXT.md Thoroughly

**Time:** 15-30 minutes

**Read and extract:**
1. **Purpose:** What problem does this area solve?
2. **Responsibilities:** What specific capabilities should exist?
3. **Boundaries:** What is explicitly out of scope?
4. **Key Concepts:** What domain entities, types, or abstractions are central?
5. **Success Criteria:** How do you know it's working correctly?
6. **Dependencies:** What does this area depend on?
7. **Consumers:** Who uses this area?
8. **Production Readiness Considerations:** What's critical for production?
9. **Risk Profile:** What are the highest-risk failure scenarios?

**Create a mental model:**
* If this area were perfect and production-ready, what would it look like?
* What documentation should exist?
* What tests would prove it works?
* What security guarantees must it provide?
* How would you know it's scalable?

---

### Step 2: Load the Template

**File:** `/docs/remediation/_templates/AUDIT_QUESTIONS_TEMPLATE.md`

**Review structure:**
* 10 dimensions (D1-D10)
* 210 questions total
* Question format (ID, text, priority, evidence, gap, risk)

**Note the question distribution:**
* D1: Documentation (20 questions)
* D2: Testing (25 questions)
* D3: Scalability (20 questions)
* D4: Versioning (25 questions)
* D5: File Size & Modularity (20 questions)
* D6: Security (30 questions)
* D7: Error Handling (25 questions)
* D8: Performance (25 questions)
* D9: Operational (10 questions)
* D10: Standardization (10 questions)

---

### Step 3: Generate Questions Dimension by Dimension

For each dimension (D1-D10), follow this process:

#### 3.1: Understand the Dimension's Focus

**D1 (Documentation):** Does documentation exist to understand, maintain, and extend this area?

**D2 (Testing):** Are there tests proving this area works correctly under all conditions?

**D3 (Scalability):** Can this area handle production load and grow with demand?

**D4 (Versioning):** Can this area evolve without breaking consumers or losing data?

**D5 (File Size & Modularity):** Is code organized, cohesive, and maintainable?

**D6 (Security):** Is this area protected against threats and attacks?

**D7 (Error Handling):** Does this area handle failures gracefully and informatively?

**D8 (Performance):** Is this area fast, efficient, and resource-conscious?

**D9 (Operational):** Can operators deploy, monitor, and recover this area?

**D10 (Standardization):** Does this area follow team standards and conventions?

---

#### 3.2: Tailor Generic Questions to Area Context

**Generic question template (from AUDIT_QUESTIONS_TEMPLATE.md):**
```markdown
### [D1-Q001]: Does this area have a comprehensive README.md that explains its purpose?
**Priority:** 🔴 Critical
```

**Tailored question (using AREA_CONTEXT.md):**
```markdown
### [D1-Q001]: Does the Authority domain have a README.md explaining the grant model, policy evaluation, and scope hierarchy?
**Priority:** 🔴 Critical
```

**What changed:**
* "this area" → "the Authority domain" (specific area name)
* "explains its purpose" → "explaining the grant model, policy evaluation, and scope hierarchy" (specific concepts from AREA_CONTEXT.md)

---

#### 3.3: Add Area-Specific Questions

Some questions won't appear in the template but are critical for this specific area based on AREA_CONTEXT.md.

**Example (from Authority AREA_CONTEXT.md):**

If AREA_CONTEXT.md mentions:
> "Authorization decisions must be deterministic, transitive, and audit-traceable"

Then add a question:
```markdown
### [D7-Q026]: Are authorization decisions deterministic (same inputs always produce same outputs)?
**Priority:** 🔴 Critical
**Context:** AREA_CONTEXT.md states authorization must be deterministic for compliance.
```

---

#### 3.4: Adjust Priority Based on Area Risk Profile

AREA_CONTEXT.md includes a **Risk Profile** section. Use it to adjust priorities.

**Example:**
If AREA_CONTEXT.md says:
> **Privilege Escalation:** Incorrect grant evaluation could allow unauthorized actions (CRITICAL)

Then questions about authorization correctness should be **🔴 Critical**, not **🟠 Important**.

---

### Step 4: Review All 210 Questions for Consistency

After generating questions for all 10 dimensions:

**Check:**
* [ ] All 210 questions are present (count them)
* [ ] Question IDs are sequential (D1-Q001 through D10-Q010)
* [ ] Each question is tailored to this area (no generic "this area" language left)
* [ ] Priorities reflect the area's risk profile
* [ ] Questions reference key concepts from AREA_CONTEXT.md
* [ ] No question references implementation code or current state
* [ ] Each question tests production readiness, not implementation validation

---

### Step 5: Validate Against Area Intent

**For each dimension, ask:**
1. Do these questions test whether the area meets its **stated purpose** (from AREA_CONTEXT.md)?
2. Do these questions cover the area's **responsibilities**?
3. Do these questions respect the area's **boundaries** (not asking about out-of-scope concerns)?
4. Do these questions test the area's **success criteria**?
5. Do these questions address the area's **risk profile**?

If any answer is "no", revise the questions.

---

### Step 6: Write the Final AUDIT_QUESTIONS.md File

**Location:** `/docs/remediation/[area]/AUDIT_QUESTIONS.md`

**Structure:**
```markdown
# [AREA NAME] — Audit Questions

**Purpose:** Production readiness audit questions tailored to this area's intent and context.
**Source Context:** `/docs/remediation/[area]/AREA_CONTEXT.md`
**Total Questions:** 210 (D1-D10)

---

## How to Use This File
[Instructions for auditors]

---

## Dimension 1: Documentation (20 questions)

### [D1-Q001]: [Tailored question text]
**Priority:** 🔴 Critical | 🟠 Important | 🟢 Nice-to-Have
**Status:** [ ] Not Evaluated

**Evidence:**


**Gap (if any):**


**Risk:**


---

[Repeat for all 210 questions]

---

## Coverage Summary
[Calculation instructions]

---

## Next Steps
[What to do after completing the audit]
```

---

## Dimension-Specific Tailoring Guidance

### D1: Documentation

**Focus areas from AREA_CONTEXT.md:**
* What key concepts need documenting? (from section 6)
* What are the success criteria? (from section 7)
* What dependencies exist? (from section 8)
* What are the risk scenarios? (from section 11)

**Tailoring approach:**
* Replace "this area" with specific area name
* Replace "its purpose" with specific purpose from AREA_CONTEXT.md
* Add questions about documenting key concepts
* Add questions about documenting risk mitigation

---

### D2: Testing

**Focus areas from AREA_CONTEXT.md:**
* What are the core responsibilities? (from section 2)
* What failure modes exist? (from section 10)
* What are the security guarantees? (from section 7 or 10)
* What are the risk scenarios? (from section 11)

**Tailoring approach:**
* Add tests for each specific responsibility
* Add failure injection tests for documented failure modes
* Add security tests for threats mentioned in risk profile
* Add concurrency tests if area has shared state (check section 10)

---

### D3: Scalability

**Focus areas from AREA_CONTEXT.md:**
* What are expected load patterns? (from section 10)
* What are performance characteristics? (from section 7)
* What are resource requirements? (from section 10)

**Tailoring approach:**
* Use specific numbers from section 10 (e.g., "10K+ grants", "<1ms evaluation")
* Ask about horizontal scalability if section 10 mentions it
* Ask about caching if area serves read-heavy workloads

---

### D4: Versioning

**Focus areas from AREA_CONTEXT.md:**
* Does area have APIs? (from section 2 or 9)
* Does area have data schemas? (from section 5 or 10)
* Does area have public contracts? (from section 2)

**Tailoring approach:**
* If area has APIs, focus on API versioning
* If area has schemas, focus on migration testing
* If area is internal-only, de-prioritize some versioning questions

---

### D5: File Size & Modularity

**Focus areas from AREA_CONTEXT.md:**
* What are the key components? (from section 5)
* What layer does area belong to? (from section 5)
* What are the boundaries? (from section 3)

**Tailoring approach:**
* Ask about separation between documented components
* Ask about layer boundary enforcement
* Ask about avoiding dependencies on out-of-scope concerns (from section 3)

---

### D6: Security

**Focus areas from AREA_CONTEXT.md:**
* What are the risk scenarios? (from section 11) **← CRITICAL**
* What security guarantees are required? (from section 7 or 10)
* Does area handle sensitive data? (from section 2)
* Does area perform authorization? (from section 2)

**Tailoring approach:**
* Elevate priority (to 🔴 Critical) for questions related to section 11 risks
* Add questions about specific threats mentioned in risk profile
* Add questions about attack surface based on section 2 responsibilities
* If area doesn't handle auth or sensitive data, de-prioritize some questions

---

### D7: Error Handling

**Focus areas from AREA_CONTEXT.md:**
* What are failure modes? (from section 10)
* How should area behave under failure? (from section 7)
* What are dependencies that might fail? (from section 8)

**Tailoring approach:**
* Ask about graceful degradation for each dependency
* Ask about error types specific to area's responsibilities
* Ask about fail-closed vs. fail-open based on risk profile

---

### D8: Performance

**Focus areas from AREA_CONTEXT.md:**
* What are performance characteristics? (from section 7)
* What are scalability targets? (from section 10)
* What are resource requirements? (from section 10)

**Tailoring approach:**
* Use specific numbers from section 7 or 10 (e.g., "<1ms p99 latency")
* Ask about benchmarks for critical operations from section 2
* Ask about profiling hot paths based on area's workload

---

### D9: Operational

**Focus areas from AREA_CONTEXT.md:**
* What must be observable? (from section 10)
* What are failure modes? (from section 10)
* Does area manage state? (from section 2 or 5)

**Tailoring approach:**
* Ask about monitoring specific metrics from section 10
* Ask about runbooks for failure modes from section 10
* If area manages state, focus on backup/restore/disaster recovery

---

### D10: Standardization

**Focus areas from AREA_CONTEXT.md:**
* What programming languages? (from section 4)
* What layer does area belong to? (from section 5)

**Tailoring approach:**
* Ask about language-specific standards (Rust: no unwrap without comment, etc.)
* Ask about layer boundary enforcement from section 5
* Ask about architecture compliance with documented layer

---

## Example: Tailoring a Question

### Generic Question (from template):
```markdown
### [D2-Q005]: Are there tests for concurrent access and race conditions (if applicable)?
**Priority:** 🔴 Critical (if area has shared state)
**Status:** [ ] Not Evaluated
```

### AREA_CONTEXT.md Extract (Authority domain):
```markdown
## 10. Production Readiness Considerations

**Data Integrity:** Grant state must never allow unauthorized access due to race conditions

**Scalability:** Support 10K+ grants with <1ms evaluation time; horizontally scalable via read replicas
```

### Tailored Question:
```markdown
### [D2-Q005]: Are there concurrency tests validating that grant state remains consistent under concurrent modifications (e.g., simultaneous grant creation and revocation)?
**Priority:** 🔴 Critical
**Context:** AREA_CONTEXT.md states "Grant state must never allow unauthorized access due to race conditions"
**Status:** [ ] Not Evaluated

**Evidence:**
[Test file name, test coverage report, CI results showing concurrency test suite]

**Gap (if any):**
[If no concurrency tests exist, describe the risk of race conditions in grant management]

**Risk:**
If grant state has race conditions, the system could grant unauthorized access during concurrent operations, violating security guarantees. This is a CRITICAL security risk.
```

**What changed:**
1. "concurrent access" → "grant state remains consistent under concurrent modifications"
2. "if applicable" removed, replaced with explicit Authority domain context
3. Added specific example: "simultaneous grant creation and revocation"
4. Added context reference to AREA_CONTEXT.md
5. Risk section made explicit and severe (security impact)

---

## Quality Checklist

Before considering question generation complete:

* [ ] Read AREA_CONTEXT.md fully
* [ ] Generated all 210 questions
* [ ] Tailored every question with area-specific language
* [ ] Adjusted priorities based on risk profile
* [ ] Added area-specific questions where template was insufficient
* [ ] Removed or de-prioritized questions not applicable to this area
* [ ] Validated questions against area's stated purpose
* [ ] Validated questions against area's responsibilities
* [ ] Validated questions against area's success criteria
* [ ] **Confirmed that no implementation code was read during generation**
* [ ] Reviewed consistency across all dimensions
* [ ] Saved to correct location: `/docs/remediation/[area]/AUDIT_QUESTIONS.md`

---

## Common Mistakes and How to Avoid Them

### Mistake 1: Reading Implementation Code

**Symptom:** Questions like "Does the `GrantRegistry` struct have a `grants` field?" (implementation detail)

**Fix:** Ask "Does the area maintain an in-memory collection of active grants with fast lookup?" (intent-based)

---

### Mistake 2: Leaving Generic Language

**Symptom:** Questions still say "this area" or "if applicable"

**Fix:** Replace with specific area name and definitive statements based on AREA_CONTEXT.md

---

### Mistake 3: Not Adjusting Priorities

**Symptom:** All questions have template priorities, even though AREA_CONTEXT.md highlights critical risks

**Fix:** Review risk profile (section 11) and elevate priorities for questions addressing those risks

---

### Mistake 4: Missing Area-Specific Concerns

**Symptom:** AREA_CONTEXT.md mentions "tamper-evident audit log" but no questions ask about tamper-evidence

**Fix:** Add dimension-specific questions for unique requirements from AREA_CONTEXT.md

---

### Mistake 5: Asking About Out-of-Scope Concerns

**Symptom:** Questions ask about authentication when AREA_CONTEXT.md section 3 says "Does NOT handle authentication"

**Fix:** Review boundaries (section 3) and remove questions about explicitly out-of-scope concerns

---

## Sign-Off

After generating AUDIT_QUESTIONS.md:

**Area Owner Review:**
* [ ] Questions are appropriate for this area
* [ ] Questions test production readiness, not just implementation validation
* [ ] No critical concerns are missing

**Signature:** _____________________ Date: __________

---

**END OF QUESTION GENERATION GUIDE**
