//! # rfsource-format
//!
//! Binary frame format for `.rfsource` files.
//!
//! ## Format Overview
//!
//! Each `.rfsource` file is an append-only stream of compressed frames:
//!
//! ```text
//! MAGIC HEADER (8 bytes: RFSOURCE\x00\x02\n)
//! Frame 1:
//!   [1 byte flag] [8 bytes uncompressed_len] [8 bytes stored_len] [compressed payload]
//! Frame 2:
//!   ...
//! Frame N:
//!   ...
//! ```
//!
//! ## Crate Law
//!
//! - Reader/writer for `.rfsource` binary frames only
//! - No business logic, no state machine, no policy
//! - Frames are opaque byte payloads to this layer

pub mod error;
pub mod frame;

pub use error::FormatError;
pub use frame::*;
