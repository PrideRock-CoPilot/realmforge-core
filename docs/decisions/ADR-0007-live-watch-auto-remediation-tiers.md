# ADR-0007: Live Watch Auto-Remediation Tiered Framework

**Date:** 2026-05-05
**Status:** Accepted
**Deciders:** Council (with User Approval)
**Consulted:** Fatima Al-Hassan (Security Architect), Nadia Kovacs (Infrastructure Architect), Dmitri Volkov (Backend), Sam Osei (Release Manager)

## Context

Live Watch (Phase 8) monitors runtime behavior and detects anomalies. The question is whether it should ever take automated action to remediate issues it detects, and if so, under what constraints.

The design must balance operational responsiveness against the risk of unintended automated actions causing harm.

## Decision Drivers

- Safety: automated remediation must not cause more harm than the detected issue
- Observability: every automated action must be auditable and reversible
- Opt-in: operators must explicitly enable any auto-remediation; default must be safe
- Bounded blast radius: any automated action must be limited in scope and frequency
- Audit trail: all actions must produce audit events and snapshot anchors

## Options Considered

### Option A: Proposals Only — No Auto-Remediation (Tier 0)

Live Watch detects and reports issues but never takes action. Operators review proposals and act manually.

- Pro: Zero risk of unintended automated action
- Con: Slower response to known, low-severity, well-understood issues

### Option B: Full Auto-Remediation

Live Watch detects and remediates all severity levels automatically.

- Pro: Fastest response time
- Con: Unacceptable risk of cascading failures from automated actions on complex issues

### Option C: Tiered Framework (Selected)

A multi-tier approach where the default is proposals only, and operators can opt-in to increasingly capable auto-remediation tiers.

- Pro: Safe default (Tier 0), bounded escalation path
- Pro: Operators control the risk profile per application
- Pro: Each tier can be validated in isolation before enabling the next

## Decision

Live Watch adopts a tiered auto-remediation framework:

- **Tier 0 (Default):** Reports and suggests only. No automated remediation. The watcher makes no changes — it only reports anomalies and suggests remediation steps.
- **Tier 1 (Opt-in per app):** Pre-approved, low-severity actions with bounded blast radius. Requires explicit operator opt-in per application.
- **Tiers 2-3:** Deferred until Tier 1 has been proven safe in production for at least one release cycle.

## Rationale

Tier 0 is the safe default: the watcher never acts without human approval unless explicitly configured to do so. Tier 1 provides a controlled path for automating well-understood, low-risk remediations. Deferring Tiers 2-3 ensures that automated remediation capability is validated incrementally. Every Tier 1 action requires explicit operator opt-in per app, ensuring that automation does not spread silently across the fleet.

## Consequences

**Positive:**
- Zero risk of unintended remediation unless operator explicitly opts in
- Incremental path to automation with bounded blast radius at each tier
- Clear security boundary: Tier 0 requires no additional threat model

**Negative / Trade-offs:**
- Tier 1 requires per-app operator configuration (operational overhead)
- Teams expecting full auto-remediation must wait for Tiers 2-3 validation

**Risks:**
- Operators may enable Tier 1 without understanding blast radius bounds (mitigated by mandatory configuration fields and per-action limits)

## Implementation Plan

1. Tier 0 is already the default behavior — Live Watch proposes, does not act
2. Tier 1 implementation in a later release: add opt-in configuration per app, define low-severity action catalog, set per-action max/day limits
3. Each Tier 1 action must: produce audit events, create snapshot anchors, support per-app disable
4. Tiers 2+ deferred; each requires a new Council decision before implementation begins

## Review Date

2027-05-05, or earlier if a production incident demonstrates an urgent need for Tier 1 auto-remediation.
