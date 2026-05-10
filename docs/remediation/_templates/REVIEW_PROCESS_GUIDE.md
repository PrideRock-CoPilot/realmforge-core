# RealmForge Area Review & Audit Process Guide

**Purpose:** Step-by-step instructions for conducting comprehensive reviews of RealmForge areas, from initial audit through remediation planning.

**Target Audience:** Area owners, architects, PM, CTO

**Status:** 📘 REFERENCE GUIDE

---

## Overview

Every RealmForge area must undergo systematic review before claiming production readiness. This guide provides the complete workflow from audit through remediation planning.

## The Review Framework

**Foundation:** Domain Audit Framework (210 questions across 10 dimensions)
* **Location:** `.assistant/skills/domain-audit/SKILL.md`
* **Dimensions:** D1-D10 (Documentation → Operational Readiness)
* **Coverage:** ~107 critical questions, 102 important, 1 nice-to-have

**Skill Orchestration:** Multi-skill workflow coordinated by Orchestrator
* Architecture review (Rena, CTO)
* Security review (Fatima, Security Architect)
* Domain review (Yusuf, Domain Architect)
* Data review (Chen, Data Architect)
* Implementation review (Dmitri, Backend)

---

## Phase 1: Pre-Audit Preparation (1-2 days)

### Step 1.1: Invoke Orchestrator for Workflow State

**Why:** Ensure no conflicting work is in-flight for this area

**How:**
```
User: "hi orchestrator, I'm starting an audit of [AREA NAME]. 
What's the current workflow state for this area?"
```

**Expected output:**
* Any in-flight work items for the area
* Stalled handoffs that might affect the audit
* Dependencies on other areas

**Handoff:**
* **From:** PM (Alex) or CTO (Rena) — audit request
* **To:** Orchestrator — workflow state check
* **Deliverable:** Current state summary

---

### Step 1.2: Invoke CTO for Architecture Context

**Why:** Understand architectural boundaries and design decisions before auditing

**How:**
```
User: "hi Rena, I'm about to audit [AREA NAME]. Can you provide:
1. What this area owns (responsibilities)
2. What it doesn't own (boundaries)
3. Key architectural decisions (existing ADRs)
4. Known technical debt or gaps"
```

**Expected output:**
* Area scope definition
* Layer boundaries and contracts
* Relevant ADR references
* Known issues flagged for investigation

**Handoff:**
* **From:** Orchestrator (after workflow check)
* **To:** Rena (CTO) — architecture context
* **Deliverable:** Architecture context document (markdown)

---

### Step 1.3: Gather Existing Documentation

**Manual step:** Collect all existing docs for the area

**Locations to check:**
* `/docs/spec/` — Specification documents
* `/docs/architecture/` — Architecture docs
* `/docs/decisions/` — ADRs
* `/docs/api/` — API contracts (if applicable)
* Crate-level `README.md` files
* Inline rustdoc comments

**Create inventory:**
```
Area: [AREA NAME]
Existing documentation:
- [ ] Specification in /docs/spec/
- [ ] Architecture doc in /docs/architecture/
- [ ] ADRs in /docs/decisions/
- [ ] API contracts in /docs/api/
- [ ] Crate README.md files
- [ ] Inline rustdoc (>= 80% coverage?)
```

**Handoff:**
* **From:** Auditor (self)
* **To:** Next phase (audit execution)
* **Deliverable:** Documentation inventory checklist

---

## Phase 2: Audit Execution (1-2 weeks)

### Step 2.1: Load Domain Audit Skill

**Why:** Access 210-question audit framework

**How:**
```
User: "Load the domain-audit skill for [AREA NAME]"
```

**What happens:**
* Skill file read: `.assistant/skills/domain-audit/SKILL.md`
* 210 questions loaded across 10 dimensions
* Audit protocol activated

**Note:** Domain audit skill should be loaded FIRST before any audit work begins

---

### Step 2.2: Invoke Relevant Skills for Evidence Collection

