# PM Decision: Auto-Promotion Threshold for AI Questions

**Decision ID:** DEC-PM-PENDING (to be assigned after decision)  
**Date Submitted:** 2025-01-30  
**Status:** PENDING PM REVIEW  
**Decision Type:** PM_DECISION_REQUIRED  
**Decision Owner:** Alex Rivera (PM)  
**Related Document:** docs/INTAKE_SYSTEM_DESIGN.md

---

## Context

The Structured Intake System's Smart Learning Engine (Section 4 of INTAKE_SYSTEM_DESIGN.md) tracks AI-asked questions and auto-promotes them to canonical forms when they reach a frequency threshold. Currently, the design proposes a fixed threshold of **5 occurrences** across different intake sessions.

**Current State:** Section 4.3 of INTAKE_SYSTEM_DESIGN.md states:
```
Auto-Promotion Logic:
- Question asked ≥5 times across different sessions → Queue for review
```

---

## Question (Q6)

**Should the auto-promotion threshold be fixed (5 occurrences for all app types) or variable (different thresholds per app type)?**

---

## Options

### Option A: Fixed Threshold (5 Occurrences for All)

**Configuration:**
```python
AUTO_PROMOTION_THRESHOLD = 5  # global constant
```

**Logic:**
- Any AI question asked ≥5 times across ANY app type → queued for admin review
- Simple, universal rule

**Pros:**
* ✅ Simple to implement and explain
* ✅ Predictable behavior
* ✅ Easy to adjust globally (change one constant)
* ✅ No app-type-specific configuration needed
* ✅ Fair treatment across all app types

**Cons:**
* ❌ May promote too quickly for rare/niche app types (e.g., "Blockchain DApp" with few users)
* ❌ May promote too slowly for common app types (e.g., "Static Website" with many users)
* ❌ Doesn't account for app type popularity differences

**Example:**
- "Static Website" intake: 500 sessions/month → 5 occurrences = 1% frequency (good signal)
- "Quantum Computing App" intake: 10 sessions/month → 5 occurrences = 50% frequency (noise?)

---

### Option B: Variable Threshold by App Type

**Configuration:**
```python
AUTO_PROMOTION_THRESHOLDS = {
    "01_static_website": 10,        # High traffic
    "03_dynamic_web_app": 8,        # High traffic
    "05_api_service": 8,            # High traffic
    "11_ai_ml_application": 5,      # Medium traffic
    "14_blockchain_dapp": 3,        # Low traffic
    # ... etc
}

DEFAULT_THRESHOLD = 5  # fallback for new app types
```

**Logic:**
- Threshold varies by app type popularity
- High-traffic app types need more occurrences to confirm pattern
- Low-traffic app types can promote with fewer occurrences

**Pros:**
* ✅ Adapts to app type popularity
* ✅ Better signal-to-noise ratio per app type
* ✅ Faster learning for niche app types
* ✅ Higher quality bar for common app types

**Cons:**
* ❌ More complex configuration
* ❌ Requires monitoring usage to set thresholds correctly
* ❌ Need to update thresholds as usage patterns change
* ❌ Could feel arbitrary ("why is my threshold 3 and theirs 10?")

**Example:**
- "Static Website" (500 sessions/month, threshold=10): 10 occurrences = 2% frequency (strong signal)
- "Quantum Computing App" (10 sessions/month, threshold=3): 3 occurrences = 30% frequency (reasonable signal)

---

### Option C: Percentage-Based Threshold (Hybrid)

**Configuration:**
```python
AUTO_PROMOTION_THRESHOLD_PERCENTAGE = 0.05  # 5% of sessions
MIN_OCCURRENCES = 3  # absolute minimum
MAX_OCCURRENCES = 15  # absolute maximum
```

**Logic:**
- Threshold = max(MIN_OCCURRENCES, min(MAX_OCCURRENCES, 5% of sessions in last 30 days))
- Automatically adapts to usage without manual configuration

**Pros:**
* ✅ Self-adjusting based on actual usage
* ✅ No manual threshold configuration needed
* ✅ Fair across all app types (always 5% frequency)
* ✅ Bounded by min/max to prevent extremes

