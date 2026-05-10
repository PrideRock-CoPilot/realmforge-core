# RealmForge Remediation — File Naming Conventions

**Purpose:** Standard file locations and naming for area context and audit artifacts

**Status:** 📋 REFERENCE GUIDE

---

## Overview

Every RealmForge area follows a consistent file structure for remediation artifacts. This ensures predictable locations for area owners, reviewers, and the orchestrator.

---

## Standard File Structure Per Area

Each of the 10 RealmForge areas has its own remediation folder:

```
docs/remediation/
├── _templates/              # Templates and reference guides
│   ├── AREA_CONTEXT_TEMPLATE.md
│   ├── AUDIT_QUESTIONS_TEMPLATE.md
│   ├── FILE_NAMING_CONVENTIONS.md (this file)
│   ├── QUESTION_GENERATION_GUIDE.md
│   ├── REMEDIATION_PLAN_TEMPLATE.md
│   ├── REVIEW_PROCESS_GUIDE.md
│   ├── QUICK_REFERENCE.md
│   └── README.md
│
├── 00_INDEX.md             # Master index of all areas and their status
│
├── [area]/                 # One folder per area
│   ├── AREA_CONTEXT.md     # Intent and purpose (Session 1)
│   ├── AUDIT_QUESTIONS.md  # 210 tailored questions (Session 2)
│   ├── AUDIT_REPORT.md     # Audit findings (Phase 3)
│   ├── GAP_ANALYSIS.md     # Gap prioritization (Phase 4)
│   └── REMEDIATION_PLAN.md # Work breakdown (Phase 5)
```

---

## The 10 RealmForge Areas

### 1. RFSource (Storage Layer)
**Folder:** `/docs/remediation/rfsource/`

**Files:**
* `AREA_CONTEXT.md` — RFSource intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for RFSource
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown (96 work items as of reference implementation)

**Status:** ✅ Complete (reference implementation)

---

### 2. Authority (Domain Layer)
**Folder:** `/docs/remediation/authority/`

**Files:**
* `AREA_CONTEXT.md` — Authority domain intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for Authority
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

### 3. Policy (Security Layer)
**Folder:** `/docs/remediation/policy/`

**Files:**
* `AREA_CONTEXT.md` — Policy engine intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for Policy
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

### 4. Catalog (Metadata Layer)
**Folder:** `/docs/remediation/catalog/`

**Files:**
* `AREA_CONTEXT.md` — Catalog intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for Catalog
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

### 5. Audit (Compliance Layer)
**Folder:** `/docs/remediation/audit/`

**Files:**
* `AREA_CONTEXT.md` — Audit log intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for Audit
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

### 6. Snapshot (Event Sourcing)
**Folder:** `/docs/remediation/snapshot/`

**Files:**
* `AREA_CONTEXT.md` — Snapshot intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for Snapshot
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

### 7. Control Plane (Orchestration)
**Folder:** `/docs/remediation/control-plane/`

**Files:**
* `AREA_CONTEXT.md` — Control plane intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for Control Plane
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

### 8. API/MCP/CLI (Interfaces)
**Folder:** `/docs/remediation/api-mcp-cli/`

**Files:**
* `AREA_CONTEXT.md` — API/MCP/CLI intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for APIs
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

### 9. Frontend (UI)
**Folder:** `/docs/remediation/frontend/`

**Files:**
* `AREA_CONTEXT.md` — Frontend intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for Frontend
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

### 10. Infrastructure (Operations)
**Folder:** `/docs/remediation/infrastructure/`

**Files:**
* `AREA_CONTEXT.md` — Infrastructure intent and purpose
* `AUDIT_QUESTIONS.md` — 210 tailored questions for Infrastructure
* `AUDIT_REPORT.md` — Audit findings
* `GAP_ANALYSIS.md` — Gap prioritization
* `REMEDIATION_PLAN.md` — Work breakdown

**Status:** 📋 Pending

---

## File Creation Order (Two-Session Workflow)

### Session 1: Capture Intent
1. **Read templates:**
   * `/docs/remediation/_templates/AREA_CONTEXT_TEMPLATE.md`
   * `/docs/remediation/_templates/REVIEW_PROCESS_GUIDE.md`

2. **Create AREA_CONTEXT.md:**
   * Location: `/docs/remediation/[area]/AREA_CONTEXT.md`
   * Content: Area purpose, responsibilities, boundaries, languages, architecture, key concepts, success criteria
   * **Critical:** Do NOT include implementation details, file names, or current state

3. **Review and sign-off:**
   * CTO (Rena) reviews for architecture alignment
   * Area owner confirms accuracy

---

### Session 2: Generate Questions
1. **Read context and templates:**
   * `/docs/remediation/[area]/AREA_CONTEXT.md` (from Session 1)
   * `/docs/remediation/_templates/AUDIT_QUESTIONS_TEMPLATE.md`
   * `/docs/remediation/_templates/QUESTION_GENERATION_GUIDE.md`
   * **Do NOT read implementation code or current crate files**

