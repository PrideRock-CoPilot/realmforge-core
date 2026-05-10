# DDR-003 Day 4 Implementation Guide

**Phase 2:** Footers + Checksums  
**Day 4:** Footer Format + Writing  
**Status:** Ready for local implementation

---

## Implementation Checklist

### Step 1: Create `footer.rs` Module

**File:** `crates/rfsource-format/src/footer.rs`

**Content:** See `/tmp/footer_structs.rs` (already generated)

**Key Components:**
* `FOOTER_SIGNATURE` constant: `b"RFSFOOT\x00"`
* `MAX_FOOTER_SIZE` constant: 1024 bytes
* `SegmentFooter` struct with:
  - `segment_index`, `frame_count`, `first_frame_id`, `last_frame_id`
  - `size_bytes`, `checksum_blake3`
  - `is_compacted`, `compacted_from`
  - `created_at`, `closed_at`
* Methods:
  - `new()` - Create empty footer
  - `write()` - Write footer to end of segment
  - `read()` - Read footer from segment end
  - `read_from_file()` - Convenience wrapper
* Unit tests (5 tests):
  - `test_footer_new`
  - `test_footer_write_read_roundtrip`
  - `test_footer_signature_verification`
  - `test_footer_size_limit`

### Step 2: Export Footer Module

**File:** `crates/rfsource-format/src/lib.rs`

**Add:**
```rust
pub mod footer;
pub use footer::{SegmentFooter, FOOTER_SIGNATURE, MAX_FOOTER_SIZE};
```

### Step 3: Update `multi_file.rs` with Footer Writing

**File:** `crates/rfsource-store/src/multi_file.rs`

**Changes to `rotate_segment()` method:**

```rust
fn rotate_segment(&mut self) -> Result<()> {
    // Close current segment
    let next_frame_id = if let Some(active) = self.manifest.active_segment_mut() {
        active.closed_at = Some(Utc::now());
        
        // NEW: Write footer before rotating
        let footer = SegmentFooter {
            segment_index: active.index,
            frame_count: active.frame_count,
            first_frame_id: active.first_frame_id,
            last_frame_id: active.last_frame_id,
            size_bytes: active.size_bytes,
            checksum_blake3: String::new(), // Day 5: BLAKE3 integration
            is_compacted: active.is_compacted,
            compacted_from: active.compacted_from.clone(),
            created_at: active.created_at,
            closed_at: active.closed_at,
        };
        
        // Write footer to segment file
        let seg_path = active.full_path(&self.repo_dir);
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&seg_path)?;
        footer.write(&mut file)?;
        file.sync_all()?; // fsync
        
        active.last_frame_id + 1
    } else {
        return Err(StoreError::Internal("No active segment to close".to_string()));
    };

    // Create new segment
    let next_index = self.manifest.next_segment_index();
    let new_seg = SegmentInfo::new(next_index, next_frame_id);
    let new_seg_path = new_seg.full_path(&self.repo_dir);

    // Initialize segment file
    initialize(&new_seg_path)?;

    // Add to manifest
    self.manifest.segments.push(new_seg);

    // Save manifest
    self.save_manifest()?;

    Ok(())
}
```

### Step 4: Add Footer Verification Test

**File:** `crates/rfsource-store/src/multi_file.rs`

**Add to tests module:**

```rust
#[test]
fn test_segment_footer_written_on_rotation() {
    let dir = tempdir().unwrap();
    let mut repo = MultiFileRepo::open_or_init(dir.path()).unwrap();

    // Write enough frames to trigger rotation (simulate large frames)
    // In real test, write >1GB of data
    // For unit test, manually trigger rotation
    
    // Close segment 0
    repo.rotate_segment().unwrap();

    // Verify footer exists on segment 0
    let seg0_path = dir.path().join(".rfsource.0");
    let footer = SegmentFooter::read_from_file(&seg0_path).unwrap();

    assert_eq!(footer.segment_index, 0);
    assert!(footer.closed_at.is_some());
    assert!(footer.frame_count > 0);
}

#[test]
fn test_footer_contains_correct_frame_range() {
    let dir = tempdir().unwrap();
    let mut repo = MultiFileRepo::open_or_init(dir.path()).unwrap();

    // Write exactly 10 frames
    for i in 0..10 {
        let frame = TestFrame {
            id: i,
            data: format!("frame_{}", i),
        };
        repo.append_frame(&frame).unwrap();
    }

    // Rotate
    repo.rotate_segment().unwrap();

    // Read footer
    let seg0_path = dir.path().join(".rfsource.0");
    let footer = SegmentFooter::read_from_file(&seg0_path).unwrap();

    assert_eq!(footer.frame_count, 10);
    assert_eq!(footer.first_frame_id, 0);
    assert_eq!(footer.last_frame_id, 9);
}
```

---

## Testing Locally

### Unit Tests
```bash
cd /path/to/realmforge-core-local
cargo test --package rfsource-format footer
cargo test --package rfsource-store multi_file::tests::test_segment_footer
```

### Expected Output
```
running 7 tests
test footer::tests::test_footer_new ... ok
test footer::tests::test_footer_write_read_roundtrip ... ok
test footer::tests::test_footer_signature_verification ... ok
test footer::tests::test_footer_size_limit ... ok
test multi_file::tests::test_segment_footer_written_on_rotation ... ok
test multi_file::tests::test_footer_contains_correct_frame_range ... ok

test result: ok. 7 passed; 0 failed; 0 ignored
```

---

## Day 4 Validation Criteria

- [ ] `footer.rs` module created with complete `SegmentFooter` struct
- [ ] Footer written to segment files on rotation
- [ ] Footer signature verification works
- [ ] Footer size limit enforced (<1KB)
- [ ] Footer read/write roundtrip test passes
- [ ] Footer contains correct frame range metadata
- [ ] All 7 unit tests pass

---

## Next Steps (Day 5)

After Day 4 validation passes:
1. Add BLAKE3 crate dependency to `rfsource-format/Cargo.toml`
2. Implement incremental checksumming during frame writes
3. Finalize checksum in footer on segment close
4. Implement manifest rebuild from segment footers
5. Add checksum verification tests

---

**Implementation Time Estimate:** 2-3 hours (local)

**Files to Create/Modify:**
1. `crates/rfsource-format/src/footer.rs` - NEW (266 lines)
2. `crates/rfsource-format/src/lib.rs` - UPDATE (+2 lines)
3. `crates/rfsource-store/src/multi_file.rs` - UPDATE (~30 lines in rotate_segment)
4. `crates/rfsource-store/src/multi_file.rs` - UPDATE (+40 lines tests)

**Total:** ~338 lines of new/updated code