**Cons:**
* ❌ More complex to understand
* ❌ Requires tracking session counts per app type
* ❌ May be unpredictable ("why did my question promote now but not last week?")
* ❌ Could delay promotion if usage drops temporarily

**Example:**
- "Static Website" (500 sessions/month): 5% = 25 → capped at 15 occurrences
- "Quantum Computing App" (10 sessions/month): 5% = 0.5 → raised to 3 (minimum)

---

## Recommendation

**Option A: Fixed Threshold (5 Occurrences)**

**Rationale:**
1. **Simplicity First:** System is new, start simple and add complexity only if needed
2. **Sufficient Signal:** 5 occurrences across different sessions is a reasonable signal regardless of app type
3. **Predictability:** Users and admins understand "5 times = review" without special cases
4. **Easy Tuning:** If 5 proves wrong, adjust globally and measure
5. **Admin Review Gate:** Since all promotions require admin approval anyway, admins can reject low-quality questions even if threshold met

**Tuning Strategy:**
- Start with 5 (current proposal)
- After 3 months, analyze:
  - How many questions promoted per app type?
  - What % of promoted questions were approved by admins?
  - Are any app types generating noise (many promotions, low approval rate)?
- If issues detected → consider Option B (variable) or Option C (percentage)

**Override Capability:**
- Allow admins to manually adjust threshold for specific app types if needed
- Log threshold overrides in audit trail

---

## Impact Analysis

### If Fixed Threshold Too Low (promotes too early):
- **Symptom:** Admin review queue fills with low-quality questions
- **Detection:** Low approval rate (<60%) for promoted questions
- **Mitigation:** Increase threshold to 7 or 10
- **Risk:** Low (admin review catches bad questions)

### If Fixed Threshold Too High (promotes too late):
- **Symptom:** Valuable questions asked repeatedly but never promoted
- **Detection:** High AI assistance usage (>10% of sessions)
- **Mitigation:** Decrease threshold to 3 or 4
- **Risk:** Medium (user experience degrades)

### Data Collection for Future Decision:
Track these metrics to inform Option B or C in the future:
- AI questions asked per app type
- Session count per app type (monthly)
- Promotion rate per app type
- Admin approval rate per app type

---

## Implementation Notes

**Database Change:** None (existing `question_frequency.occurrence_count` field supports all options)

**Service Logic:**
```python
# Option A (recommended)
def check_promotion_threshold(question_id: UUID) -> bool:
    """Check if question meets auto-promotion threshold."""
    frequency = get_question_frequency(question_id)
    return frequency.occurrence_count >= 5

# Option B (if chosen later)
def check_promotion_threshold(question_id: UUID, app_type_id: str) -> bool:
    frequency = get_question_frequency(question_id)
    threshold = AUTO_PROMOTION_THRESHOLDS.get(app_type_id, 5)
    return frequency.occurrence_count >= threshold

# Option C (if chosen later)
def check_promotion_threshold(question_id: UUID, app_type_id: str) -> bool:
    frequency = get_question_frequency(question_id)
    sessions_last_30_days = get_session_count(app_type_id, days=30)
    threshold = max(3, min(15, int(sessions_last_30_days * 0.05)))
    return frequency.occurrence_count >= threshold
```

---

## Proposed Decision Record Format

After PM decides, update Section 11.2 of INTAKE_SYSTEM_DESIGN.md:

```markdown
**Q6: Auto-Promotion Threshold**
- **Decision:** [Option A / Option B / Option C]
- **Threshold:** [5 fixed / variable by app type / 5% percentage]
- **Effective Date:** [Date]
- **Review Date:** [Date] (recommend 3-month review)
- **Justification:** [1-2 sentences]
- **Decided By:** Alex Rivera (PM)
- **Decision Date:** [Date]
```

---

## Next Steps

1. **PM Review:** Alex reviews options and decides
2. **Documentation:** Update INTAKE_SYSTEM_DESIGN.md Section 11.2 and Section 4.3
3. **Implementation:** Update `intake-engine` service layer with chosen threshold logic
4. **Monitoring:** Set up dashboard to track promotion metrics
5. **3-Month Review:** Analyze data and consider adjusting

---

**Decision Required By:** 2025-02-06 (to finalize Phase 2 implementation plan)