**Architecture-focused areas** (Authority, Control Plane, Policy):
```
User: "hi Yusuf, I'm auditing [AREA NAME] domain model. 
Can you review:
1. Domain entity types and invariants
2. State machine correctness
3. Bounded context boundaries
4. Domain purity (no I/O leakage)"
```

**Security-focused areas** (Policy, Audit, Governance):
```
User: "hi Fatima, I'm auditing [AREA NAME] security posture.
Can you provide:
1. Threat model (STRIDE analysis)
2. Attack surface assessment
3. Bypass risks
4. Audit trail completeness"
```

**Data-focused areas** (Catalog, Snapshot, RFSource):
```
User: "hi Chen, I'm auditing [AREA NAME] data architecture.
Can you review:
1. Schema design and normalization
2. Index strategy and performance
3. Persistence layer separation
4. Data integrity guarantees"
```

**Infrastructure-focused areas** (Infrastructure, Deployment):
```
User: "hi Nadia, I'm auditing [AREA NAME] operational readiness.
Can you review:
1. PostgreSQL schema and migrations
2. Observability (metrics, logs, traces)
3. Runbook completeness
4. Disaster recovery procedures"
```

**Implementation-focused areas** (All Rust crates):
```
User: "hi Dmitri, I'm auditing [AREA NAME] implementation quality.
Can you review:
1. Code structure and organization
2. Error handling completeness
3. Test coverage (unit, integration)
4. Performance characteristics"
```

**Handoff pattern:**
* **From:** Auditor + Domain Audit Skill
* **To:** Relevant skill owner (Yusuf, Fatima, Chen, Nadia, Dmitri)
* **Deliverable:** Evidence collection notes (markdown)

---

### Step 2.3: Code Review (If Applicable)

**For areas with existing implementation:**

**Invoke Code Review (Owen):**
```
User: "hi Owen, I need a comprehensive code review of [AREA NAME].
Crates to review: [list crates]
Focus areas:
1. Architecture compliance (layer boundaries)
2. Rust idioms and safety
3. Error handling patterns
4. Test coverage gaps"
```

**Expected output:**
* Per-crate code review document
* Critical issues flagged
* Technical debt inventory
* Compliance with architecture rules

**Handoff:**
* **From:** Audit team
* **To:** Owen (Code Review)
* **Deliverable:** Code review reports (one per crate)

---

### Step 2.4: Answer All 210 Audit Questions

**Process:**
Using the domain audit skill framework, systematically answer each question:

**D1: Documentation (20 questions)**
* Inline docs, README, architecture guides, API contracts
* Evidence: Documentation files, rustdoc coverage reports

**D2: Testing (25 questions)**
* Unit, integration, property-based, concurrency, failure injection
* Evidence: Test files, coverage reports, CI results

**D3: Scalability (20 questions)**
* Load testing, performance benchmarks, resource limits
* Evidence: Benchmark results, load test reports

**D4: Versioning (25 questions)**
* Schema migrations, API versioning, backward compatibility
* Evidence: Migration files, versioning strategy docs

**D5: File Size & Modularity (20 questions)**
* File size limits, module boundaries, code organization
* Evidence: File line counts, module structure

**D6: Security (30 questions)**
* Threat models, access control, audit trails, encryption
* Evidence: Threat model docs, security test results

**D7: Error Handling (25 questions)**
* Error types, recovery procedures, graceful degradation
* Evidence: Error handling code, recovery tests

**D8: Performance (25 questions)**
* Benchmarks, profiling, optimization, resource usage
* Evidence: Benchmark reports, profiling results

**D9: Operational (10 questions)**
* Monitoring, runbooks, deployment, disaster recovery
* Evidence: Runbooks, monitoring configs, deployment docs

**D10: Standardization (10 questions)**
* Coding standards, naming conventions, architecture compliance
* Evidence: Code reviews, architecture compliance checks

