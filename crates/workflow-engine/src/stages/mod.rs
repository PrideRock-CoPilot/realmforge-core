//! # Stage Agent Implementations
//!
//! Each lifecycle stage in the focused workflow has its own agent struct
//! implementing the `StageAgent` trait. These agents own their stage-specific
//! validation logic and exit gate enforcement.

pub mod architecture;
pub mod code_review;
pub mod council;
pub mod design;
pub mod development;
pub mod documentation;
pub mod idea;
pub mod peer_review;
pub mod planning;
pub mod testing;
