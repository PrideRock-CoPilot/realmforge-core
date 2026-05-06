# Council Decision: Intake Form Approval Authority

**Decision ID:** DEC-COUNCIL-PENDING (to be assigned after approval)  
**Date Submitted:** 2025-01-30  
**Status:** PENDING COUNCIL REVIEW  
**Decision Type:** COUNCIL_DECISION_REQUIRED  
**Related Document:** docs/INTAKE_SYSTEM_DESIGN.md

---

## Context

The Structured Intake System (docs/INTAKE_SYSTEM_DESIGN.md) includes an AI learning engine that can generate new intake forms when users select "My app type isn't listed" (Section 4.2). These AI-generated forms are stored as drafts and queued for admin review before becoming canonical.

**Current State:** Section 6.2 of INTAKE_SYSTEM_DESIGN.md states:
```
Who Can Approve:
- New Questions: PM (Alex) or CTO (Rena)
- New Forms: Council Decision Required  ← THIS DECISION
- Form Edits: PM (Alex)
```

---

## Question (Q5)

**Who has authority to approve AI-generated intake forms for promotion to canonical status?**

---

## Options

### Option A: Full Council Approval Required

**Authority:** All council members must approve new forms
- CEO (Victor Chen)
- CTO (Dr. Rena Okafor)
- PM (Alex Rivera)
- + Any architect whose domain is affected (likely Domain, Security, API)

**Approval Process:**
1. AI generates draft form → queued for review
2. Admin Review Portal notifies council
3. Council reviews form (async or meeting)
4. Requires consensus (or majority vote, TBD)
5. If approved → promoted to canonical
6. If rejected → blocked, reason documented

**Pros:**
* ✅ Highest governance standard
* ✅ Multiple perspectives catch edge cases
* ✅ Reduces risk of inappropriate forms entering production
* ✅ Aligns with "big decision" pattern (like architecture approval in Phase 3→4)

**Cons:**
* ❌ Slower approval cycle (coordination overhead)
* ❌ Could bottleneck if forms generated frequently
* ❌ May be overkill for simple/obvious forms
* ❌ Council fatigue if many forms queued

**Best For:** High-risk environments, early stages when pattern not established

---

### Option B: PM + CTO Approval Sufficient

**Authority:** PM (Alex) and CTO (Rena) can approve together
- Does NOT require full council or CEO
- May consult other architects as needed (advisory, not blocking)

**Approval Process:**
1. AI generates draft form → queued for review
2. Admin Review Portal notifies PM and CTO
3. PM reviews for business/product fit
4. CTO reviews for technical feasibility
5. Both approve → promoted to canonical
6. Either rejects → blocked or sent back for revision

**Pros:**
* ✅ Faster approval cycle (2 people, clear roles)
* ✅ PM owns product decisions (form content, user experience)
* ✅ CTO owns technical feasibility (modules, personas, complexity)
* ✅ Can still escalate to council for controversial cases
* ✅ Aligns with "PM + CTO" pattern for new questions

**Cons:**
* ❌ Less oversight than full council
* ❌ PM/CTO may miss domain-specific concerns
* ❌ No CEO/business sign-off on new product directions
* ❌ Could approve forms that don't align with strategic goals

**Best For:** Mature systems, high trust in PM/CTO judgment

---

## Option C: Hybrid (Recommended for Discussion)

**Authority:** Tiered approval based on form complexity
- **Simple forms** (≤10 questions, existing modules only): PM + CTO
- **Complex forms** (>10 questions, new modules, regulatory concerns): Full Council

**Approval Process:**
1. AI generates draft form with complexity score
2. If simple → PM + CTO path (Option B)
3. If complex → Council path (Option A)
4. PM/CTO can always escalate to council if uncertain

**Pros:**
* ✅ Balances speed and governance
* ✅ Reserves council time for high-stakes decisions
* ✅ Clear escalation path

**Cons:**
* ❌ Requires defining "complexity" threshold (subjectivity)
* ❌ More complex workflow logic

