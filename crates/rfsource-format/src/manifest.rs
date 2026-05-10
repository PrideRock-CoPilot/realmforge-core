//! Manifest format for multi-file `.rfsource` repositories.
//!
//! ## Overview
//!
//! The manifest file (`.rfsource.manifest`) tracks all segments in a multi-file
//! repository. It enables O(log N) query routing by maintaining metadata about
//! which frames are in which segments.
//!
//! ## File Location
//!
//! The manifest lives alongside segment files:
//! ```text
//! my-repo/
//! ├── .rfsource.0        # First segment
//! ├── .rfsource.1        # Second segment
//! ├── .rfsource.2        # Active segment
//! └── .rfsource.manifest # This file
//! ```
//!
//! ## Design (DDR-003)
//!
//! - **Fixed 1 GB segment size** - PostgreSQL-inspired segmentation
//! - **Sequential naming** - `.rfsource.0`, `.1`, `.2`, etc.
//! - **Manifest for fast queries** - O(log N) frame lookup
//! - **Per-segment footers** - Enables manifest rebuild if lost
//! - **BLAKE3 checksums** - Corruption detection
//! - **Git-gc-style compaction** - Manual merge of closed segments
//!
//! See: `/docs/design/DDR-003-MULTI-FILE-DESIGN.md`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{FormatError, Result};

/// Segment size threshold for rotation: 1 GB.
///
/// When the active segment exceeds this size, it is closed and a new
/// segment is created. This matches the proven PostgreSQL pattern.
pub const SEGMENT_SIZE_THRESHOLD: u64 = 1_073_741_824; // 1 GB

/// Maximum segments before compaction recommended: 500 (~500 GB).
///
/// This is a soft limit for monitoring. Repos exceeding this should
/// consider running `rfsource-cli compact` to merge closed segments.
pub const SEGMENT_COUNT_WARNING: usize = 500;

/// Critical segment count threshold: 1,000 (~1 TB).
///
/// Repos exceeding this count risk filesystem performance degradation
/// and must run compaction.
pub const SEGMENT_COUNT_CRITICAL: usize = 1000;

/// Manifest for a multi-file `.rfsource` repository.
///
/// The manifest is the primary index for fast queries. It tracks all
/// segments, archived segments, and compaction metadata.
///
/// ## Persistence
///
/// The manifest is serialized to JSON and stored as `.rfsource.manifest`.
/// It is updated atomically on segment rotation and compaction.
///
/// ## Recovery
///
/// If the manifest is lost or corrupted, it can be rebuilt from per-segment
/// footers using `rfsource-cli rebuild-manifest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest format version (currently 1).
    pub version: u32,

    /// All active segments in frame order.
    ///
    /// Segments are stored in ascending order by `first_frame_id`.
    /// The last segment is the active (writable) segment.
    pub segments: Vec<SegmentInfo>,

    /// Archived segments after compaction.
    ///
    /// Archived segments are renamed (e.g., `.rfsource.0.archived`) and
    /// tracked here for audit purposes. They can be restored if needed.
    #[serde(default)]
    pub archived_segments: Vec<ArchivedSegmentInfo>,

    /// Compaction generation counter.
    ///
    /// Incremented each time `compact` is run. Used for debugging and
    /// monitoring compaction frequency.
    #[serde(default)]
    pub compaction_generation: u32,

    /// Total number of frames across all segments.
    pub total_frames: u64,

    /// Total size of all segments in bytes.
    pub total_size_bytes: u64,
}

/// Metadata for a single segment file.
///
/// Each segment is a self-contained `.rfsource` file with a footer
/// containing this metadata (duplicated for resilience).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// Segment index (sequential: 0, 1, 2, ...).
    pub index: u32,

    /// Relative path to segment file (e.g., `.rfsource.0`).
    pub path: String,

    /// Segment file size in bytes.
    pub size_bytes: u64,

    /// Number of frames in this segment.
    pub frame_count: u64,

    /// First frame ID in this segment (inclusive).
    pub first_frame_id: u64,

    /// Last frame ID in this segment (inclusive).
    pub last_frame_id: u64,

    /// Whether this is a compacted segment.
    ///
    /// Compacted segments have names like `.rfsource.compact.0` and
    /// represent merged closed segments.
    #[serde(default)]
    pub is_compacted: bool,

    /// Original segment indices merged into this segment (if compacted).
    ///
    /// Empty for non-compacted segments.
    #[serde(default)]
    pub compacted_from: Vec<u32>,

    /// BLAKE3 checksum of segment data (hex string).
    ///
    /// None for active (open) segments, Some for closed segments.
    /// Used for corruption detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_blake3: Option<String>,

    /// When this segment was created.
    pub created_at: DateTime<Utc>,

    /// When this segment was closed (rotated to new segment).
    ///
    /// None for the active segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
}