**Documentation format:**
```
## Dimension [N]: [Name]

### Question [N]: [Question text]
**Status:** ✅ Answered | ⚠️ Partial | ❌ Gap
**Priority:** 🔴 Critical | 🟠 Important | 🟢 Nice-to-Have

**Evidence:**
[Location of evidence, summary of findings]

**Gap (if any):**
[What's missing, why it matters]

**Risk:**
[Impact if not addressed]
```

**Handoff:**
* **From:** Auditor + Domain Audit Skill
* **To:** Self (compilation phase)
* **Deliverable:** Complete 210-question evidence document

---

### Step 2.5: Calculate Coverage Metrics

**Manual calculation:**

```python
total_questions = 210
answered = [count ✅ Answered]
partial = [count ⚠️ Partial]
gaps = [count ❌ Gap]

coverage_percentage = ((answered + (partial * 0.5)) / total_questions) * 100

critical_questions = 107
critical_answered = [count critical ✅]
critical_partial = [count critical ⚠️]
critical_gaps = [count critical ❌]

critical_coverage = ((critical_answered + (critical_partial * 0.5)) / critical_questions) * 100
```

**Coverage targets:**
* **MVP readiness:** ≥ 70% overall, ≥ 60% critical
* **Production readiness:** ≥ 90% overall, ≥ 85% critical
* **Excellence:** 100% overall, 100% critical

---

## Phase 3: Audit Report Creation (2-3 days)

### Step 3.1: Invoke Tech Writer for Report Structure

**Why:** Clara (Tech Writer) ensures consistent report format

**How:**
```
User: "hi Clara, I need to create an audit report for [AREA NAME].
I have 210 questions answered with [X]% coverage.
Can you help structure the report?"
```

**Expected output:**
* Report template
* Section structure recommendations
* Writing style guidance

**Handoff:**
* **From:** Auditor (with raw evidence)
* **To:** Clara (Tech Writer)
* **Deliverable:** Report structure template

---

### Step 3.2: Create AUDIT_REPORT.md

**File location:** `/docs/remediation/[area]/AUDIT_REPORT.md`

**Structure:**
```markdown
# [AREA NAME] Audit Report

## Executive Summary
- Total questions: 210
- Evidence coverage: [X]%
- Critical coverage: [Y]%
- Critical issues: [N]
- Production readiness: [Status]

## Audit Methodology
[How audit was conducted, which skills involved]

## Findings by Dimension

### D1: Documentation ([X]% coverage)
[Summary of findings]
**Critical gaps:** [List]

### D2: Testing ([X]% coverage)
[Summary of findings]
**Critical gaps:** [List]

[Repeat for all 10 dimensions]

## Critical Issues Summary

### CRIT-001: [Title]
**Severity:** BLOCKER | HIGH | MEDIUM | LOW
**Dimension:** [D1-D10]
**Impact:** [Description]
**Evidence:** [Location]
**Risk:** [What happens if not fixed]

[Repeat for all critical issues]

## Appendices
### Appendix A: Full Question List (210 questions)
[Complete list with status]

### Appendix B: Evidence Locations
[Index of all evidence files/locations]

### Appendix C: Reviewer Sign-Offs
- CTO Review: [Rena signature/date]
- [Domain] Architect Review: [Name signature/date]
- Security Review: [Fatima signature/date]
```

**Handoff:**
* **From:** Auditor + Clara (Tech Writer)
* **To:** Orchestrator (for review routing)
* **Deliverable:** AUDIT_REPORT.md

---

### Step 3.3: Route for Architecture Review

**Invoke Orchestrator:**
```
User: "orchestrator, I've completed the audit report for [AREA NAME].
Ready for architecture review. What's the handoff?"
```

**Invoke CTO:**
```
User: "hi Rena, the audit report for [AREA NAME] is complete.
Location: /docs/remediation/[area]/AUDIT_REPORT.md
Ready for your architecture review."
```

**Expected output:**
* Architecture compliance assessment
* Critical issues validated
* Remediation priorities suggested

