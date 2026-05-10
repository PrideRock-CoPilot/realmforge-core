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
//! ## Multi-File Support (DDR-003)
//!
//! For repositories larger than 1.5 GB, RFSource uses fixed-size segments:
//!
//! ```text
//! my-repo/
//! ├── .rfsource.0        # First segment (1 GB)
//! ├── .rfsource.1        # Second segment (1 GB)
//! ├── .rfsource.2        # Active segment
//! └── .rfsource.manifest # Segment metadata
//! ```
//!
//! See `/docs/design/DDR-003-MULTI-FILE-DESIGN.md` for full design.
//!
//! ## Crate Law
//!
//! - Reader/writer for `.rfsource` binary frames only
//! - No business logic, no state machine, no policy
//! - Frames are opaque byte payloads to this layer

pub mod error;
pub mod frame;
pub mod manifest;
pub mod footer;
pub mod checksum;

pub use error::FormatError;
pub use frame::*;
pub use manifest::{Manifest, SegmentInfo, ArchivedSegmentInfo};
pub use footer::SegmentFooter;
pub use checksum::{SegmentChecksum, ChecksumWriter, ChecksumReader, ChecksumError, checksum_file, verify_file};
