# RealmForge: Agent & Workflow Architecture

**Document Purpose:**  
Define how RealmForge prevents hallucinations and ensures quality through STRUCTURAL enforcement, not prompt engineering. This is the "control plane for AI agents" - how LLMs become governed, traceable, auditable workers.

**Date:** 2025-01-XX  
**Status:** 🚨 **CRITICAL DESIGN GAP** - Must be designed before implementation resumes  
**Audience:** Architects, engineers, and anyone building agent-powered systems

---

## The Core Problem

**Current state of AI agent systems:**
* LLMs are given instructions: "Don't hallucinate", "Follow the process", "Ask good questions"
* LLMs ignore instructions when convenient
* No enforcement of order-of-operations
* No prevention of skipping steps
* Quality depends on "hoping the LLM is smart enough"

**Example failure:**
```
User: "I need a login screen"
Bad LLM: "Here's a login form component!" (generates code immediately)
  ❌ Didn't ask: OAuth or email/password?
  ❌ Didn't ask: Multi-factor auth required?
  ❌ Didn't ask: Session management strategy?
  ❌ Didn't ask: User provisioning workflow?
  ❌ Didn't ask: Password reset flow?
  ❌ Didn't create design doc
  ❌ Didn't get sign-off
  ❌ Just started coding
```

**The user's insight:**
> "This is not done by wishing the LLM is magically better. This is done by having the path defined. We need the app to fill in the missing questions that the requestor either did not think about or does not know."

