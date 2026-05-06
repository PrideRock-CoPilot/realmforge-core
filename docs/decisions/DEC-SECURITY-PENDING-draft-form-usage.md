# Security Architect Decision: AI-Generated Draft Form Usage

**Decision ID:** DEC-SECURITY-PENDING (to be assigned after decision)  
**Date Submitted:** 2025-01-30  
**Status:** PENDING SECURITY REVIEW  
**Decision Type:** SECURITY_DECISION_REQUIRED  
**Decision Owner:** Fatima Al-Hassan (Security Architect)  
**Related Document:** docs/INTAKE_SYSTEM_DESIGN.md

---

## Context

The Structured Intake System allows AI to generate new intake forms on-the-fly when a user selects "My app type isn't listed" (Section 4.2). These forms contain conditional questions, module mappings, and persona assignments. The security question is whether these AI-generated draft forms can be used immediately for the requesting user's intake session, or must be queued for approval before ANY use.

**Threat Model Reference:** Section 6.1, Threat T1: Malicious Form Injection

---

## Question (Q7)

**Can AI-generated draft forms be used immediately by the requesting user, or must they be approved before first use?**

---

## Options

### Option A: Immediate Use (Current Design)

**Workflow:**
1. User requests new app type → AI generates draft form
2. Draft form stored with `status='draft'` and `is_ai_generated=true`
3. **User proceeds with intake using draft form immediately**
4. Draft form queued for admin review (async)
5. If approved → promoted to canonical for future users
6. If rejected → blocked for future users (but original user's intake already complete)

**Pros:**
* ✅ Best user experience (no wait time)
* ✅ Unblocks user immediately
* ✅ Encourages system usage (users see value instantly)
* ✅ Reduces abandonment rate

**Cons:**
* ❌ **Security Risk:** Malicious user could craft prompts to generate exploitative forms
* ❌ **Data Quality:** Bad form could capture wrong requirements
* ❌ **No Pre-Validation:** Admin reviews AFTER user already used form
* ❌ **Cleanup Cost:** If rejected, user's project may be based on bad requirements

---

### Option B: Approval Before First Use

**Workflow:**
1. User requests new app type → AI generates draft form
2. Draft form stored with `status='pending_approval'`
3. **User notified: "Form generated, awaiting approval (est. 24-48 hours)"**
4. Admin reviews and approves/rejects
5. If approved → user notified, can now start intake
6. If rejected → user notified, asked to refine request

**Pros:**
* ✅ **Security:** Admin validates form before any use (no malicious forms in production)
* ✅ **Quality Gate:** Bad forms caught before impacting users
* ✅ **No Cleanup:** Rejections happen before projects created
* ✅ **Audit Trail:** All forms vetted before use

**Cons:**
* ❌ **User Experience:** 24-48 hour delay (or longer if admins busy)
* ❌ **Abandonment Risk:** Users may leave if blocked
* ❌ **Admin Bottleneck:** Could overwhelm admin queue
* ❌ **First-Mover Penalty:** User who discovers new app type pays waiting cost

---

### Option C: Hybrid - One-Time Use, Approval for Reuse (Recommended)

**Workflow:**
1. User requests new app type → AI generates draft form
2. Draft form stored with `status='draft'` and `one_time_use=true`
3. **User proceeds with intake using draft form (one-time)**
4. Draft form queued for admin review
5. If approved → promoted to canonical, available for all future users
6. If rejected → blocked for future use, original user's session marked for manual review

**Pros:**
* ✅ **User Experience:** No wait time for requesting user
* ✅ **Security:** Form not available to OTHER users until approved
* ✅ **Risk Containment:** Damage limited to single user's session
* ✅ **Quality Feedback:** Admin can see how form was actually used before deciding

**Cons:**
* ❌ **Partial Risk:** Requesting user still exposed to potentially bad form
* ❌ **Review Overhead:** Admin must review both form AND resulting session
* ❌ **Cleanup Risk:** If rejected, may need to audit/fix user's project

---

## Security Analysis

### Threat T1: Malicious Form Injection (from Section 6.1)

**Attack Scenario:**
1. Attacker describes fake app type: "I need a system that collects Social Security Numbers and credit card data"
2. AI generates form with questions: "SSN?", "Credit Card Number?", "CVV?"
3. If **Option A:** Form used immediately → PII collected, stored in database
4. If **Option B:** Admin catches malicious intent, rejects form before use
5. If **Option C:** Attacker's own intake collects PII (self-harm?), but form blocked for others

**Risk Assessment:**
- **Option A:** ❌ **HIGH RISK** — Malicious forms can collect PII from ANY user before review
- **Option B:** ✅ **LOW RISK** — No form used without admin validation
- **Option C:** ⚠️ **MEDIUM RISK** — Damage limited to attacker's own session

**Mitigation Layers:**
1. **Input Validation:** AI prompt sanitization, reject suspicious keywords (SSN, credit card, etc.)
2. **Rate Limiting:** Max 1 new form per user per 24 hours
3. **PII Detection:** Scan AI-generated questions for PII keywords, flag for extra review
4. **Session Monitoring:** Flag sessions using draft forms for manual audit
5. **Admin Review:** All draft forms reviewed within 24 hours (SLA)

---

### Threat T2: Form Quality Degradation

**Attack Scenario:**
1. User poorly describes app type: "I need a thing that does stuff"
2. AI generates vague/useless form
3. User completes intake with nonsense answers
4. Project created with garbage requirements

**Risk Assessment:**
- **Option A:** ❌ **HIGH RISK** — Bad project created immediately
- **Option B:** ✅ **LOW RISK** — Admin catches bad form before use
- **Option C:** ⚠️ **MEDIUM RISK** — One bad project, but no others affected

---

### Threat T3: Resource Exhaustion

**Attack Scenario:**
1. Attacker floods system with draft form requests
2. Admin queue overwhelmed
3. Legitimate forms delayed

**Mitigation (all options):**
- Rate limiting: 1 form per user per 24 hours
- Cost tracking: Flag users generating many forms
- Admin prioritization: Legitimate users' forms reviewed first

---

## Recommendation

**Option C: Hybrid (One-Time Use, Approval for Reuse)**

**Rationale:**
1. **Balances Security & UX:** Requesting user not blocked, but damage contained
2. **Risk Containment:** Malicious forms can only harm attacker's own session (self-inflicted)
3. **Learning Feedback:** Admin sees actual usage before deciding on canonical promotion
4. **Pragmatic:** Real-world risk of malicious forms is LOW (user would harm their own project)
5. **Mitigation Layers:** Combined with PII detection + rate limiting + admin review

**Enhanced Security Controls:**
1. **PII Detection:** Scan all AI-generated questions for PII keywords
   - If detected → Block immediate use, require approval (fall back to Option B)
2. **Session Flagging:** Mark sessions using draft forms for manual review
3. **24-Hour Admin SLA:** All draft forms reviewed within 24 hours
4. **Rollback Capability:** If form rejected, provide guidance to user on revising requirements

**Implementation:**
```sql
-- Add flag to intake_sessions
ALTER TABLE intake_sessions ADD COLUMN used_draft_form BOOLEAN DEFAULT FALSE;

-- Add PII detection result
ALTER TABLE application_types ADD COLUMN pii_detected BOOLEAN DEFAULT FALSE;
```

**Decision Logic:**
```python
def can_use_draft_form(form: ApplicationType, user: Actor) -> Tuple[bool, str]:
    """Determine if draft form can be used immediately."""
    
    # PII detected → require approval
    if form.pii_detected:
        return (False, "Form contains potential PII questions, awaiting admin review")
    
    # Rate limit exceeded → block
    if user.draft_forms_created_today >= 1:
        return (False, "Rate limit: max 1 new form per 24 hours")
    
    # OK for one-time use by requesting user
    return (True, "Draft form available for your intake session")
```

---

## Fallback Plan

If Option C proves risky in practice:
1. **Monitor:** Track sessions using draft forms (completion rate, data quality)
2. **Alert:** Flag any forms collecting PII or suspicious data
3. **Escalate:** If ANY malicious form detected, switch to Option B (approval-first)
4. **Review:** After 3 months, analyze incidents and decide to keep Option C or switch

---

## Proposed Decision Record Format

After Security Architect decides, update Section 11.2 of INTAKE_SYSTEM_DESIGN.md:

```markdown
**Q7: AI-Generated Form Usage**
- **Decision:** [Option A / Option B / Option C]
- **Usage Policy:** [Immediate use / Approval-first / One-time use with reuse approval]
- **Security Controls:** [PII detection, rate limiting, admin SLA]
- **Effective Date:** [Date]
- **Review Date:** [Date] (recommend 3-month review)
- **Justification:** [1-2 sentences]
- **Decided By:** Fatima Al-Hassan (Security Architect)
- **Decision Date:** [Date]
```

---

## Integration with Q5 (Council Decision)

Note: Q5 determines who approves forms for canonical status. Q7 determines whether forms can be used BEFORE that approval.

**Combined Scenarios:**
- **Q5 = Option A (Council), Q7 = Option C (One-time):** Council approves for reuse, user uses immediately
- **Q5 = Option B (PM+CTO), Q7 = Option B (Approval-first):** PM+CTO must approve before anyone uses
- **Q5 = Option B (PM+CTO), Q7 = Option C (One-time):** PM+CTO decides on reuse, requesting user uses immediately

**Recommended Pairing:** Q5 Option A or B + Q7 Option C

---

**Decision Required By:** 2025-02-06 (to finalize Phase 4 implementation security requirements)
