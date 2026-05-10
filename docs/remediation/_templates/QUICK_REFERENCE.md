# RealmForge Area Review — Quick Reference Card

**Purpose:** One-page guide for conducting area reviews. Read `REVIEW_PROCESS_GUIDE.md` for full details.

---

## The 6-Phase Workflow

```
Phase 1: Pre-Audit Prep (1-2 days)
   └─→ Orchestrator check → CTO context → Doc inventory

Phase 2: Audit Execution (1-2 weeks)
   └─→ Load domain-audit → Invoke skills → Answer 210 questions

Phase 3: Audit Report (2-3 days)
   └─→ Tech writer consult → Create AUDIT_REPORT.md → CTO review

Phase 4: Gap Analysis (3-5 days)
   └─→ Extract gaps → PM prioritization → Create GAP_ANALYSIS.md

Phase 5: Remediation Plan (1-2 weeks)
   └─→ Work streams → Work items → Skill validation → Create REMEDIATION_PLAN.md

Phase 6: Execution Kickoff (1 day)
   └─→ Orchestrator init → PM assignment → Weekly checkpoints
```

---

## Essential Skill Invocations

| When | Invoke | What to Ask |
|------|--------|-------------|
| **Start audit** | `orchestrator` | "What's in-flight for [AREA]?" |
| **Architecture context** | `hi Rena` | "What does [AREA] own/not own? Known gaps?" |
| **Domain model review** | `hi Yusuf` | "Review entity types, state machines, invariants" |
| **Security review** | `hi Fatima` | "Threat model, attack surface, audit trail" |
| **Data review** | `hi Chen` | "Schema design, indexes, persistence layer" |
| **Infrastructure review** | `hi Nadia` | "PostgreSQL, observability, runbooks" |
| **Code review** | `hi Owen` | "Review [crates] for compliance, safety, tests" |
| **Implementation review** | `hi Dmitri` | "Rust idioms, error handling, coverage" |
| **Prioritization** | `hi Alex` | "Prioritize [N] gaps by impact and dependencies" |
| **Report structure** | `hi Clara` | "Help structure audit report for [AREA]" |
| **Strategic approval** | `hi Victor` | "Approve [N] weeks investment for [AREA]" |

---

## The 210-Question Framework (Domain Audit)

**Always load first:** Domain audit skill before starting evidence collection

**10 Dimensions:**
* **D1:** Documentation (20 questions) — Inline docs, README, architecture, APIs
* **D2:** Testing (25 questions) — Unit, integration, property-based, concurrency, failure injection
* **D3:** Scalability (20 questions) — Load testing, performance benchmarks, resource limits
* **D4:** Versioning (25 questions) — Migrations, API versioning, backward compatibility
* **D5:** File Size (20 questions) — File size limits, module boundaries, code organization
* **D6:** Security (30 questions) — Threat models, access control, audit trails, encryption
* **D7:** Error Handling (25 questions) — Error types, recovery, graceful degradation
* **D8:** Performance (25 questions) — Benchmarks, profiling, optimization, resource usage
* **D9:** Operational (10 questions) — Monitoring, runbooks, deployment, disaster recovery
* **D10:** Standardization (10 questions) — Coding standards, naming, architecture compliance

**Coverage targets:**
* **MVP:** ≥70% overall, ≥60% critical
* **Production:** ≥90% overall, ≥85% critical
* **Excellence:** 100% overall, 100% critical

---

## Critical Issue Severity Guide

| Severity | Meaning | Example |
|----------|---------|---------|
| 🔴 **BLOCKER** | Blocks other work, production deployment | No WAL → data loss on crash |
| 🟠 **HIGH** | Significant risk, must fix before release | Governance bypassable via direct file access |
| 🟡 **MEDIUM** | Important, impacts quality/scalability | O(n) search doesn't scale |
| 🟢 **LOW** | Nice-to-have, minor impact | Missing performance benchmarks |

---

## Work Stream Breakdown Pattern

**Typical work streams for Rust areas:**
1. **Foundation** — Core types, architecture, layer boundaries
2. **Security** — Threat mitigation, audit trail, enforcement
3. **Data Layer** — Schema, indexes, persistence, integrity
4. **Service Logic** — Business rules, orchestration, transactions
5. **Testing** — Unit, integration, property-based, concurrency, failure injection
6. **Documentation** — README, rustdoc, ADRs, guides
7. **Operations** — Monitoring, runbooks, deployment, disaster recovery
8. **Performance** — Optimization, benchmarking, profiling

**Ownership assignment:**
* Foundation → Dmitri (Backend) or Yusuf (Domain Architect)
* Security → Fatima (Security Architect)
* Data Layer → Chen (Data Architect)
* Operations → Nadia (Infra Architect)
* Documentation → Clara (Tech Writer)
* Testing → Dmitri + Meg (QA)

---

## Handoff Protocol