**Handoff:**
* **From:** Auditor (via Orchestrator)
* **To:** Rena (CTO)
* **Deliverable:** Architecture review sign-off

---

## Phase 4: Gap Analysis (3-5 days)

### Step 4.1: Extract All Gaps from Audit

**Process:**
Review all 210 questions and extract every ❌ Gap and ⚠️ Partial answer.

**For each gap, document:**
```
### Gap ID: GAP-[AREA]-[DIM]-[NUM]
**Question:** [Original audit question]
**Current state:** [What exists now]
**Desired state:** [What's needed for production]
**Priority:** 🔴 BLOCKER | 🟠 HIGH | 🟡 MEDIUM | 🟢 LOW
**Estimated effort:** [N] days/weeks
**Dependencies:** [Other gaps or areas]
**Owner:** [Skill who should fix this]
**Risk if not addressed:** [Impact description]
```

---

### Step 4.2: Invoke PM for Prioritization

**Why:** Alex (PM) helps prioritize gaps by business impact and dependencies

**How:**
```
User: "hi Alex, I have [N] gaps identified for [AREA NAME].
Need help prioritizing. Here's the gap list: [summary]"
```

**Expected output:**
* Gaps sorted by priority (blocker → nice-to-have)
* MVP vs. full production categorization
* Dependency order validated

**Handoff:**
* **From:** Auditor
* **To:** Alex (PM)
* **Deliverable:** Prioritized gap list

---

### Step 4.3: Create GAP_ANALYSIS.md

**File location:** `/docs/remediation/[area]/GAP_ANALYSIS.md`

**Structure:**
```markdown
# [AREA NAME] Gap Analysis

## Summary

**Total gaps:** [N]
**By priority:**
- 🔴 Blocker: [N]
- 🟠 High: [N]
- 🟡 Medium: [N]
- 🟢 Low: [N]

**By dimension:**
- D1 (Documentation): [N] gaps
- D2 (Testing): [N] gaps
- [etc.]

**Estimated remediation effort:**
- Sequential: [N-M] weeks
- Parallel: [N-M] weeks

## Gap Inventory

### Blocker Gaps (Must fix for MVP)

#### GAP-[AREA]-D1-001: [Title]
[Full gap details as documented in 4.1]

[Repeat for all blockers]

### High Priority Gaps

[Repeat pattern]

### Medium Priority Gaps

[Repeat pattern]

### Low Priority Gaps

[Repeat pattern]

## Risk Assessment

**If all gaps remain unaddressed:**
- Data loss risk: [HIGH/MEDIUM/LOW]
- Security breach risk: [HIGH/MEDIUM/LOW]
- Operational failure risk: [HIGH/MEDIUM/LOW]
- Scalability collapse risk: [HIGH/MEDIUM/LOW]

**If only blockers addressed (MVP):**
- Acceptable for: [Use case description]
- Not acceptable for: [Use case description]
- Residual risks: [List]

## Appendices

### Appendix A: Gap-to-Question Mapping
[Each gap traced back to audit question]

### Appendix B: Dependency Graph
[Visual or text representation of gap dependencies]
```

**Handoff:**
* **From:** Auditor + Alex (PM)
* **To:** Orchestrator (for sign-off routing)
* **Deliverable:** GAP_ANALYSIS.md

---

## Phase 5: Remediation Plan Creation (1-2 weeks)

### Step 5.1: Organize Gaps into Work Streams

**Process:**
Group gaps by skill ownership and logical work areas.

**Example work streams:**
* **WS-01: Foundation** (Architecture, core types)
* **WS-02: Security** (Threat mitigation, audit trail)
* **WS-03: Data Layer** (Schema, indexes, persistence)
* **WS-04: Service Logic** (Business rules, orchestration)
* **WS-05: Testing** (Test coverage, failure injection)
* **WS-06: Documentation** (Docs, ADRs, guides)
* **WS-07: Operations** (Monitoring, runbooks, deployment)
* **WS-08: Performance** (Optimization, benchmarking)