/// Metadata for an archived segment.
///
/// After compaction, original segments are renamed to `.archived` and
/// tracked here. They can be restored with `rfsource-cli restore-archived`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedSegmentInfo {
    /// Original segment index before compaction.
    pub original_index: u32,

    /// Path to archived file (e.g., `.rfsource.0.archived`).
    pub archived_path: String,

    /// Index of the compacted segment this was merged into.
    pub compacted_into: u32,

    /// When this segment was archived.
    pub archived_at: DateTime<Utc>,
}

impl Manifest {
    /// Create a new empty manifest.
    ///
    /// Used when initializing a new multi-file repository or migrating
    /// from single-file format.
    pub fn new() -> Self {
        Self {
            version: 1,
            segments: Vec::new(),
            archived_segments: Vec::new(),
            compaction_generation: 0,
            total_frames: 0,
            total_size_bytes: 0,
        }
    }

    /// Get the active (writable) segment, if any.
    ///
    /// The active segment is the last segment without a `closed_at` timestamp.
    pub fn active_segment(&self) -> Option<&SegmentInfo> {
        self.segments.last().filter(|s| s.closed_at.is_none())
    }

    /// Get the active segment mutably.
    pub fn active_segment_mut(&mut self) -> Option<&mut SegmentInfo> {
        self.segments.last_mut().filter(|s| s.closed_at.is_none())
    }

    /// Get the next segment index for a new segment.
    pub fn next_segment_index(&self) -> u32 {
        self.segments
            .iter()
            .map(|s| s.index)
            .max()
            .map(|max| max + 1)
            .unwrap_or(0)
    }

    /// Find segments containing frames in the given range [start, end] (inclusive).
    ///
    /// Returns segments in order. Used for range queries (e.g., commits 9000-9500).
    pub fn find_segments_for_range(&self, start: u64, end: u64) -> Vec<&SegmentInfo> {
        self.segments
            .iter()
            .filter(|s| {
                // Segment overlaps range if:
                // segment.first <= end AND segment.last >= start
                s.first_frame_id <= end && s.last_frame_id >= start
            })
            .collect()
    }