---

## Stakeholder Perspectives

### Victor (CEO)
**Concerns:**
- New application types = new product directions (strategic)
- Budget implications if forms lead to large projects
- Risk of scope creep if forms too permissive

**Questions for Victor:**
- Should CEO have visibility/veto on new application types?
- Is this a strategic decision (council) or tactical (PM/CTO)?

---

### Rena (CTO)
**Concerns:**
- Technical feasibility of AI-generated forms
- Module/persona assignments must be accurate
- No layer violations or architectural drift

**Questions for Rena:**
- Can you trust AI-generated forms with PM review, or need full council?
- Is this an architecture decision requiring your final sign-off?

---

### Alex (PM)
**Concerns:**
- Form quality and user experience
- Avoiding bad forms that confuse users
- Speed of approval (can't block users for weeks)

**Questions for Alex:**
- Can you approve forms with CTO alone, or need council backing?
- How often do you expect new forms to be generated?

---

### Fatima (Security Architect)
**Concerns:**
- Malicious form injection (Threat T1 in Section 6.1)
- Forms that bypass security review gates
- PII collection in AI-generated questions

**Questions for Fatima:**
- Should security review be part of form approval?
- Option B (PM+CTO) vs Option A (Council) — which is safer?

---

### Dmitri (Backend)
**Implementation Note:**
- Approval logic lives in `intake-engine` service layer
- Simple boolean flag: `requires_council_approval: bool`
- Can support hybrid with form complexity scoring

**No vote required (implementation follows decision)**

---

## Recommendation

**Start with Option A (Full Council), transition to Option B after 5 forms approved.**

**Rationale:**
1. **Early Stage:** System is new, patterns not established → need collective wisdom
2. **Risk Mitigation:** AI-generated forms are unproven → higher oversight initially
3. **Learning Period:** Council reviews first 5 forms → establishes quality bar
4. **Transition:** After 5 successful approvals, council delegates to PM+CTO for routine cases
5. **Escalation:** PM/CTO can always bring forms to council if uncertain

**Transition Criteria:**
- 5 AI-generated forms approved with no major issues
- PM and CTO demonstrate consistent judgment
- Council votes to delegate authority (recorded in minutes)

---

## Proposed Decision Record Format

After council decides, update Section 11.2 of INTAKE_SYSTEM_DESIGN.md:

```markdown
**Q5: Who Can Approve New Forms?**
- **Decision:** [Option A / Option B / Option C]
- **Authority:** [List approvers]
- **Effective Date:** [Date]
- **Review Date:** [Date] (if hybrid/transition)
- **Justification:** [1-2 sentences]
- **Decided By:** RealmForge Decision Council
- **Meeting Date:** [Date]
- **Attendees:** Victor, Rena, Alex, [others]
```

---

## Next Steps

1. **Council Review:** Circulate this document to all council members
2. **Discussion:** Async comments or schedule council meeting
3. **Decision:** Council votes on Option A, B, C, or proposes alternative
4. **Documentation:** Update INTAKE_SYSTEM_DESIGN.md Section 11.2
5. **Implementation:** Update `intake-engine` approval logic
6. **ADR:** Create formal ADR documenting decision and rationale

---

## Related Decisions

- **Q6 (PM Decision):** Auto-promotion threshold (5 occurrences fixed or variable)
- **Q7 (Security Decision):** Can draft forms be used immediately or require approval first
- **DEC-COUNCIL-[###]:** Intake System Architecture (ADR to be created in Task 5)

---

## Appendix: Current Approval Matrix

| Item | Current Approver | Proposed Change |
|------|-----------------|-----------------|
| New Questions (AI-asked) | PM or CTO | No change |
| Form Edits (existing forms) | PM | No change |
| **New Forms (AI-generated)** | **TBD (THIS DECISION)** | **Options A/B/C above** |

---

**Decision Required By:** 2025-02-06 (to unblock Phase 1 implementation)