**Criteria for work stream creation:**
* Single skill ownership (or small skill group)
* Logical coherence (related work)
* Parallelizable where possible
* Clear dependencies

---

### Step 5.2: Break Work Streams into Work Items

**For each work stream:**
1. Extract gaps assigned to this stream
2. Create work items (atomic, testable units)
3. Sequence by dependencies
4. Estimate effort
5. Define acceptance criteria
6. Create handoff/sign-off templates

**Work item template:**
```markdown
## WS-[N]-[NNN]: [Work Item Title]

**Priority:** 🔴 BLOCKER | 🟠 HIGH | 🟡 MEDIUM | 🟢 LOW
**Estimated Effort:** [N] days/weeks
**Status:** [ ]

**Context:**
[Why this work is necessary]

**Scope:**
[What will be done]
1. [Specific task]
2. [Specific task]

**Acceptance Criteria:**
- [ ] [Testable criterion]
- [ ] [Testable criterion]

**Dependencies:**
- **BLOCKS ON:** [Work item ID]

**Deliverable:**
- [ ] [Artifact location]

**HANDOFF:**
```
To: [Reviewer]
Deliverable Location: [Path]
Summary: [Owner fills in]
Date Handed Off: [YYYY-MM-DD]
```

**SIGN-OFF:**
```
Reviewer: [Name]
Review Date: [YYYY-MM-DD]
Checklist:
  [ ] [Criterion]
Sign-Off: [Signature/date]
```

**Next Dependency:**
- Unblocks: [Work item ID]
```

---

### Step 5.3: Calculate Critical Path and Timeline

**Process:**
1. Map all dependencies between work items
2. Identify longest dependency chain (critical path)
3. Calculate sequential timeline (one engineer)
4. Calculate parallel timeline (multiple engineers)
5. Identify parallelization opportunities

**Timeline estimation:**
* **Blocker items:** Sum effort, add 20% buffer
* **High priority:** Can be parallelized if independent
* **Medium/Low:** Can be deferred past MVP

---

### Step 5.4: Invoke Multiple Skills for Work Stream Validation

**For each work stream, invoke the skill owner:**

**Example:**
```
User: "hi Dmitri, I've created a remediation plan for [AREA NAME].
Work Stream 01 (Foundation) has [N] work items assigned to you.
Can you review:
1. Is the work breakdown correct?
2. Are effort estimates realistic?
3. Are dependencies accurate?
4. Any missing work?"
```

**Repeat for:**
* Fatima (Security work streams)
* Chen (Data work streams)
* Nadia (Operations work streams)
* Clara (Documentation work streams)

**Expected output:**
* Work breakdown validation
* Effort estimate adjustments
* Missing work items identified
* Dependency corrections

---

### Step 5.5: Create REMEDIATION_PLAN.md

**Use template:** `/docs/remediation/_templates/REMEDIATION_PLAN_TEMPLATE.md`

**Process:**
1. Copy template to `/docs/remediation/[area]/REMEDIATION_PLAN.md`
2. Replace all `[PLACEHOLDERS]` with actual content
3. Include all work streams and work items
4. Add appendices (dependency graph, risk register, go/no-go criteria)

**Key sections to populate:**
* Executive Summary (from audit + gap analysis)
* Work Stream Organization (from work breakdown)
* All work items (from work stream creation)
* Dependency graph (from critical path analysis)
* Risk register (from gap analysis + skill feedback)
* Go/no-go criteria (from CTO + PM)

---

### Step 5.6: Route for Final Review and Sign-Off

**Invoke Orchestrator:**
```
User: "orchestrator, remediation plan for [AREA NAME] is ready for final review.
Who needs to sign off?"
```

**Typical sign-off chain:**
1. **CTO (Rena)** — Architecture and technical feasibility
2. **PM (Alex)** — Timeline, resource allocation, tracking structure
3. **CEO (Victor)** — Strategic approval, investment commitment (if high effort)
4. **Area Owner** — Commitment to execute

