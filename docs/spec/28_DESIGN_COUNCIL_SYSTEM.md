---
doc_id: DOC-SPEC-028
title: "Design Council System — Evidence-Driven Design Research Engine"
status: draft
owner: design-council
reviewers: [cto, pm, domain-architect, security-architect, api-architect, data-architect, tech-writer]
created_at: 2026-05-07
last_reviewed_at: 2026-05-07
source_of_truth: true
product_area: design-research
work_path_ids: [WP-DESIGN-001]
related_decision_ids: []
related_file_ids: [FILE-DESIGN-COUNCIL-LIB]
visual_node_ids: [VN-DESIGN-COUNCIL]
visual_edge_ids: []
approval_state: pending
---

# Design Council System — Evidence-Driven Design Research Engine

## Purpose

The Design Council system codifies the Design Council protocol as a Rust crate. It provides
the types, state machines, and validation logic that represent a design research session:
from problem framing through example gathering, pattern extraction, recommendation synthesis,
risk surfacing, and design brief production.

This is the **engineering artifact** of the `/design-council` skill. The skill is the
organizational persona and process; this crate is the data model that makes Design Council
outputs structured, queryable, versioned, and reusable.

## System Boundary

```
┌─────────────────────────────────────────────────────────────┐
│                    Design Council System                      │
│                                                              │
│  design-council crate (pure types, logic, validation)        │
│  - DesignProblem                                             │
│  - ResearchExample, ExampleVerdict                           │
│  - PatternExtraction, DesignPattern                          │
│  - DesignRecommendation, RecommendationRank                  │
│  - RiskSurface, BlindSpot                                    │
│  - DesignBrief (aggregate root)                              │
│  - DesignBriefStatus state machine                           │
│                                                              │
│  NO I/O — no database, no HTTP, no file access              │
│  PURE TYPES — depends on serde, chrono for timestamps       │
└─────────────────────────────────────────────────────────────┘
```

## Layer Position

The `design-council` crate lives at the **domain layer**, same as `authority-domain`:

```
design-council (pure types, no IO)
      ↓
control-service (orchestrates sessions, persists briefs via control-store)
      ↓
control-store (persists design briefs to PostgreSQL)
```

- `design-council` MUST NOT depend on any other RealmForge crate
- `control-service` MAY import `design-council` for session orchestration
- `control-store` MAY store serialized `DesignBrief` records

## Types

### DesignProblemId
A typed newtype wrapping `Uuid`. Identifies a unique design problem study.

### DesignProblem

```rust
pub struct DesignProblem {
    pub id: DesignProblemId,
    pub problem_statement: String,         // Single sentence
    pub system_boundary: String,           // What subsystem this affects
    pub constraints: Vec<String>,          // Performance, security, compatibility
    pub success_signal: String,            // What "good" looks like
    pub non_goals: Vec<String>,            // Explicitly out of scope
    pub created_at: DateTime<Utc>,
}

```

### ExampleVerdict

```rust
pub enum ExampleVerdict {
    Success,
    Failure,
    Mixed,
}
```

### ResearchExample

```rust
pub struct ResearchExample {
    pub system_name: String,               // System that faced this problem
    pub context_description: String,       // System context (scale, domain, tech stack)
    pub approach_taken: String,            // What they did
    pub verdict: ExampleVerdict,           // Did it work?
    pub deciding_factors: Vec<String>,     // Why it worked or didn't
    pub relevant_constraints: Vec<String>, // Scale, team, regulatory, etc.
    pub key_takeaway: String,              // What we can learn
    pub source_type: ResearchSourceType,
}
```

### ResearchSourceType

```rust
pub enum ResearchSourceType {
    OpenSourceProject,
    PublishedPostMortem,
    IndustryPattern,
    AcademicPaper,
    PastRealmForgeDecision,
    InternalPostMortem,
}
```

### DesignPattern

```rust
pub struct DesignPattern {
    pub description: String,               // The pattern statement
    pub pattern_type: PatternType,
    pub supporting_examples: Vec<String>,  // References to ResearchExample system names
    pub context_notes: Option<String>,     // When this pattern applies or doesn't
}
```

### PatternType

```rust
pub enum PatternType {
    Success,    // What successful approaches shared
    Failure,    // What failures shared
    Contextual, // Worked only under specific conditions
}
```

### PatternExtraction

```rust
pub struct PatternExtraction {
    pub success_patterns: Vec<DesignPattern>,
    pub failure_patterns: Vec<DesignPattern>,
    pub contextual_factors: Vec<DesignPattern>,
}
```

### RecommendationRank

```rust
pub enum RecommendationRank {
    Primary,
    Alternative,
}
```

### DesignRecommendation

```rust
pub struct DesignRecommendation {
    pub name: String,                      // Short descriptive label
    pub description: String,               // What the approach is
    pub rank: RecommendationRank,
    pub supporting_evidence: Vec<String>,  // Which examples and patterns back this
    pub risk_level: RiskLevel,
    pub risks: Vec<String>,                // Specific risks referencing failure examples
    pub mitigations: Vec<String>,          // How to address the risks
    pub combination_notes: Option<String>, // If combining elements from multiple examples
}
```

