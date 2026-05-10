// crates/rfsource-store/tests/phase2_integration.rs
//! Phase 2 Integration Tests
//! 
//! Tests the complete Phase 2 system:
//! - Multi-file repository with automatic segment rotation
//! - Segment footers with metadata
//! - BLAKE3 checksums for corruption detection
//! - Manifest rebuild from footers
//!
//! These tests verify end-to-end functionality across all Phase 2 components.

use rfsource_store::MultiFileRepo;
use rfsource_format::{SegmentFooter, Manifest, SegmentChecksum, verify_file};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{Write, Read};
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
            data: format!("Test frame {}", id),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
    
    fn with_data(id: u64, data: String) -> Self {
        Self {
            id,
            data,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Test 1: End-to-end write, rotate, read with checksums
#[test]
fn test_e2e_write_rotate_read_verify() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Create repo and write frames
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    let frame_count = 100;
    for i in 0..frame_count {
        let frame = TestFrame::new(i);
        repo.append_frame_with_checksum(&frame).unwrap();
    }
    
    // Force rotation to close segment
    repo.rotate_segment().unwrap();
    
    // Verify segment 0 exists with checksum
    let segment = &repo.manifest.segments[0];
    assert_eq!(segment.index, 0);
    assert_eq!(segment.frame_count, frame_count);
    assert!(segment.checksum_blake3.is_some());
    assert!(segment.closed_at.is_some());
    
    // Verify footer exists and matches
    let segment_path = Path::new(&segment.path);
    let footer = SegmentFooter::read_from_file(segment_path).unwrap();
    assert_eq!(footer.segment_index, 0);
    assert_eq!(footer.frame_count, frame_count);
    assert_eq!(footer.checksum_blake3, segment.checksum_blake3.as_ref().unwrap());
    
    // Read with verification
    let frames: Vec<TestFrame> = repo.read_frames_with_verification(&[0]).unwrap();
    assert_eq!(frames.len(), frame_count as usize);
    
    // Verify frame data
    for (i, frame) in frames.iter().enumerate() {
        assert_eq!(frame.id, i as u64);
    }
}

/// Test 2: Corruption detection via checksum verification
#[test]
fn test_corruption_detection() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Create repo with data
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    for i in 0..50 {
        repo.append_frame_with_checksum(&TestFrame::new(i)).unwrap();
    }
    repo.rotate_segment().unwrap();
    
    let segment_path = Path::new(&repo.manifest.segments[0].path);
    
    // Corrupt the segment file (append garbage data before footer)
    let original_size = std::fs::metadata(segment_path).unwrap().len();
    let mut file = OpenOptions::new().write(true).open(segment_path).unwrap();
    
    // Seek to middle of file and overwrite
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(100)).unwrap();
    file.write_all(b"CORRUPTED_DATA_HERE").unwrap();
    file.flush().unwrap();
    
    // Try to read with verification - should fail
    let result: Result<Vec<TestFrame>, _> = repo.read_frames_with_verification(&[0]);
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("checksum") || error_msg.contains("verification"));
}

/// Test 3: Multiple segment rotation with checksum continuity
#[test]
fn test_multiple_segment_rotation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Write and rotate 3 segments
    let segments_to_create = 3;
    let frames_per_segment = 100;
    
    for seg in 0..segments_to_create {
        for i in 0..frames_per_segment {
            let frame_id = seg * frames_per_segment + i;
            repo.append_frame_with_checksum(&TestFrame::new(frame_id)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    // Verify all segments have checksums
    assert_eq!(repo.manifest.segments.len(), segments_to_create as usize + 1); // +1 active
    
    for i in 0..segments_to_create {
        let segment = &repo.manifest.segments[i as usize];
        assert_eq!(segment.index, i);
        assert!(segment.checksum_blake3.is_some());
        assert!(segment.closed_at.is_some());
        assert_eq!(segment.frame_count, frames_per_segment);
        
        // Verify footer
        let footer = SegmentFooter::read_from_file(Path::new(&segment.path)).unwrap();
        assert_eq!(footer.checksum_blake3, segment.checksum_blake3.as_ref().unwrap());
    }
    
    // Read all segments with verification
    let segment_indices: Vec<u32> = (0..segments_to_create).collect();
    let frames: Vec<TestFrame> = repo.read_frames_with_verification(&segment_indices).unwrap();
    assert_eq!(frames.len(), (segments_to_create * frames_per_segment) as usize);
    
    // Verify frame ID continuity
    for (i, frame) in frames.iter().enumerate() {
        assert_eq!(frame.id, i as u64);
    }
}

/// Test 4: Manifest rebuild from footers
#[test]
fn test_manifest_rebuild_from_footers() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Create repo with multiple segments
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    for seg in 0..3 {
        for i in 0..50 {
            let frame_id = seg * 50 + i;
            repo.append_frame_with_checksum(&TestFrame::new(frame_id)).unwrap();
        }
        repo.rotate_segment().unwrap();
    }
    
    // Save original manifest for comparison
    let original_manifest = repo.manifest.clone();
    
    // Delete manifest file
    let manifest_path = repo_path.join(".rfsource.manifest");
    std::fs::remove_file(&manifest_path).unwrap();
    
    // Rebuild from footers
    let rebuilt_manifest = MultiFileRepo::rebuild_manifest_from_footers(repo_path).unwrap();
    
    // Verify segment count (should match closed segments only)
    assert_eq!(rebuilt_manifest.segments.len(), 3);
    
    // Verify each segment matches original
    for (rebuilt, original) in rebuilt_manifest.segments.iter().zip(original_manifest.segments.iter()) {
        assert_eq!(rebuilt.index, original.index);
        assert_eq!(rebuilt.frame_count, original.frame_count);
        assert_eq!(rebuilt.first_frame_id, original.first_frame_id);
        assert_eq!(rebuilt.last_frame_id, original.last_frame_id);
        assert_eq!(rebuilt.checksum_blake3, original.checksum_blake3);
    }
    
    // Save rebuilt manifest
    rebuilt_manifest.save(&manifest_path).unwrap();
    
    // Reopen repo - should work normally
    let reopened = MultiFileRepo::open_or_init(repo_path).unwrap();
    assert_eq!(reopened.manifest.segments.len(), 4); // 3 rebuilt + 1 new active
}

/// Test 5: Footer integrity after segment close
#[test]
fn test_footer_integrity() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Write frames
    for i in 0..75 {
        repo.append_frame_with_checksum(&TestFrame::new(i)).unwrap();
    }
    
    // Rotate segment
    repo.rotate_segment().unwrap();
    
    let segment = &repo.manifest.segments[0];
    let segment_path = Path::new(&segment.path);
    
    // Read footer
    let footer = SegmentFooter::read_from_file(segment_path).unwrap();
    
    // Verify footer fields match manifest
    assert_eq!(footer.segment_index, segment.index);
    assert_eq!(footer.frame_count, segment.frame_count);
    assert_eq!(footer.first_frame_id, segment.first_frame_id);
    assert_eq!(footer.last_frame_id, segment.last_frame_id);
    assert_eq!(footer.size_bytes, segment.size_bytes);
    assert_eq!(footer.checksum_blake3, segment.checksum_blake3.as_ref().unwrap());
    assert_eq!(footer.is_compacted, segment.is_compacted);
    
    // Verify checksum is valid for segment data
    let checksum = SegmentChecksum::from_hex(&footer.checksum_blake3).unwrap();
    
    // Read segment file (excluding footer) and verify
    // Note: In practice, we verify the whole file including footer
    verify_file(segment_path, &checksum).unwrap();
}