2. **Generate AUDIT_QUESTIONS.md:**
   * Location: `/docs/remediation/[area]/AUDIT_QUESTIONS.md`
   * Content: 210 questions tailored to this area's context
   * Format: Use template structure (D1-Q001 through D10-Q010)
   * Questions based on INTENT (what should exist), not CURRENT STATE (what does exist)

3. **Review:**
   * Area owner confirms questions are appropriate
   * Questions test production readiness, not implementation validation

---

## File Naming Rules

### AREA_CONTEXT.md
* **Location:** `/docs/remediation/[area]/AREA_CONTEXT.md`
* **Naming:** Exactly `AREA_CONTEXT.md` (uppercase, no variations)
* **Format:** Markdown
* **Purpose:** Capture area intent without implementation details
* **Created:** Session 1 (before audit begins)

---

### AUDIT_QUESTIONS.md
* **Location:** `/docs/remediation/[area]/AUDIT_QUESTIONS.md`
* **Naming:** Exactly `AUDIT_QUESTIONS.md` (uppercase, no variations)
* **Format:** Markdown
* **Purpose:** 210 tailored audit questions for this area
* **Created:** Session 2 (after AREA_CONTEXT.md exists, before audit execution)

---

### AUDIT_REPORT.md
* **Location:** `/docs/remediation/[area]/AUDIT_REPORT.md`
* **Naming:** Exactly `AUDIT_REPORT.md` (uppercase, no variations)
* **Format:** Markdown
* **Purpose:** Findings from answering all 210 questions
* **Created:** Phase 3 (after audit execution)

---

### GAP_ANALYSIS.md
* **Location:** `/docs/remediation/[area]/GAP_ANALYSIS.md`
* **Naming:** Exactly `GAP_ANALYSIS.md` (uppercase, no variations)
* **Format:** Markdown
* **Purpose:** Prioritized list of gaps extracted from audit
* **Created:** Phase 4 (after audit report)

---

### REMEDIATION_PLAN.md
* **Location:** `/docs/remediation/[area]/REMEDIATION_PLAN.md`
* **Naming:** Exactly `REMEDIATION_PLAN.md` (uppercase, no variations)
* **Format:** Markdown
* **Purpose:** Work breakdown with work streams and work items
* **Created:** Phase 5 (after gap analysis)

---

## Key Principles

### 1. Consistency
All 10 areas use identical file names and locations. No variations allowed.

### 2. Predictability
Any skill or reviewer can find area artifacts at known locations without searching.

### 3. Traceability
Each file references its dependencies:
* `AUDIT_QUESTIONS.md` → `AREA_CONTEXT.md`
* `AUDIT_REPORT.md` → `AUDIT_QUESTIONS.md`
* `GAP_ANALYSIS.md` → `AUDIT_REPORT.md`
* `REMEDIATION_PLAN.md` → `GAP_ANALYSIS.md`

### 4. Intent-Based Questioning
Questions in `AUDIT_QUESTIONS.md` are generated from `AREA_CONTEXT.md` (intent), NOT from implementation code.

### 5. Two-Session Discipline
Session 1 (context) and Session 2 (questions) are separated to prevent implementation details from biasing question generation.

---

## Master Index

All areas and their status are tracked in:
* **Location:** `/docs/remediation/00_INDEX.md`
* **Content:** Table of all 10 areas with file status and completion percentages

---

## Templates Directory

All reference templates live in:
* **Location:** `/docs/remediation/_templates/`
* **Contents:**
  * `AREA_CONTEXT_TEMPLATE.md` — Template for capturing intent
  * `AUDIT_QUESTIONS_TEMPLATE.md` — Template for 210 questions
  * `FILE_NAMING_CONVENTIONS.md` — This file
  * `QUESTION_GENERATION_GUIDE.md` — How to generate questions from context
  * `REMEDIATION_PLAN_TEMPLATE.md` — Template for remediation plans
  * `REVIEW_PROCESS_GUIDE.md` — Complete 6-phase review workflow
  * `QUICK_REFERENCE.md` — One-page cheat sheet
  * `README.md` — Template directory overview

---

## Common Mistakes to Avoid

**❌ Mistake 1:** Creating files with different names (e.g., `authority_context.md` instead of `AREA_CONTEXT.md`)
**✅ Fix:** Always use exact file names as specified in this guide

**❌ Mistake 2:** Generating questions by reading implementation code
**✅ Fix:** Only read `AREA_CONTEXT.md` and `_templates/` when generating questions

**❌ Mistake 3:** Including implementation details in `AREA_CONTEXT.md`
**✅ Fix:** Focus on intent, purpose, and responsibilities — not current state

**❌ Mistake 4:** Skipping Session 1 and jumping straight to question generation
**✅ Fix:** Always create `AREA_CONTEXT.md` first in a separate session

**❌ Mistake 5:** Creating area folders with incorrect names
**✅ Fix:** Use lowercase, hyphenated folder names as specified (e.g., `api-mcp-cli`, not `API_MCP_CLI`)

---

**END OF FILE NAMING CONVENTIONS**
