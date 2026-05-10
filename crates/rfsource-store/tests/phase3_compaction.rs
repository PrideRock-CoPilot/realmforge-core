// crates/rfsource-store/tests/phase3_compaction.rs
//! Phase 3 Integration Tests - Compaction System
//!
//! Tests the complete compaction system:
//! - Compaction planning with different strategies
//! - Compaction execution with checksum verification
//! - Segment archival and restoration
//! - Rollback functionality
//! - Multi-generation compaction
//! - Query correctness after compaction
//!
//! These tests verify end-to-end functionality across all Phase 3 components.

use rfsource_store::{MultiFileRepo, compaction::*};
use rfsource_format::{SegmentFooter, SegmentChecksum, verify_file};
use std::path::Path;
use std::fs;
use tempfile::TempDir;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestFrame {
    id: u64,
    data: String,
    timestamp: i64,
}

impl TestFrame {
    fn new(id: u64) -> Self {
        Self {
            id,
            data: format!("Frame {}", id),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
    
    fn with_size(id: u64, size: usize) -> Self {
        Self {
            id,
            data: "x".repeat(size),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Test 1: Small segments compaction strategy
#[test]
fn test_compact_small_segments() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Create 5 small segments (50 frames each)
    for seg in 0..5 {
        for i in 0..50 {
            let frame_id = seg * 50 + i;
            repo.append_frame_with_checksum(&TestFrame::new(frame_id)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    // Create plan for small segments
    let strategy = CompactionStrategy::SmallSegments {
        threshold_bytes: 1024 * 1024 * 1024, // 1GB - all test segments are small
    };
    
    let plan = CompactionPlan::create(&repo.manifest, strategy).unwrap();
    
    assert_eq!(plan.segments.len(), 5);
    assert_eq!(plan.total_frames, 250);
    
    // Execute compaction
    let mut compactor = Compactor::new(repo_path, &mut repo.manifest);
    let result = compactor.compact::<TestFrame>(&plan).unwrap();
    
    assert_eq!(result.segments_compacted, 5);
    assert_eq!(result.frames_compacted, 250);
    
    // Verify compacted segment exists
    let compacted_path = repo_path.join("segment_00000.compacted.rfsource");
    assert!(compacted_path.exists());
    
    // Verify footer
    let footer = SegmentFooter::read_from_file(&compacted_path).unwrap();
    assert_eq!(footer.frame_count, 250);
    assert!(footer.is_compacted);
    assert_eq!(footer.compacted_from.as_ref().unwrap().len(), 5);
    
    // Verify original segments archived
    let archive_dir = repo_path.join(".archived");
    assert!(archive_dir.exists());
    assert_eq!(fs::read_dir(archive_dir).unwrap().count(), 5);
}

/// Test 2: Query correctness after compaction
#[test]
fn test_query_after_compaction() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Write frames across multiple segments
    let total_frames = 300;
    for i in 0..total_frames {
        repo.append_frame_with_checksum(&TestFrame::new(i)).unwrap();
        
        // Rotate every 50 frames
        if (i + 1) % 50 == 0 {
            repo.rotate_segment().unwrap();
        }
    }
    
    // Read all frames before compaction
    let segments_before: Vec<u32> = repo.manifest.segments.iter()
        .filter(|s| s.closed_at.is_some())
        .map(|s| s.index)
        .collect();
    
    let frames_before: Vec<TestFrame> = repo.read_frames_with_verification(&segments_before).unwrap();
    assert_eq!(frames_before.len(), total_frames as usize);
    
    // Compact all segments
    let plan = CompactionPlan::create(&repo.manifest, CompactionStrategy::All).unwrap();
    let mut compactor = Compactor::new(repo_path, &mut repo.manifest);
    compactor.compact::<TestFrame>(&plan).unwrap();
    
    // Read all frames after compaction
    let segments_after: Vec<u32> = repo.manifest.segments.iter()
        .filter(|s| s.closed_at.is_some())
        .map(|s| s.index)
        .collect();
    
    let frames_after: Vec<TestFrame> = repo.read_frames_with_verification(&segments_after).unwrap();
    
    // Verify same frames returned
    assert_eq!(frames_after.len(), frames_before.len());
    
    for (before, after) in frames_before.iter().zip(frames_after.iter()) {
        assert_eq!(before.id, after.id);
        assert_eq!(before.data, after.data);
    }
}

/// Test 3: Rollback restores original segments
#[test]
fn test_rollback_compaction() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Create segments
    for seg in 0..3 {
        for i in 0..50 {
            repo.append_frame_with_checksum(&TestFrame::new(seg * 50 + i)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    // Save original manifest state
    let original_segments = repo.manifest.segments.clone();
    let original_gen = repo.manifest.compaction_generation;
    
    // Compact
    let plan = CompactionPlan::create(&repo.manifest, CompactionStrategy::All).unwrap();
    let mut compactor = Compactor::new(repo_path, &mut repo.manifest);
    compactor.compact::<TestFrame>(&plan).unwrap();
    
    // Verify compaction happened
    assert_eq!(repo.manifest.compaction_generation, original_gen + 1);
    assert!(repo.manifest.archived_segments.len() == 3);
    
    // Rollback
    compactor.rollback_last_compaction().unwrap();
    
    // Verify restoration
    assert_eq!(repo.manifest.compaction_generation, original_gen + 1); // Generation doesn't decrease
    assert_eq!(repo.manifest.archived_segments.len(), 0);
    
    // Verify segments restored (count should match)
    let restored_non_active = repo.manifest.segments.iter()
        .filter(|s| s.closed_at.is_some())
        .count();
    
    let original_non_active = original_segments.iter()
        .filter(|s| s.closed_at.is_some())
        .count();
    
    assert_eq!(restored_non_active, original_non_active);
    
    // Verify segment files exist
    for segment in &repo.manifest.segments {
        if segment.closed_at.is_some() && !segment.is_compacted {
            assert!(Path::new(&segment.path).exists(),
                "Segment file {} should exist after rollback", segment.path);
        }
    }
}

/// Test 4: Multi-generation compaction
#[test]
fn test_multi_generation_compaction() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Generation 0: Create 6 small segments
    for seg in 0..6 {
        for i in 0..20 {
            repo.append_frame_with_checksum(&TestFrame::new(seg * 20 + i)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    let initial_gen = repo.manifest.compaction_generation;
    
    // Generation 0→1: Compact first 3 segments
    let plan = CompactionPlan::create(&repo.manifest, CompactionStrategy::Range {
        start_index: 0,
        end_index: 2,
    }).unwrap();
    
    let mut compactor = Compactor::new(repo_path, &mut repo.manifest);
    compactor.compact::<TestFrame>(&plan).unwrap();
    
    assert_eq!(repo.manifest.compaction_generation, initial_gen + 1);
    
    // Generation 1→2: Compact remaining segments
    let plan2 = CompactionPlan::create(&repo.manifest, CompactionStrategy::Range {
        start_index: 3,
        end_index: 5,
    }).unwrap();
    
    compactor.compact::<TestFrame>(&plan2).unwrap();
    
    assert_eq!(repo.manifest.compaction_generation, initial_gen + 2);
    
    // Verify two compacted segments exist
    let compacted_count = repo.manifest.segments.iter()
        .filter(|s| s.is_compacted)
        .count();
    
    assert_eq!(compacted_count, 2);
    
    // Verify archived segments
    assert_eq!(repo.manifest.archived_segments.len(), 6);
}

/// Test 5: Compaction preserves checksums
#[test]
fn test_compaction_checksum_integrity() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Write segments
    for seg in 0..3 {
        for i in 0..30 {
            repo.append_frame_with_checksum(&TestFrame::new(seg * 30 + i)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    // Compact
    let plan = CompactionPlan::create(&repo.manifest, CompactionStrategy::All).unwrap();
    let mut compactor = Compactor::new(repo_path, &mut repo.manifest);
    let result = compactor.compact::<TestFrame>(&plan).unwrap();
    
    // Verify compacted segment checksum
    let compacted_checksum = SegmentChecksum::from_hex(&result.compacted_checksum).unwrap();
    verify_file(&result.compacted_path, &compacted_checksum).unwrap();
    
    // Verify footer checksum matches
    let footer = SegmentFooter::read_from_file(&result.compacted_path).unwrap();
    assert_eq!(footer.checksum_blake3, result.compacted_checksum);
    
    // Read with verification should succeed
    let compacted_segment = repo.manifest.segments.iter()
        .find(|s| s.is_compacted)
        .unwrap();
    
    let frames: Vec<TestFrame> = repo.read_frames_with_verification(&[compacted_segment.index]).unwrap();
    assert_eq!(frames.len(), 90);
}

/// Test 6: Compaction with large segments (>1GB equivalent in frames)
#[test]
fn test_compact_large_dataset() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Create segments with larger frames
    let frames_per_segment = 500;
    let segment_count = 5;
    
    for seg in 0..segment_count {
        for i in 0..frames_per_segment {
            let frame_id = seg * frames_per_segment + i;
            // Create ~10KB frames
            repo.append_frame_with_checksum(&TestFrame::with_size(frame_id, 10_000)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    let total_frames = segment_count * frames_per_segment;
    
    // Compact all
    let plan = CompactionPlan::create(&repo.manifest, CompactionStrategy::All).unwrap();
    let mut compactor = Compactor::new(repo_path, &mut repo.manifest);
    
    let start = std::time::Instant::now();
    let result = compactor.compact::<TestFrame>(&plan).unwrap();
    let duration = start.elapsed();
    
    assert_eq!(result.frames_compacted, total_frames);
    
    // Calculate throughput (should be >50 MB/s on modern hardware)
    let throughput_mbps = (result.bytes_written / 1024 / 1024) as f64 / duration.as_secs_f64();
    println!("Compaction throughput: {:.2} MB/s", throughput_mbps);
    
    assert!(throughput_mbps > 10.0, "Compaction too slow: {:.2} MB/s", throughput_mbps);
}

/// Test 7: Compaction plan creation with no eligible segments
#[test]
fn test_no_segments_to_compact() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Empty repo - no segments to compact
    let result = CompactionPlan::create(&repo.manifest, CompactionStrategy::All);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No segments eligible"));
}

/// Test 8: Cannot compact active segment
#[test]
fn test_cannot_compact_active_segment() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Write some frames but don't rotate
    for i in 0..10 {
        repo.append_frame_with_checksum(&TestFrame::new(i)).unwrap();
    }
    
    // Get active segment index
    let active_index = repo.manifest.active_segment().unwrap().index;
    
    // Try to compact active segment
    let result = CompactionPlan::create(&repo.manifest, CompactionStrategy::Range {
        start_index: active_index,
        end_index: active_index,
    });
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("active segment"));
}

/// Test 9: Compaction updates manifest correctly
#[test]
fn test_compaction_manifest_update() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Create segments
    for seg in 0..4 {
        for i in 0..25 {
            repo.append_frame_with_checksum(&TestFrame::new(seg * 25 + i)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    let segments_before = repo.manifest.segments.len();
    let gen_before = repo.manifest.compaction_generation;
    
    // Compact
    let plan = CompactionPlan::create(&repo.manifest, CompactionStrategy::All).unwrap();
    let mut compactor = Compactor::new(repo_path, &mut repo.manifest);
    compactor.compact::<TestFrame>(&plan).unwrap();
    
    // Verify manifest updates
    assert_eq!(repo.manifest.compaction_generation, gen_before + 1);
    
    // Should have: 1 compacted + 1 active (new) = 2 segments
    assert_eq!(repo.manifest.segments.len(), 2);
    
    // Verify archived count
    assert_eq!(repo.manifest.archived_segments.len(), 4);
    
    // Reload manifest from disk and verify persistence
    let manifest_path = repo_path.join(".rfsource.manifest");
    let loaded = rfsource_format::Manifest::load(&manifest_path).unwrap();
    
    assert_eq!(loaded.compaction_generation, repo.manifest.compaction_generation);
    assert_eq!(loaded.segments.len(), repo.manifest.segments.len());
    assert_eq!(loaded.archived_segments.len(), repo.manifest.archived_segments.len());
}

/// Test 10: Frame ID continuity after compaction
#[test]
fn test_frame_id_continuity() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Create segments with specific frame ID ranges
    for seg in 0..3 {
        for i in 0..50 {
            let frame_id = seg * 50 + i;
            repo.append_frame_with_checksum(&TestFrame::new(frame_id)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    // Verify frame ID ranges before compaction
    for segment in &repo.manifest.segments {
        if segment.closed_at.is_some() {
            let expected_first = segment.index * 50;
            let expected_last = expected_first + 49;
            assert_eq!(segment.first_frame_id, expected_first);
            assert_eq!(segment.last_frame_id, expected_last);
        }
    }
    
    // Compact
    let plan = CompactionPlan::create(&repo.manifest, CompactionStrategy::All).unwrap();
    let mut compactor = Compactor::new(repo_path, &mut repo.manifest);
    compactor.compact::<TestFrame>(&plan).unwrap();
    
    // Verify compacted segment has correct frame ID range
    let compacted = repo.manifest.segments.iter()
        .find(|s| s.is_compacted)
        .unwrap();
    
    assert_eq!(compacted.first_frame_id, 0);
    assert_eq!(compacted.last_frame_id, 149);
    assert_eq!(compacted.frame_count, 150);
}
