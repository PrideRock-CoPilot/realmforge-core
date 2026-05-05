# Phase 2a Gaps Identified

Gap 1: `scope.rs` Default impl uses `unwrap()` without INVARIANT comment
Gap 2: No `SnapshotDelta` for manifest diffing
Gap 3: `evaluate_policy_chain` short-circuits (plan says collect all denials)
Gap 4: No `stat()` method on object store for individual object metadata
Gap 5: No public `can_transition_to()` on CommandStatus
Gap 6: skill_creator.rs needs version-increment operations
Gap 7: Some files may exceed 300 lines