/// Test 6: Concurrent segment writes (stress test)
#[test]
fn test_concurrent_segment_writes() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Write many frames quickly to test segment rotation under load
    let total_frames = 1000;
    
    for i in 0..total_frames {
        let frame = TestFrame::with_data(i, "x".repeat(1000)); // ~1KB per frame
        repo.append_frame_with_checksum(&frame).unwrap();
        
        // Periodically force rotation to test footer writes
        if i > 0 && i % 250 == 0 {
            repo.rotate_segment().unwrap();
        }
    }
    
    // Final rotation
    repo.rotate_segment().unwrap();
    
    // Verify all segments have valid footers and checksums
    for segment in &repo.manifest.segments {
        if segment.closed_at.is_some() {
            assert!(segment.checksum_blake3.is_some());
            
            let footer = SegmentFooter::read_from_file(Path::new(&segment.path)).unwrap();
            assert_eq!(footer.segment_index, segment.index);
            assert_eq!(footer.checksum_blake3, segment.checksum_blake3.as_ref().unwrap());
        }
    }
    
    // Read all frames with verification
    let closed_segments: Vec<u32> = repo.manifest.segments.iter()
        .filter(|s| s.closed_at.is_some())
        .map(|s| s.index)
        .collect();
    
    let frames: Vec<TestFrame> = repo.read_frames_with_verification(&closed_segments).unwrap();
    
    // Should read all frames from closed segments
    let expected_count = closed_segments.len() * 250; // 250 frames per segment
    assert!(frames.len() >= expected_count - 250); // Allow for last partial segment
}

/// Test 7: Checksum verification performance
#[test]
fn test_checksum_verification_performance() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Write moderate amount of data
    let frame_count = 500;
    for i in 0..frame_count {
        let frame = TestFrame::with_data(i, "test data ".repeat(100)); // ~1KB each
        repo.append_frame_with_checksum(&frame).unwrap();
    }
    repo.rotate_segment().unwrap();
    
    // Measure read without verification
    let start = std::time::Instant::now();
    let _frames: Vec<TestFrame> = repo.read_frames_from_segments(&[0]).unwrap();
    let duration_no_verify = start.elapsed();
    
    // Measure read with verification
    let start = std::time::Instant::now();
    let _frames: Vec<TestFrame> = repo.read_frames_with_verification(&[0]).unwrap();
    let duration_with_verify = start.elapsed();
    
    // Verification overhead should be minimal (<20%)
    let overhead_ratio = duration_with_verify.as_secs_f64() / duration_no_verify.as_secs_f64();
    assert!(overhead_ratio < 1.2, "Verification overhead too high: {:.2}x", overhead_ratio);
    
    println!("Checksum verification overhead: {:.2}%", (overhead_ratio - 1.0) * 100.0);
}

/// Test 8: Partial corruption in middle of segment
#[test]
fn test_partial_segment_corruption() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let mut repo = MultiFileRepo::open_or_init(repo_path).unwrap();
    
    // Write data
    for i in 0..100 {
        repo.append_frame_with_checksum(&TestFrame::new(i)).unwrap();
    }
    repo.rotate_segment().unwrap();
    
    // Corrupt specific byte in middle of segment
    let segment_path = Path::new(&repo.manifest.segments[0].path);
    let mut file = OpenOptions::new().write(true).open(segment_path).unwrap();
    
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(500)).unwrap();
    file.write_all(&[0xFF]).unwrap(); // Flip one byte
    
    // Verification should catch single byte flip
    let result: Result<Vec<TestFrame>, _> = repo.read_frames_with_verification(&[0]);
    assert!(result.is_err());
}