**Invoke each for sign-off:**
```
User: "hi Rena, remediation plan for [AREA NAME] ready for CTO sign-off.
Location: /docs/remediation/[area]/REMEDIATION_PLAN.md"
```

**Expected output:**
* Sign-off or feedback for revision
* Any strategic adjustments to scope/timeline

---

## Phase 6: Execution Kickoff (1 day)

### Step 6.1: Invoke Orchestrator for Workflow Initialization

**Why:** Register all work items in workflow tracking

**How:**
```
User: "orchestrator, [AREA NAME] remediation plan approved and ready to execute.
Initialize workflow tracking for all [N] work items."
```

**Expected output:**
* All work items registered
* Dependency chains visible
* First work item identified for kickoff

---

### Step 6.2: Assign First Work Item

**Invoke PM:**
```
User: "hi Alex, [AREA NAME] remediation begins now.
First work item: WS-01-001: [Title]
Assigned to: [Skill Name]
Ready for kickoff?"
```

**Invoke assigned skill:**
```
User: "hi [Skill Name], you're assigned to WS-01-001: [Title].
Start when ready. Handoff to orchestrator when complete."
```

---

### Step 6.3: Setup Weekly Checkpoints

**Process:**
1. Create checkpoint schedule (every Monday)
2. Add to PM calendar
3. Notify all work stream owners

**Checkpoint format:** Use Appendix D in REMEDIATION_PLAN.md

---

## Supporting Templates

### Template 1: Evidence Collection Checklist

**Use during audit Phase 2.2**

```markdown
# Evidence Collection Checklist — [AREA NAME]

## D1: Documentation
- [ ] README.md exists and complete
- [ ] Architecture doc exists
- [ ] API contracts documented
- [ ] Rustdoc coverage ≥ 80%
- [ ] ADRs for major decisions

## D2: Testing
- [ ] Unit tests exist
- [ ] Integration tests exist
- [ ] Test coverage ≥ 80%
- [ ] Property-based tests (if applicable)
- [ ] Concurrency tests (if applicable)
- [ ] Failure injection tests (if applicable)

[Repeat for all 10 dimensions]
```

---

### Template 2: Critical Issue Template

**Use when documenting critical issues in audit**

```markdown
### CRIT-[NNN]: [Short Title]

**Severity:** 🔴 BLOCKER | 🟠 HIGH | 🟡 MEDIUM | 🟢 LOW
**Dimension:** [D1-D10]
**Discovered by:** [Skill name]
**Date identified:** [YYYY-MM-DD]

**Description:**
[What the issue is, why it's critical]

**Impact:**
[What happens if not fixed]

**Evidence:**
[Where the issue was observed]

**Root cause:**
[Why this happened]

**Affected components:**
[List crates/modules/files]

**Remediation:**
[High-level fix strategy]

**Estimated effort:**
[N days/weeks]

**Priority justification:**
[Why this severity level]

**Dependencies:**
[What must be fixed first or in parallel]
```

---

### Template 3: Skill Invocation Matrix

**Quick reference for which skill to invoke**

| Task | Invoke | Expected Output |
|------|--------|----------------|
| Workflow state check | `/orchestrator` | In-flight work, stalls, dependencies |
| Architecture context | `/cto` (Rena) | Area boundaries, ADRs, known gaps |
| Domain model review | `/domain-architect` (Yusuf) | Entity types, state machines, invariants |
| Security review | `/security-architect` (Fatima) | Threat model, attack surface, audit trail |
| API contract review | `/api-architect` (Marcus) | API design, versioning, error standards |
| Infrastructure review | `/infra-architect` (Nadia) | PostgreSQL, observability, runbooks |
| Data architecture review | `/data-architect` (Chen) | Schema, indexes, persistence layer |
| Code review | `/code-review` (Owen) | Implementation quality, compliance |
| Rust implementation review | `/backend` (Dmitri) | Rust idioms, error handling, tests |
| Gap prioritization | `/pm` (Alex) | Priority order, MVP scope, timeline |
| Report writing | `/tech-writer` (Clara) | Report structure, writing style |
| Strategic approval | `/ceo` (Victor) | Investment decision, scope approval |
| Decision making | `/council` | Multi-stakeholder structured decision |