**What's needed:**
* Structural enforcement (the app refuses to proceed without required inputs)
* Workflow gates (can't code without design approval)
* Guided intake (system knows what questions to ask)
* Self-improving agents (learn from past failures)

---

## What is an Agent in RealmForge?

An **Agent** is NOT a raw LLM. It's a governed execution context with:

1. **Identity** - Who is this agent? (Policy crate: `AgentId`, `AgentCredential`)
2. **Capabilities** - What CAN it do? (Policy crate: `CapabilityGrant`)
3. **Workspace** - What CAN it see/modify? (Policy crate: `WorkPath`, `BlastRadius`)
4. **Audit Trail** - What DID it do? (Audit crate: `EventLog`)
5. **Cost Tracking** - What did it COST? (Catalog crate: `CostRecord`)
6. **Workflow State** - What MUST happen next? (Control-plane crate: `WorkflowGate`)

**Agents are CONTAINED:**
```
┌─────────────────────────────────────────────┐
│  Agent: ai-agent-login-screen-12345         │
│                                             │
│  Identity:                                  │
│    AgentId: agent-abc123                    │
│    Credential: API key (scoped, expiring)   │
│                                             │
│  Capabilities (from policy-engine):         │
│    ✅ READ  /docs/spec/*                    │
│    ✅ WRITE /docs/spec/intake/*             │
│    ❌ WRITE /crates/* (DENIED - no design)  │
│    ❌ WRITE /db/migrations/* (DENIED)       │
│                                             │
│  Workflow State (from control-plane):       │
│    Current Phase: INTAKE                    │
│    Next Gate: DESIGN_APPROVAL_REQUIRED      │
│    Blocked: Cannot proceed to DESIGN until  │
│             all intake questions answered   │
│                                             │
│  Cost Tracking:                             │
│    Tokens: 15,432                           │
│    Cost: $0.23                              │
│    Owner: work-path-login-feature           │
└─────────────────────────────────────────────┘
```

**Key Principle:**
> An agent is a WORKER, not a decision-maker. It executes within boundaries set by policy, workflow, and approval gates.

---

## How Capabilities Work

### Capability Model (Policy Crate)

**Capabilities are EXPLICIT grants, not inferred permissions.**

```rust
// authority-domain/src/capability.rs
pub struct CapabilityGrant {
    pub agent_id: AgentId,
    pub work_path: WorkPath,
    pub capability: Capability,
    pub scope: Scope,
    pub granted_by: UserId,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub enum Capability {
    Read,
    Write,
    Execute,
    Delete,
    Approve,  // Only humans or designated approvers
}

pub struct Scope {
    pub paths: Vec<PathPattern>,  // /docs/spec/*, /crates/control-api/*
    pub operations: Vec<Operation>,  // CREATE_FILE, UPDATE_FILE, DELETE_FILE
    pub blast_radius: BlastRadiusLimit,  // Max files/lines affected
}

pub struct BlastRadiusLimit {
    pub max_files: usize,
    pub max_lines: usize,
    pub max_cost_usd: f64,
    pub requires_approval_above: Option<BlastRadiusThreshold>,
}
```

**Grant Example:**
```rust
// Intake agent gets limited write access
CapabilityGrant {
    agent_id: AgentId("agent-intake-001"),
    work_path: WorkPath("login-feature"),
    capability: Capability::Write,
    scope: Scope {
        paths: vec![
            PathPattern::new("/docs/spec/intake/*"),
            PathPattern::new("/docs/spec/22_OPEN_DECISIONS.md"),
        ],
        operations: vec![
            Operation::CreateFile,
            Operation::UpdateFile,
        ],
        blast_radius: BlastRadiusLimit {
            max_files: 10,
            max_lines: 500,
            max_cost_usd: 1.00,
            requires_approval_above: Some(BlastRadiusThreshold {
                files: 5,
                lines: 200,
            }),
        },
    },
    granted_by: UserId("user-alice"),
    granted_at: Utc::now(),
    expires_at: Some(Utc::now() + Duration::hours(24)),
}
```

### Capability Enforcement (Policy Crate)

**Every agent action goes through policy check:**

```rust
// policy-engine/src/lib.rs
pub struct PolicyEngine {
    grants: GrantStore,
    audit: AuditLog,
}

impl PolicyEngine {
    pub fn check_permission(
        &self,
        agent_id: AgentId,
        action: Action,
    ) -> Result<PermissionDecision, PolicyError> {
        // 1. Load agent's grants
        let grants = self.grants.for_agent(&agent_id)?;
        
        // 2. Check if action matches any grant
        let matching_grant = grants.iter()
            .find(|g| g.permits(&action))
            .ok_or(PolicyError::PermissionDenied)?;
        
        // 3. Check blast radius
        if action.exceeds_blast_radius(&matching_grant.scope) {
            return Ok(PermissionDecision::RequiresApproval {
                reason: "Blast radius exceeded",
                approver_required: matching_grant.granted_by,
            });
        }
        
        // 4. Log to audit trail
        self.audit.log(AuditEvent::PermissionCheck {
            agent_id,
            action: action.clone(),
            decision: PermissionDecision::Granted,
            timestamp: Utc::now(),
        })?;
        
        Ok(PermissionDecision::Granted)
    }
}
```

**Result:**
* Agent asks: "Can I write to /crates/control-api/src/main.rs?"
* Policy engine checks grants
* No grant exists for /crates/* (agent only has /docs/spec/intake/* write)
* Returns `PermissionDenied`
* Agent CANNOT proceed - the app refuses the action

---

## Workflow Gates: Enforcing Order of Operations

### The Workflow Model (Control-Plane Crate)

**Workflow phases are EXPLICIT states with gates between them.**

```rust
// control-service/src/workflow.rs
pub struct Workflow {
    pub work_path: WorkPath,
    pub current_phase: Phase,
    pub gates: Vec<WorkflowGate>,
    pub history: Vec<PhaseTransition>,
}

pub enum Phase {
    Intake,           // Gathering requirements
    Design,           // Creating design docs
    DesignApproval,   // Waiting for human sign-off
    Implementation,   // Writing code
    CodeReview,       // Peer review
    QA,               // Testing
    Deployment,       // Deploying to environment
    Monitoring,       // Post-deployment observation
}

pub struct WorkflowGate {
    pub from_phase: Phase,
    pub to_phase: Phase,
    pub requirements: Vec<Requirement>,
    pub enforced_by: GateEnforcer,
}

pub enum Requirement {
    AllQuestionsAnswered,
    DesignDocExists,
    DesignApproved { by: UserId },
    TestsPassing,
    BlastRadiusCalculated,
    CostEstimated,
    SecurityReviewed,
    OpenDecisionsClosed,
}

pub enum GateEnforcer {
    Automatic,    // System checks conditions
    Manual,       // Human approval required
    Hybrid,       // System checks + human approval
}
```

### Gate Example: Cannot Code Without Design Approval

```rust
// control-service/src/gates/design_approval.rs
pub struct DesignApprovalGate;

impl WorkflowGate for DesignApprovalGate {
    fn can_transition(
        &self,
        work_path: &WorkPath,
        from: Phase,
        to: Phase,
    ) -> Result<GateDecision, WorkflowError> {
        // Attempting to go from Design → Implementation?
        if from == Phase::Design && to == Phase::Implementation {
            // Check requirements
            let design_doc = self.check_design_doc_exists(work_path)?;
            let approval = self.check_design_approved(work_path)?;
            let open_decisions = self.check_open_decisions_closed(work_path)?;
            
            if !design_doc {
                return Ok(GateDecision::Blocked {
                    reason: "Design document does not exist",
                    action_required: "Create /docs/spec/<feature>/DESIGN.md",
                });
            }
            
            if !approval {
                return Ok(GateDecision::Blocked {
                    reason: "Design not approved by human",
                    action_required: "Get design approval from stakeholder",
                    approver: work_path.owner(),
                });
            }
            
            if !open_decisions {
                return Ok(GateDecision::Blocked {
                    reason: "Open decisions remain in 22_OPEN_DECISIONS.md",
                    action_required: "Close all USER_APPROVAL_REQUIRED and COUNCIL_DECISION_REQUIRED items",
                });
            }
            
            // All requirements met
            Ok(GateDecision::Allowed)
        } else {
            Ok(GateDecision::Allowed)  // Other transitions not blocked by this gate
        }
    }
}
```

**Result:**
* Agent tries to transition: Design → Implementation
* Gate checks:
  1. ❌ Design doc exists? NO → **BLOCKED**
  2. Design approved? (not checked, already blocked)
  3. Open decisions closed? (not checked, already blocked)
* Agent receives: `GateDecision::Blocked`
* Agent CANNOT write code - workflow refuses transition

---

## The Intake Agent: Guided Discovery

### What is the Intake Agent?

**Purpose:** Turn vague user requests into complete, unambiguous specs.

**Capabilities:**
* ✅ READ: Existing specs, design docs, open decisions
* ✅ WRITE: Intake docs, open decisions register
* ❌ WRITE: Implementation code (no grant)
* ❌ APPROVE: Designs (not authorized)

**Workflow Phase:** INTAKE (locked until all questions answered)

### The Intake Workflow

**Example: User says "I need a login screen"**

```
Step 1: Load Intake Checklist
  Agent loads: /docs/spec/intake/INTAKE_CHECKLIST_AUTH.md
  
  Checklist includes:
  - [ ] Authentication method (OAuth, SAML, email/password, SSO)
  - [ ] Multi-factor authentication (required, optional, none)
  - [ ] Session management (JWT, cookies, server-side sessions)
  - [ ] Password policy (min length, complexity, expiration)
  - [ ] Password reset flow (email, SMS, security questions)
  - [ ] User provisioning (self-service, admin-only, automated)
  - [ ] Role-based access control (RBAC levels defined)
  - [ ] Audit logging requirements (login attempts, IP tracking)
  - [ ] Compliance requirements (GDPR, HIPAA, SOC2)
  - [ ] UI/UX requirements (branding, accessibility, mobile-responsive)
  - [ ] Deployment target (realm, AWS, multi-environment)
  - [ ] Cost budget (infrastructure, third-party services)

Step 2: Ask Questions Systematically
  Agent presents questions ONE AT A TIME (not all at once):
  
  Agent: "You said you need a login screen. Let's define the requirements."
  Agent: "Question 1 of 12: What authentication method?"
  Agent: "  A) OAuth (Google, Microsoft, GitHub)"
  Agent: "  B) Email + password (managed in-app)"
  Agent: "  C) SAML SSO (enterprise single sign-on)"
  Agent: "  D) Multiple methods (user chooses)"
  
  User: "Email and password"
  
  Agent: "✅ Recorded: Email + password authentication"
  Agent: "Question 2 of 12: Is multi-factor authentication required?"
  Agent: "  A) Required for all users"
  Agent: "  B) Optional (user can enable)"
  Agent: "  C) Not needed"
  
  User: "Optional"
  
  Agent: "✅ Recorded: MFA optional"
  Agent: "Question 3 of 12: How should sessions be managed?"
  ...

Step 3: Record Answers
  Agent writes to: /docs/spec/intake/login-screen-INTAKE.md
  
  Content:
  ```markdown
  # Login Screen - Intake Report
  
  **Requested by:** Alice
  **Date:** 2025-01-15
  **Work Path:** login-feature
  
  ## Requirements Gathered
  
  ### Authentication
  - **Method:** Email + password (managed in-app)
  - **Multi-factor auth:** Optional (user can enable)
  - **Session management:** JWT tokens (15-minute expiry, refresh token)
  - **Password policy:** Min 12 characters, complexity required
  - **Password reset:** Email-based reset flow
  
  ### User Provisioning
  - **Registration:** Self-service (user signs up)
  - **Email verification:** Required before first login
  - **Role assignment:** Default role "user", admin assigns other roles
  
  ### Compliance
  - **GDPR:** Required (data export, deletion, consent)
  - **Audit logging:** Log all login attempts, failed attempts, IP addresses
  
  ### UI/UX
  - **Branding:** Use RealmForge brand colors, logo
  - **Accessibility:** WCAG 2.1 AA required
  - **Mobile:** Responsive design required
  
  ### Deployment
  - **Target:** AWS (customer BYOC)
  - **Environment:** Production + staging
  
  ### Cost
  - **Budget:** $50/month for email service (SendGrid)
  - **Infrastructure:** Included in app hosting cost
  ```

Step 4: Identify Open Decisions
  Agent appends to: /docs/spec/22_OPEN_DECISIONS.md
  
  ```markdown
  ## DEC-USER-042: Login Screen - Session Duration
  
  **Status:** USER_APPROVAL_REQUIRED
  **Context:** JWT expiry set to 15 minutes. Refresh token expiry not specified.
  
  **Options:**
  A) Refresh token expires after 7 days (user must log in weekly)
  B) Refresh token expires after 30 days (user must log in monthly)
  C) Refresh token never expires (until revoked manually)
  
  **Recommendation:** Option B (30 days) balances security and convenience.
  
  **Blocked:** Cannot proceed to design until decision made.
  ```

Step 5: Check Workflow Gate
  Agent attempts: INTAKE → DESIGN transition
  Gate checks:
    ✅ All checklist questions answered? YES
    ❌ All open decisions closed? NO (DEC-USER-042 pending)
  Gate decision: **BLOCKED**
  
  Agent to user: "Intake complete! However, one decision remains:"
  Agent to user: "DEC-USER-042: How long should refresh tokens last?"
  Agent to user: "Please choose option A, B, or C."

Step 6: User Closes Decision
  User: "Option B - 30 days"
  
  Agent updates /docs/spec/22_OPEN_DECISIONS.md:
  ```markdown
  ## DEC-USER-042: Login Screen - Session Duration
  
  **Status:** ✅ **CLOSED**
  **Decision:** Option B - 30-day refresh token expiry
  **Decided by:** Alice
  **Date:** 2025-01-15
  ```

Step 7: Transition to Design Phase
  Agent attempts: INTAKE → DESIGN transition
  Gate checks:
    ✅ All checklist questions answered? YES
    ✅ All open decisions closed? YES
  Gate decision: **ALLOWED**
  
  Workflow transitions to DESIGN phase.
  Agent loses write access to /crates/* (still blocked)
  Agent gains write access to /docs/spec/login-screen/* (design docs)
```

### Why This Works

**Structural enforcement:**
* Agent CANNOT skip questions (checklist is mandatory)
* Agent CANNOT proceed to design with open decisions (gate blocks)
* Agent CANNOT write code without design approval (capability denied)

**No "hoping LLM is smart":**
* Checklist defines what questions to ask (not LLM's judgment)
* Gate defines what must be true to proceed (not LLM's interpretation)
* Policy defines what agent can write to (not LLM's discretion)

---

## The Design Agent: Structured Design Docs

### Capabilities
* ✅ READ: Intake docs, existing design patterns, architecture docs
* ✅ WRITE: Design docs (/docs/spec/<feature>/DESIGN.md)
* ❌ WRITE: Implementation code (still blocked by gate)

### Workflow
```
Step 1: Load Intake Report
  Agent reads: /docs/spec/intake/login-screen-INTAKE.md
  
Step 2: Load Design Templates
  Agent loads: /docs/spec/templates/AUTH_DESIGN_TEMPLATE.md
  
  Template includes:
  - Component diagram
  - API contracts
  - Database schema
  - Error handling
  - Security considerations
  - Cost estimate
  - Test strategy

Step 3: Generate Design Doc
  Agent writes: /docs/spec/login-screen/DESIGN.md
  
  Content:
  ```markdown
  # Login Screen - Design Document
  
  **Work Path:** login-feature
  **Owner:** Alice
  **Status:** DRAFT
  
  ## Architecture
  
  ### Components
  - Frontend: LoginForm (React component)
  - API: POST /api/auth/login (control-api crate)
  - Service: AuthService::login() (control-service crate)
  - Domain: UserCredential, Session (authority-domain crate)
  - Store: user_credentials table, sessions table (control-store crate)
  
  ### API Contract
  ```
  POST /api/auth/login
  Request:
  {
    "email": "user@example.com",
    "password": "...",
    "mfa_code": "123456"  // optional
  }
  
  Response (success):
  {
    "access_token": "eyJ...",
    "refresh_token": "...",
    "expires_in": 900,  // 15 minutes
    "user": {
      "id": "user-abc123",
      "email": "user@example.com",
      "roles": ["user"]
    }
  }
  
  Response (MFA required):
  {
    "mfa_required": true,
    "mfa_methods": ["email", "totp"]
  }
  
  Response (error):
  {
    "error": "invalid_credentials",
    "message": "Email or password incorrect"
  }
  ```
  
  ### Database Schema
  ```sql
  CREATE TABLE user_credentials (
    user_id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    mfa_enabled BOOLEAN DEFAULT FALSE,
    mfa_secret VARCHAR(255),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
  );
  
  CREATE TABLE sessions (
    session_id UUID PRIMARY KEY,
    user_id UUID REFERENCES user_credentials(user_id),
    access_token VARCHAR(512) NOT NULL,
    refresh_token VARCHAR(512) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    ip_address INET,
    user_agent TEXT
  );
  ```
  
  ### Security
  - Passwords hashed with Argon2id
  - Rate limiting: 5 failed attempts → 15-minute lockout
  - JWT signed with RS256 (private key in vault)
  - Audit log: All login attempts, successes, failures
  
  ### Cost Estimate
  - SendGrid (email service): $15/month (1,000 emails)
  - Database storage: <1GB (negligible)
  - Compute: Included in app hosting
  - **Total:** $15/month
  
  ### Test Strategy
  - Unit tests: AuthService::login() (success, failure, MFA)
  - Integration tests: API endpoint (valid, invalid, rate limit)
  - E2E tests: LoginForm (happy path, error states, MFA flow)
  ```

Step 4: Request Design Approval
  Agent creates: /docs/spec/login-screen/DESIGN_APPROVAL_REQUEST.md
  
  ```markdown
  # Login Screen - Design Approval Request
  
  **Status:** PENDING_APPROVAL
  **Requested by:** ai-agent-design-001
  **Date:** 2025-01-15
  
  **Design Document:** [DESIGN.md](DESIGN.md)
  **Intake Report:** [Intake](../intake/login-screen-INTAKE.md)
  
  **Summary:**
  - Email + password auth with optional MFA
  - JWT-based sessions (15-min access, 30-day refresh)
  - Argon2id password hashing
  - Rate limiting (5 attempts → 15-min lockout)
  - Cost: $15/month
  
  **Approval Required From:** Alice (work path owner)
  
  **Questions for Reviewer:**
  1. Is Argon2id acceptable, or prefer bcrypt/scrypt?
  2. Is 15-minute lockout acceptable, or should it be longer?
  3. Is SendGrid acceptable, or prefer AWS SES?
  
  **Approver Actions:**
  - [ ] Review design document
  - [ ] Answer questions above
  - [ ] Sign off: `APPROVED by [name] on [date]`
  ```
  
  Agent sends notification to Alice (work path owner)

Step 5: Human Reviews and Approves
  Alice reviews design doc, answers questions:
  
  Alice updates DESIGN_APPROVAL_REQUEST.md:
  ```markdown
  ## Approval Decision
  
  **Status:** ✅ **APPROVED**
  **Approved by:** Alice
  **Date:** 2025-01-15
  
  **Answers to Questions:**
  1. Argon2id is fine.
  2. 15-minute lockout is acceptable.
  3. SendGrid is fine, we already use it.
  
  **Comments:** Design looks good. Proceed to implementation.
  ```

Step 6: Workflow Gate Check
  Agent attempts: DESIGN → IMPLEMENTATION transition
  Gate checks:
    ✅ Design doc exists? YES
    ✅ Design approved? YES (Alice signed off)
    ✅ Open decisions closed? YES (all closed in intake)
  Gate decision: **ALLOWED**
  
  Workflow transitions to IMPLEMENTATION phase.
  Agent gains write access to /crates/* (capability granted by policy engine)
```

### Why This Works

**No code without sign-off:**
* Agent CANNOT transition to implementation until design approved (gate blocks)
* Approval must be EXPLICIT (`APPROVED by [name]` in doc)
* Gate checks signature, not just file existence

**Design is COMPLETE:**
* Template ensures all sections covered (no missing pieces)
* Cost estimate required (no "we'll figure it out later")
* Test strategy required (quality built in from start)

---

## The Implementation Agent: Constrained Coding

### Capabilities
* ✅ READ: Design docs, existing code, architecture docs
* ✅ WRITE: Implementation code (/crates/*, within blast radius)
* ❌ WRITE: Database migrations (requires separate approval)
* ❌ DEPLOY: (no deployment capability)

### Workflow
```
Step 1: Load Design Doc
  Agent reads: /docs/spec/login-screen/DESIGN.md
  
Step 2: Check Blast Radius
  Design specifies files to modify:
  - crates/control-api/src/routes/auth.rs (new file)
  - crates/control-service/src/auth_service.rs (new file)
  - crates/authority-domain/src/user_credential.rs (new file)
  - crates/control-store/src/user_store.rs (modified)
  - db/migrations/003_user_auth.sql (new file)
  
  Blast radius check:
    Files: 5
    New lines: ~800
    Cost estimate: $0.50 (LLM API cost for code generation)
  
  Agent's capability grant:
    Max files: 10 ✅
    Max lines: 1000 ✅
    Max cost: $1.00 ✅
  
  Blast radius: **WITHIN LIMITS**

Step 3: Generate Code File-by-File
  Agent creates files in sequence, following design doc exactly:
  
  File 1: crates/authority-domain/src/user_credential.rs
  ```rust
  // Generated by ai-agent-impl-001
  // Design doc: /docs/spec/login-screen/DESIGN.md
  // Work path: login-feature
  
  use uuid::Uuid;
  use serde::{Deserialize, Serialize};
  
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct UserCredential {
      pub user_id: UserId,
      pub email: Email,
      pub password_hash: PasswordHash,
      pub mfa_enabled: bool,
      pub mfa_secret: Option<String>,
  }
  
  impl UserCredential {
      pub fn verify_password(&self, password: &str) -> Result<(), AuthError> {
          argon2::verify_encoded(&self.password_hash.0, password.as_bytes())
              .map_err(|_| AuthError::InvalidCredentials)
      }
  }
  ```
  
  ... (generates remaining files)

Step 4: Run Tests
  Agent attempts to run tests:
  `cargo test --package control-service --test auth_service_tests`
  
  Tests fail:
  ```
  test auth_service::login_with_valid_credentials ... FAILED
  test auth_service::login_with_invalid_credentials ... FAILED
  test auth_service::login_rate_limiting ... FAILED
  ```
  
  Agent reads failure messages, fixes bugs, re-runs tests.
  
  After 3 iterations, tests pass:
  ```
  test result: ok. 15 passed; 0 failed
  ```

Step 5: Request Code Review
  Agent creates: /docs/spec/login-screen/CODE_REVIEW_REQUEST.md
  
  ```markdown
  # Login Screen - Code Review Request
  
  **Status:** PENDING_REVIEW
  **Implemented by:** ai-agent-impl-001
  **Date:** 2025-01-15
  
  **Design Document:** [DESIGN.md](DESIGN.md)
  
  **Files Changed:**
  - crates/control-api/src/routes/auth.rs (new, 150 lines)
  - crates/control-service/src/auth_service.rs (new, 200 lines)
  - crates/authority-domain/src/user_credential.rs (new, 100 lines)
  - crates/control-store/src/user_store.rs (modified, +180 lines)
  - db/migrations/003_user_auth.sql (new, 50 lines)
  
  **Tests:**
  - Unit tests: 10 passed
  - Integration tests: 5 passed
  - Total coverage: 87%
  
  **Review Checklist:**
  - [ ] Follows design doc exactly?
  - [ ] No security vulnerabilities?
  - [ ] Error handling complete?
  - [ ] Tests comprehensive?
  - [ ] No `unwrap()` without comment?
  - [ ] Logging appropriate?
  
  **Reviewer:** Owen (Code Review skill)
  ```

Step 6: Workflow Gate Check
  Agent attempts: IMPLEMENTATION → CODE_REVIEW transition
  Gate checks:
    ✅ Code files created? YES
    ✅ Tests passing? YES (15/15 passed)
    ✅ Code review requested? YES
  Gate decision: **ALLOWED**
  
  Workflow transitions to CODE_REVIEW phase.
  Agent loses write access to /crates/* (no more changes until review passes)
```

### Why This Works

**Blast radius enforcement:**
* Agent CANNOT modify more files than grant allows (policy blocks)
* Agent CANNOT skip tests (workflow gate requires passing tests)
* Agent CANNOT merge without code review (gate blocks deployment)

**Code follows design:**
* Design doc is the CONTRACT (agent implements what's specified)
* Deviation from design requires new approval (can't improvise)
* Test strategy from design ensures quality

---

## Self-Improvement: Agents Learn from Failures

### The Learning Loop

**Every work path execution generates learning data:**

```rust
// control-service/src/learning.rs
pub struct WorkPathExecution {
    pub work_path: WorkPath,
    pub phases: Vec<PhaseExecution>,
    pub outcome: Outcome,
    pub lessons: Vec<Lesson>,
}

pub struct PhaseExecution {
    pub phase: Phase,
    pub agent_id: AgentId,
    pub duration: Duration,
    pub iterations: usize,  // How many tries before success
    pub errors: Vec<ExecutionError>,
    pub cost: Cost,
}

pub struct Lesson {
    pub phase: Phase,
    pub failure_type: FailureType,
    pub root_cause: String,
    pub fix_applied: String,
    pub prevention: String,
}

pub enum FailureType {
    MissingRequirement,  // Intake incomplete
    DesignFlaw,          // Design didn't account for edge case
    ImplementationBug,   // Code didn't match design
    TestFailure,         // Tests revealed issue
    DeploymentFailure,   // Production deployment failed
}
```

### Example: Agent Learns from Login Screen Failure

**Scenario:** Agent implemented login screen, deployed to production, rate limiting didn't work, users got locked out forever.

**Post-Mortem:**
```markdown
# Login Screen - Post-Mortem

**Work Path:** login-feature
**Deployment Date:** 2025-01-15
**Incident Date:** 2025-01-16
**Impact:** 50 users locked out for 24 hours

## What Went Wrong

**Root Cause:** Rate limiting implementation used in-memory cache. When app restarted, lockout state was lost. Users could retry immediately.

**Design Flaw:** Design doc didn't specify lockout persistence (assumed in-memory was fine).

**Intake Gap:** Intake checklist didn't ask: "Should rate limiting survive app restarts?"

## Lessons Learned

1. **Intake Phase:**
   - ADD to checklist: "Should rate limiting state persist across restarts?"
   - ADD to checklist: "What happens if app crashes during lockout period?"

2. **Design Phase:**
   - ADD to template: "State persistence section" (what state must survive restarts?)
   - ADD to template: "Failure scenarios" (what happens if X crashes/restarts/fails?)

3. **Implementation Phase:**
   - ADD to code review checklist: "Is critical state persisted?"
   - ADD to test strategy: "Test app restart scenarios"

## Remediation

1. Update /docs/spec/intake/INTAKE_CHECKLIST_AUTH.md
2. Update /docs/spec/templates/AUTH_DESIGN_TEMPLATE.md
3. Update code review checklist
4. Re-design login screen with persistent rate limiting
5. Deploy fix
```

### Checklist Update (Self-Improvement)

**Before (missing question):**
```markdown
## Intake Checklist: Authentication

- [ ] Authentication method (OAuth, SAML, email/password, SSO)
- [ ] Multi-factor authentication (required, optional, none)
- [ ] Session management (JWT, cookies, server-side sessions)
- [ ] Password policy (min length, complexity, expiration)
- [ ] Rate limiting (enabled, disabled)
```

**After (learned question added):**
```markdown
## Intake Checklist: Authentication

- [ ] Authentication method (OAuth, SAML, email/password, SSO)
- [ ] Multi-factor authentication (required, optional, none)
- [ ] Session management (JWT, cookies, server-side sessions)
- [ ] Password policy (min length, complexity, expiration)
- [ ] Rate limiting (enabled, disabled)
- [ ] Rate limiting persistence (in-memory, database, Redis)
- [ ] Rate limiting reset behavior (on app restart, preserved)
- [ ] Lockout recovery (manual admin unlock, automatic after N hours)
```

**Result:**
* Next time agent does intake for ANY auth feature, it asks these questions
* Prevents same mistake in future login screens, password reset, MFA, etc.
* **System gets smarter from failures**

---

## The Control Plane Architecture

### How All the Pieces Fit Together

```
┌─────────────────────────────────────────────────────────────────┐
│                    RealmForge Control Plane                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────┐     ┌─────────────────┐                   │
│  │  User Request   │────→│  Intake Agent   │                   │
│  │  "Login screen" │     │  (guided q's)   │                   │
│  └─────────────────┘     └────────┬────────┘                   │
│                                    │                             │
│                           ┌────────▼─────────┐                  │
│                           │ Intake Complete  │                  │
│                           │ All Q's answered │                  │
│                           │ Open decisions   │                  │
│                           │ closed           │                  │
│                           └────────┬─────────┘                  │
│                                    │                             │
│                    ┌───────────────▼───────────────┐            │
│                    │ Workflow Gate: INTAKE→DESIGN  │            │
│                    │ Checks:                       │            │
│                    │  ✅ Checklist complete        │            │
│                    │  ✅ No open decisions         │            │
│                    │ Decision: ALLOWED             │            │
│                    └───────────────┬───────────────┘            │
│                                    │                             │
│                           ┌────────▼─────────┐                  │
│                           │  Design Agent    │                  │
│                           │  (uses template) │                  │
│                           └────────┬─────────┘                  │
│                                    │                             │
│                           ┌────────▼─────────┐                  │
│                           │ Design Complete  │                  │
│                           │ Request approval │                  │
│                           └────────┬─────────┘                  │
│                                    │                             │
│                    ┌───────────────▼────────────────┐           │
│                    │ Workflow Gate: DESIGN→IMPL     │           │
│                    │ Checks:                        │           │
│                    │  ✅ Design doc exists          │           │
│                    │  ❌ Design approved? NO        │           │
│                    │ Decision: BLOCKED              │           │
│                    │ Waiting for: Human approval    │           │
│                    └───────────────┬────────────────┘           │
│                                    │                             │
│                           ┌────────▼─────────┐                  │
│                           │  Human Reviews   │                  │
│                           │  Signs off       │                  │
│                           └────────┬─────────┘                  │
│                                    │                             │
│                    ┌───────────────▼───────────────┐            │
│                    │ Workflow Gate: DESIGN→IMPL    │            │
│                    │ Checks:                       │            │
│                    │  ✅ Design doc exists         │            │
│                    │  ✅ Design approved? YES      │            │
│                    │ Decision: ALLOWED             │            │
│                    └───────────────┬───────────────┘            │
│                                    │                             │
│                           ┌────────▼─────────┐                  │
│                           │  Impl Agent      │                  │
│                           │  (writes code)   │                  │
│                           └────────┬─────────┘                  │
│                                    │                             │
│                           ┌────────▼─────────┐                  │
│                           │ Code Complete    │                  │
│                           │ Tests passing    │                  │
│                           │ Request review   │                  │
│                           └────────┬─────────┘                  │
│                                    │                             │
│                    ┌───────────────▼──────────────┐             │
│                    │ Workflow Gate: IMPL→REVIEW   │             │
│                    │ Checks:                      │             │
│                    │  ✅ Tests passing           │             │
│                    │  ✅ Blast radius OK         │             │
│                    │ Decision: ALLOWED            │             │
│                    └───────────────┬──────────────┘             │
│                                    │                             │
│                           ┌────────▼─────────┐                  │
│                           │  Code Review     │                  │
│                           │  (human or AI)   │                  │
│                           └────────┬─────────┘                  │
│                                    │                             │
│                                   ...                            │
│                                    │                             │
│                           ┌────────▼─────────┐                  │
│                           │  Deployment      │                  │
│                           │  (realm or AWS)  │                  │
│                           └──────────────────┘                  │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘

At every step:
- Policy Engine checks agent capabilities
- Workflow Engine enforces gate requirements
- Audit Log records every action
- Cost Tracker accumulates spend
- Learning System captures lessons
```

---

## Implementation Priorities

### Phase 1: Foundation (Weeks 1-4)
1. **Policy Engine** (`policy-engine` crate)
   - Capability grants model
   - Permission checking
   - Blast radius calculation
   - Audit logging integration

2. **Workflow Engine** (`control-service` crate)
   - Workflow state model
   - Gate definitions
   - Phase transitions
   - Requirement checking

3. **Agent Identity** (`authority-domain` crate)
   - AgentId, AgentCredential types
   - Agent registration
   - Credential management

### Phase 2: Intake System (Weeks 5-8)
4. **Intake Checklists**
   - Auth checklist
   - CRUD checklist
   - API checklist
   - UI checklist

5. **Intake Agent** (control-service + LLM integration)
   - Load checklist
   - Ask questions systematically
   - Record answers
   - Identify open decisions
   - Check gate requirements

6. **Open Decisions Register** (`docs/spec/22_OPEN_DECISIONS.md`)
   - Decision states (USER_APPROVAL_REQUIRED, COUNCIL_DECISION_REQUIRED, CLOSED)
   - Decision tracking
   - Blocker detection

### Phase 3: Design System (Weeks 9-12)
7. **Design Templates**
   - Auth design template
   - CRUD design template
   - API design template
   - UI design template

8. **Design Agent** (control-service + LLM integration)
   - Load intake report
   - Load design template
   - Generate design doc
   - Request approval
   - Check gate requirements

9. **Approval Workflow** (control-service)
   - Approval request model
   - Human notification
   - Approval signature validation
   - Gate unblocking

### Phase 4: Implementation System (Weeks 13-16)
10. **Implementation Agent** (control-service + LLM integration)
    - Load design doc
    - Check blast radius
    - Generate code file-by-file
    - Run tests
    - Request code review

11. **Code Review Integration** (control-service)
    - Code review request model
    - Review checklist validation
    - Human or AI reviewer
    - Approval/rejection handling

12. **Test Enforcement** (control-service)
    - Test execution
    - Coverage tracking
    - Gate blocking on test failure

### Phase 5: Learning System (Weeks 17-20)
13. **Execution Recording** (`audit` crate)
    - WorkPathExecution model
    - PhaseExecution tracking
    - Error recording
    - Cost accumulation

14. **Lesson Extraction** (control-service)
    - Post-mortem analysis
    - Root cause identification
    - Checklist updates
    - Template updates

15. **Self-Improvement Loop** (control-service)
    - Automatic checklist updates
    - Template versioning
    - Agent retraining triggers

---

## Success Criteria

**The agent system is successful when:**

1. ✅ **No code without design approval**
   - Agents CANNOT write code files before design is signed off
   - Workflow gate BLOCKS transition (not just warns)

2. ✅ **No hallucinations in requirements**
   - Intake checklist FORCES all questions to be asked
   - Agent CANNOT skip questions (workflow blocks)

3. ✅ **No security vulnerabilities from agents**
   - Policy engine DENIES actions outside capability grants
   - Blast radius LIMITS damage from agent mistakes

4. ✅ **Complete audit trail**
   - Every agent action LOGGED (who, what, when, why)
   - Every approval SIGNED (human signature required)

5. ✅ **Self-improving checklists**
   - Post-mortems UPDATE checklists
   - Future work paths BENEFIT from past failures

6. ✅ **Predictable costs**
   - Every agent action HAS cost estimate
   - Cumulative cost TRACKED per work path
   - Budget overruns BLOCKED by policy

7. ✅ **Blast radius containment**
   - Agent CANNOT modify more files than grant allows
   - Large changes REQUIRE explicit approval
   - Rollback ALWAYS possible (snapshot-based)

---

## Conclusion

**This is NOT about making LLMs smarter.**

**This is about:**
* Structural enforcement (gates, policies, capabilities)
* Guided workflows (checklists, templates, approval flows)
* Self-improvement (learning from failures)
* Audit trails (every action logged)
* Cost control (budgets enforced)

**RealmForge's thesis:**
> "Git made human collaboration scalable. RealmForge makes AI execution governable."

**The control plane for AI agents is:**
* Not a prompt
* Not a fine-tuned model
* Not a RAG pipeline

**It's a workflow engine + policy engine + audit log + cost tracker + learning system.**

**And THAT is what makes agents safe to use in production.**

---

**Status:** 🚨 **DESIGN COMPLETE** - Ready for architecture review and implementation planning

**Related Documents:**
* [DEPLOYMENT_MODEL_DECISION.md](DEPLOYMENT_MODEL_DECISION.md) - Deployment architecture
* [COMPLETE_LIFECYCLE_ANALYSIS.md](COMPLETE_LIFECYCLE_ANALYSIS.md) - Full lifecycle gaps
* [00_LANDSCAPE_ANALYSIS_INDEX.md](00_LANDSCAPE_ANALYSIS_INDEX.md) - Analysis index