**Every handoff must be explicit:**

1. **Sender:** Completes deliverable, fills HANDOFF section
2. **Sender:** Pings receiver: "Ready for review at [location]"
3. **Receiver:** Reviews against acceptance criteria
4. **Receiver:** Either (a) provides specific feedback or (b) signs off
5. **Receiver:** Updates SIGN-OFF section with name and date
6. **Receiver:** Notifies next skill in chain if applicable
7. **Orchestrator:** Records handoff with date/sender/receiver/deliverable

**No implicit handoffs. No status changes without confirmed receipt.**

---

## Quality Gates (All 4 Required for Production)

**Gate 1: Specification Complete**
- [ ] Requirements documented, architecture approved (Rena), security reviewed (Fatima), API contracts defined (Marcus), ADRs written (Clara)

**Gate 2: Implementation Complete**
- [ ] Code complete, unit tests ≥80%, integration tests passing, code review passed (Owen), peer review passed (Nora)

**Gate 3: QA Certified**
- [ ] Acceptance tests passing, performance benchmarks met, security tests passed, QA sign-off (Meg)

**Gate 4: Production Ready**
- [ ] Runbook complete, monitoring configured, disaster recovery tested, documentation complete, Release Manager sign-off (Sam)

---

## Escalation Thresholds

| Time Blocked | Action |
|--------------|--------|
| **< 4 hours** | Owner resolves within domain |
| **4-24 hours** | Escalate to Nora (Peer Review) for independent assessment |
| **24-48 hours** | Escalate to Alex (PM) and Orchestrator |
| **48-72 hours** | Escalate to Rena (CTO) for technical decision |
| **> 72 hours** | Escalate to Victor (CEO) for strategic call |

**Emergency:** Security critical → Fatima → Rena → Victor immediately

---

## File Locations

**Templates:**
* `/docs/remediation/_templates/REMEDIATION_PLAN_TEMPLATE.md`
* `/docs/remediation/_templates/REVIEW_PROCESS_GUIDE.md` (full guide)
* `/docs/remediation/_templates/QUICK_REFERENCE.md` (this file)

**Area folders:**
* `/docs/remediation/[area]/README.md` — Area description
* `/docs/remediation/[area]/AUDIT_REPORT.md` — 210-question evidence
* `/docs/remediation/[area]/GAP_ANALYSIS.md` — Prioritized gaps
* `/docs/remediation/[area]/REMEDIATION_PLAN.md` — Work items with handoffs

**Master index:**
* `/docs/remediation/00_INDEX.md` — Status tracking for all 10 areas

---

## Common Mistakes to Avoid

❌ **Skipping skill invocation** — Don't attempt architecture review without Rena, security without Fatima
✅ **Use skill invocation matrix** — Know when to invoke which skill

❌ **Answering audit questions without evidence** — "Yes" without file path/test result
✅ **Every answer references specific location** — "Yes, see /crates/X/src/Y.rs:123"

❌ **Vague acceptance criteria** — "Performance is good enough"
✅ **Specific and testable** — "p95 latency < 50ms under 100 concurrent requests"

❌ **Forgetting orchestrator** — Work items stall, handoffs missed
✅ **Invoke at start, after handoffs, when blocked** — Explicit workflow tracking

❌ **Underestimating effort** — 2 weeks → actually 6 weeks
✅ **Add 20-30% buffer, validate with skill owners** — Realistic timelines

---

## Example: Starting an Authority Domain Audit

```
User: "hi orchestrator, starting audit of Authority domain. What's in-flight?"
→ Orchestrator provides workflow state

User: "hi Rena, I'm auditing Authority domain. What does it own/not own? Known gaps?"
→ CTO provides architecture context

User: "Load domain-audit skill"
→ 210-question framework activated

User: "hi Yusuf, Authority domain audit in progress. Review entity types, state machines, invariants"
→ Domain Architect provides evidence

User: "hi Fatima, Authority domain audit in progress. Review threat model and access control"
→ Security Architect provides evidence

[Continue through all 210 questions...]

User: "hi Clara, audit complete with 85% coverage. Help structure AUDIT_REPORT.md"
→ Tech Writer provides report template

[Create AUDIT_REPORT.md, GAP_ANALYSIS.md, REMEDIATION_PLAN.md per templates]

User: "hi Rena, audit report ready for architecture review at /docs/remediation/authority/AUDIT_REPORT.md"
→ CTO reviews and signs off

User: "orchestrator, Authority remediation plan approved. Initialize workflow"
→ Orchestrator registers work items, identifies first task

User: "hi Alex, Authority remediation begins. First work item: WS-01-001, assigned to Yusuf"
→ PM coordinates kickoff
```

---

**Read full guide:** `/docs/remediation/_templates/REVIEW_PROCESS_GUIDE.md`

**Master index:** `/docs/remediation/00_INDEX.md`