---

## Appendix A: Timeline Estimates by Area

**RFSource (Storage Layer):** 13-18 weeks
* 9 crates, 10 critical issues
* Complete audit done (example to follow)

**Authority (Domain Layer):** 4-6 weeks (estimated)
* Pure domain logic, well-bounded
* Expect fewer operational gaps

**Policy (Security Layer):** 5-6 weeks (estimated)
* Threat modeling intensive
* Enforcement and audit trail work

**Catalog (Metadata Layer):** 5-6 weeks (estimated)
* Schema design, PostgreSQL backend
* Query optimization

**Audit (Compliance Layer):** 3-4 weeks (estimated)
* Tamper-evident logging
* Query and reporting APIs

**Snapshot (Event Sourcing):** 4-5 weeks (estimated)
* Event schema, replay logic
* Compaction strategy

**Control Plane (Orchestration):** 6-8 weeks (estimated)
* Service layer coordination
* Transaction boundaries

**API/MCP/CLI (Interfaces):** 4-5 weeks (estimated)
* Thin transport layers
* Error standards, versioning

**Frontend (UI):** 8-10 weeks (estimated)
* Component library
* Accessibility compliance

**Infrastructure (Operations):** 4-5 weeks (estimated)
* Observability stack
* Deployment automation

---

## Appendix B: Quality Gate Definitions

**Gate 1: Specification Complete**
- [ ] Requirements documented
- [ ] Architecture design approved (Rena)
- [ ] Security reviewed (Fatima)
- [ ] API contracts defined (Marcus)
- [ ] ADRs written (Clara)

**Gate 2: Implementation Complete**
- [ ] Code complete per spec
- [ ] Unit tests ≥ 80% coverage
- [ ] Integration tests passing
- [ ] Code review passed (Owen)
- [ ] Peer review passed (Nora)

**Gate 3: QA Certified**
- [ ] All acceptance tests passing
- [ ] Performance benchmarks met
- [ ] Security tests passed
- [ ] Accessibility tests passed (frontend only)
- [ ] QA sign-off (Meg)

**Gate 4: Production Ready**
- [ ] Deployment runbook complete
- [ ] Monitoring configured
- [ ] Disaster recovery tested
- [ ] Documentation complete
- [ ] Release Manager sign-off (Sam)

---

## Appendix C: Common Pitfalls and How to Avoid Them

**Pitfall 1: Skipping skill invocation**
* **Symptom:** Attempting to do architecture review without Rena, security review without Fatima
* **Fix:** Always invoke the skill owner for their domain
* **Prevention:** Use the skill invocation matrix (Template 3)

**Pitfall 2: Incomplete evidence collection**
* **Symptom:** Audit questions marked "answered" without evidence location
* **Fix:** Every answer must reference specific files, test results, or documents
* **Prevention:** Use evidence collection checklist (Template 1)

**Pitfall 3: Underestimating remediation effort**
* **Symptom:** Timeline estimates that don't account for dependencies, testing, review cycles
* **Fix:** Add 20-30% buffer, validate estimates with skill owners
* **Prevention:** Invoke skill owners during work breakdown (Phase 5.4)

**Pitfall 4: No workflow tracking**
* **Symptom:** Work items stall, handoffs missed, blockers unnoticed
* **Fix:** Invoke orchestrator at start, after handoffs, when blocked
* **Prevention:** Follow orchestrator discipline (Phase 6.1)

**Pitfall 5: Vague acceptance criteria**
* **Symptom:** Work items marked "done" but reviewers can't verify
* **Fix:** Make criteria specific and testable ("p95 latency < 50ms" not "fast enough")
* **Prevention:** Review acceptance criteria with skill owner and reviewer upfront

---

**END OF REVIEW PROCESS GUIDE**