### RiskLevel

```rust
pub enum RiskLevel {
    Low,
    Medium,
    High,
}
```

### BlindSpot

```rust
pub struct BlindSpot {
    pub description: String,               // What we didn't study or assumed
    pub spot_type: BlindSpotType,
    pub potential_impact: String,          // What could go wrong
}
```

### BlindSpotType

```rust
pub enum BlindSpotType {
    UnstudiedSystem,
    ImplicitAssumption,
    DisprovingEvidence,
    ReversalCost,
}
```

### RiskSurface

```rust
pub struct RiskSurface {
    pub unstudied_systems: Vec<String>,       // What we didn't study
    pub assumptions: Vec<String>,             // Implicit assumptions
    pub disproving_evidence: Vec<String>,     // What would disprove the recommendation
    pub reversal_cost_estimate: String,       // Cost of being wrong
    pub blind_spots: Vec<BlindSpot>,
}
```

### DesignBriefStatus

```rust
pub enum DesignBriefStatus {
    Draft,            // Being researched
    Recommendation,   // Research phase complete, brief produced
    Accepted,         // Decision made based on this brief
    Superseded,       // Newer research has replaced this
    Archived,         // Retained for historical reference
}
```

### DesignBrief (Aggregate Root)

```rust
pub struct DesignBrief {
    pub id: DesignProblemId,
    pub title: String,
    pub session_date: DateTime<Utc>,
    pub participants: Vec<String>,           // Skills or individuals convened
    pub design_problem: DesignProblem,
    pub research_examples: Vec<ResearchExample>,
    pub pattern_extraction: Option<PatternExtraction>,
    pub recommendations: Vec<DesignRecommendation>,
    pub risk_surface: Option<RiskSurface>,
    pub adr_reference: Option<String>,       // Linked ADR after decision
    pub status: DesignBriefStatus,
}
```

## State Machine

```
Draft ───→ Recommendation ───→ Accepted
               │                    │
               │                    ├──→ Superseded
               │                    └──→ Archived
               └──→ Archived
```

Valid transitions enforced by `DesignBrief::transition_to()`:

1. Draft → Recommendation (research complete, brief produced)
2. Recommendation → Accepted (decision made to adopt)
3. Recommendation → Archived (decision was not to adopt; brief retained for reference)
4. Accepted → Superseded (new research replaced this)
5. Accepted → Archived (decision no longer relevant but retained)

Invalid transitions that return an error:
- Draft → Accepted (must go through Recommendation)
- Recommendation → Draft (cannot reopen closed research)
- Accepted → Recommendation (cannot revise after decision)

## Validation Rules

A `DesignBrief` is valid when:

1. `design_problem.problem_statement` is non-empty and ≤ 200 characters
2. `research_examples.len() >= 6` (the six-example minimum)
3. At least 2 research examples have verdict `Failure` (unless documented gap)
4. If `pattern_extraction` is present, it must have at least one pattern
5. If `recommendations` is non-empty, each must name at least one supporting evidence item
6. If `risk_surface` is present, assumptions must be non-empty (at least one named)
7. Status transitions must follow the valid state machine

These rules are enforced by `DesignBrief::validate()` which returns a `Result<(), DesignCouncilError>`.

## DesignCouncilError

```rust
pub enum DesignCouncilError {
    InsufficientExamples { count: usize, minimum: usize },
    InsufficientFailures { failure_count: usize, minimum: usize },
    EmptyProblemStatement,
    ProblemStatementTooLong { length: usize, max: usize },
    NoPatternsProvided,
    RecommendationWithoutEvidence { recommendation: String },
    RiskSurfaceWithoutAssumptions,
    InvalidStatusTransition { from: DesignBriefStatus, to: DesignBriefStatus },
    DuplicateExampleName { system_name: String },
    EmptyRecommendations,
}
```

## Future Extension Points

1. **Persistence** — `control-store` will gain a `design_briefs` table with serialized JSONB
2. **Service layer** — `control-service` will gain a `design_council_service.rs` with:
   - `create_design_problem()`
   - `add_research_examples()`
   - `extract_patterns()`
   - `produce_recommendations()`
   - `finalize_brief()`
3. **API surface** — `/v1/design-briefs` REST endpoints
4. **MCP tools** — `core_design_brief` tools for agent invocation
5. **Brief search** — Query past design briefs by problem, pattern, or recommendation
6. **Knowledge integration** — Auto-suggest existing briefs for new design problems

## Validation Gate

- `cargo check -p design-council` compiles with no warnings
- All types implement `Debug`, `Clone`, `PartialEq`
- `DesignBrief::validate()` correctly rejects invalid briefs and accepts valid ones
- Status transition tests cover all valid and invalid paths
- All errors return typed `DesignCouncilError` variants