    /// Load manifest from JSON file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read_to_string(path.as_ref())?;
        let manifest: Manifest = serde_json::from_str(&data)?;
        Ok(manifest)
    }

    /// Save manifest to JSON file atomically.
    ///
    /// Writes to a temporary file first, then renames to ensure atomic update.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let tmp_path = path.with_extension("manifest.tmp");

        // Write to temp file
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp_path, json)?;

        // Atomic rename
        std::fs::rename(&tmp_path, path)?;

        Ok(())
    }

    /// Check if segment count exceeds warning threshold.
    pub fn needs_compaction_warning(&self) -> bool {
        self.segments.len() > SEGMENT_COUNT_WARNING
    }

    /// Check if segment count exceeds critical threshold.
    pub fn needs_compaction_critical(&self) -> bool {
        self.segments.len() > SEGMENT_COUNT_CRITICAL
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentInfo {
    /// Create a new segment info for a fresh segment.
    ///
    /// The segment starts with `first_frame_id` and has no frames yet.
    pub fn new(index: u32, first_frame_id: u64) -> Self {
        Self {
            index,
            path: format!(".rfsource.{}", index),
            size_bytes: 0,
            frame_count: 0,
            first_frame_id,
            last_frame_id: first_frame_id, // Will be updated as frames are added
            is_compacted: false,
            compacted_from: Vec::new(),
            checksum_blake3: None,
            created_at: Utc::now(),
            closed_at: None,
        }
    }

    /// Create a compacted segment info.
    pub fn new_compacted(index: u32, first_frame_id: u64, compacted_from: Vec<u32>) -> Self {
        Self {
            index,
            path: format!(".rfsource.compact.{}", index),
            size_bytes: 0,
            frame_count: 0,
            first_frame_id,
            last_frame_id: first_frame_id,
            is_compacted: true,
            compacted_from,
            checksum_blake3: None,
            created_at: Utc::now(),
            closed_at: None,
        }
    }

    /// Check if this segment should be rotated (exceeds size threshold).
    pub fn should_rotate(&self) -> bool {
        self.size_bytes >= SEGMENT_SIZE_THRESHOLD
    }

    /// Get the full path to this segment file relative to repo root.
    pub fn full_path(&self, repo_root: impl AsRef<Path>) -> PathBuf {
        repo_root.as_ref().join(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_new_is_empty() {
        let manifest = Manifest::new();
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.segments.len(), 0);
        assert_eq!(manifest.total_frames, 0);
        assert_eq!(manifest.total_size_bytes, 0);
    }

    #[test]
    fn manifest_next_segment_index_starts_at_zero() {
        let manifest = Manifest::new();
        assert_eq!(manifest.next_segment_index(), 0);
    }

    #[test]
    fn manifest_next_segment_index_increments() {
        let mut manifest = Manifest::new();
        manifest.segments.push(SegmentInfo::new(0, 0));
        assert_eq!(manifest.next_segment_index(), 1);

        manifest.segments.push(SegmentInfo::new(1, 1000));
        assert_eq!(manifest.next_segment_index(), 2);
    }

    #[test]
    fn segment_info_new_creates_valid_segment() {
        let seg = SegmentInfo::new(0, 0);
        assert_eq!(seg.index, 0);
        assert_eq!(seg.path, ".rfsource.0");
        assert_eq!(seg.first_frame_id, 0);
        assert_eq!(seg.last_frame_id, 0);
        assert!(!seg.is_compacted);
        assert!(seg.closed_at.is_none());
    }

    #[test]
    fn segment_info_compacted_has_correct_path() {
        let seg = SegmentInfo::new_compacted(0, 0, vec![0, 1, 2]);
        assert_eq!(seg.path, ".rfsource.compact.0");
        assert!(seg.is_compacted);
        assert_eq!(seg.compacted_from, vec![0, 1, 2]);
    }

    #[test]
    fn segment_should_rotate_at_threshold() {
        let mut seg = SegmentInfo::new(0, 0);
        assert!(!seg.should_rotate());

        seg.size_bytes = SEGMENT_SIZE_THRESHOLD - 1;
        assert!(!seg.should_rotate());

        seg.size_bytes = SEGMENT_SIZE_THRESHOLD;
        assert!(seg.should_rotate());

        seg.size_bytes = SEGMENT_SIZE_THRESHOLD + 1;
        assert!(seg.should_rotate());
    }

    #[test]
    fn find_segments_for_range_finds_overlapping() {
        let mut manifest = Manifest::new();

        // Segment 0: frames 0-999
        let mut seg0 = SegmentInfo::new(0, 0);
        seg0.last_frame_id = 999;
        manifest.segments.push(seg0);

        // Segment 1: frames 1000-1999
        let mut seg1 = SegmentInfo::new(1, 1000);
        seg1.last_frame_id = 1999;
        manifest.segments.push(seg1);

        // Segment 2: frames 2000-2999
        let mut seg2 = SegmentInfo::new(2, 2000);
        seg2.last_frame_id = 2999;
        manifest.segments.push(seg2);

        // Query range 500-1500 should return segments 0 and 1
        let result = manifest.find_segments_for_range(500, 1500);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].index, 0);
        assert_eq!(result[1].index, 1);

        // Query range 0-500 should return only segment 0
        let result = manifest.find_segments_for_range(0, 500);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 0);

        // Query range 2500-2600 should return only segment 2
        let result = manifest.find_segments_for_range(2500, 2600);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 2);
    }

    #[test]
    fn manifest_serialization_roundtrip() {
        let mut manifest = Manifest::new();
        manifest.segments.push(SegmentInfo::new(0, 0));
        manifest.total_frames = 100;
        manifest.total_size_bytes = 1_000_000;

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, manifest.version);
        assert_eq!(deserialized.segments.len(), 1);
        assert_eq!(deserialized.total_frames, 100);
        assert_eq!(deserialized.total_size_bytes, 1_000_000);
    }

    #[test]
    fn active_segment_returns_last_open_segment() {
        let mut manifest = Manifest::new();

        // No segments yet
        assert!(manifest.active_segment().is_none());

        // Add open segment
        manifest.segments.push(SegmentInfo::new(0, 0));
        assert!(manifest.active_segment().is_some());
        assert_eq!(manifest.active_segment().unwrap().index, 0);

        // Close segment 0, add segment 1
        manifest.segments[0].closed_at = Some(Utc::now());
        manifest.segments.push(SegmentInfo::new(1, 1000));

        assert!(manifest.active_segment().is_some());
        assert_eq!(manifest.active_segment().unwrap().index, 1);
    }

    #[test]
    fn needs_compaction_thresholds() {
        let mut manifest = Manifest::new();

        // Add segments up to warning threshold
        for i in 0..=SEGMENT_COUNT_WARNING {
            manifest.segments.push(SegmentInfo::new(i as u32, i as u64 * 1000));
        }

        assert!(manifest.needs_compaction_warning());
        assert!(!manifest.needs_compaction_critical());

        // Add more to hit critical
        for i in (SEGMENT_COUNT_WARNING + 1)..=SEGMENT_COUNT_CRITICAL {
            manifest.segments.push(SegmentInfo::new(i as u32, i as u64 * 1000));
        }

        assert!(manifest.needs_compaction_warning());
        assert!(manifest.needs_compaction_critical());
    }
}
