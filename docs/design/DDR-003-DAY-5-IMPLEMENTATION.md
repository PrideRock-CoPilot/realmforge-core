# DDR-003 Day 5 Implementation Guide
## BLAKE3 Checksumming + Manifest Rebuild

**Status:** Design Complete, Ready for Local Implementation
**Dependencies:** Day 4 (Footer Format)
**Time Estimate:** 2-3 hours local coding + testing

---

## Phase 2 Day 5 Overview

Add BLAKE3 checksumming to detect data corruption:
- Streaming checksum calculation during writes
- Automatic verification during reads
- Manifest rebuild from segment footers (disaster recovery)
- No performance impact (BLAKE3 >1 GB/s single-threaded)

---

## Step 1: Add BLAKE3 Dependency

**File:** `Cargo.toml` (workspace root)

```toml
[dependencies]
blake3 = "1.5"
hex = "0.4"
```

---

## Step 2: Create Checksum Module

**File:** `crates/rfsource-format/src/checksum.rs`

**Source:** `/tmp/checksum_module.rs` (400 lines)

**What it provides:**
- `SegmentChecksum` - BLAKE3 hash wrapper (32 bytes)
- `ChecksumWriter` - Streaming Write wrapper that calculates hash
- `ChecksumReader` - Streaming Read wrapper that verifies hash
- `checksum_file()` - Calculate checksum of entire file
- `verify_file()` - Verify file against expected checksum
- `ChecksumError` - Mismatch and I/O errors

**Key APIs:**
```rust
// Create from hex string
let checksum = SegmentChecksum::from_hex("0123...cdef")?;

// Write with checksum
let mut writer = ChecksumWriter::new(file);
writer.write_all(data)?;
let (file, checksum, bytes_written) = writer.finalize();

// Read with verification
let expected = SegmentChecksum::from_hex("...")?;
let mut reader = ChecksumReader::new_with_expected(file, expected);
reader.read_to_end(&mut buf)?;
reader.verify()?; // Returns error if mismatch

// File-level operations
let checksum = checksum_file("segment_00000.rfsource")?;
verify_file("segment_00000.rfsource", &checksum)?;
```

---

## Step 3: Export Checksum Module

**File:** `crates/rfsource-format/src/lib.rs`

**Add:**
```rust
pub mod checksum;
pub use checksum::{SegmentChecksum, ChecksumWriter, ChecksumReader, ChecksumError, checksum_file, verify_file};
```

---

## Step 4: Integrate into Multi-File Store

**File:** `crates/rfsource-store/src/multi_file.rs`

**Source:** `/tmp/multi_file_checksum_integration.rs`

**Changes:**
1. Import checksum modules at top
2. Update `rotate_segment()` to calculate and store checksum
3. Add `append_frame_with_checksum()` method (optional - can be done later)
4. Add `read_frames_with_verification()` method
5. Add `rebuild_manifest_from_footers()` function

**Key Integration Points:**

### A. Update `rotate_segment()` (Critical)
```rust
pub fn rotate_segment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let current_segment = self.manifest.active_segment()?;
    let segment_path = segment_path(&self.repo_dir, current_segment.index);
    
    // Calculate final checksum for the complete segment
    let checksum = checksum_file(&segment_path)?;
    
    // Update manifest with checksum
    let segment_mut = self.manifest.active_segment_mut()?;
    segment_mut.checksum_blake3 = Some(checksum.to_hex());
    segment_mut.closed_at = Some(chrono::Utc::now());
    
    // Write footer with checksum
    let footer = SegmentFooter {
        // ... existing fields ...
        checksum_blake3: checksum.to_hex(),
    };
    
    footer.write(&segment_path)?;
    
    // ... rest of rotation logic ...
}
```

### B. Add Verification on Read
```rust
pub fn read_frames_with_verification<T: serde::de::DeserializeOwned>(
    &self,
    segment_indices: &[u32],
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    for &seg_idx in segment_indices {
        let segment = /* find segment */;
        
        // Verify checksum if present
        if let Some(checksum_hex) = &segment.checksum_blake3 {
            let expected = SegmentChecksum::from_hex(checksum_hex)?;
            verify_file(&segment.path, &expected)?;
        }
        
        // Read frames normally...
    }
}
```

### C. Add Manifest Rebuild
```rust
pub fn rebuild_manifest_from_footers(
    repo_dir: &Path,
) -> Result<Manifest, Box<dyn std::error::Error>> {
    let mut segments = Vec::new();
    
    // Find all segment files
    for entry in std::fs::read_dir(repo_dir)? {
        let path = entry.path();
        if !is_segment_file(&path) { continue; }
        
        // Read footer
        if let Ok(footer) = SegmentFooter::read_from_file(&path) {
            segments.push(SegmentInfo {
                index: footer.segment_index,
                checksum_blake3: Some(footer.checksum_blake3),
                // ... other fields from footer ...
            });
        }
    }
    
    // Sort and build manifest
    segments.sort_by_key(|s| s.index);
    Ok(Manifest { segments, /* ... */ })
}
```

---

## Step 5: Run Tests

### Unit Tests (Checksum Module)
```bash
cd crates/rfsource-format
cargo test checksum::tests::
```

**Expected:** 9/9 tests pass
- test_checksum_from_hex
- test_checksum_from_hex_invalid
- test_checksum_writer
- test_checksum_reader_no_verification
- test_checksum_reader_with_verification_success
- test_checksum_reader_with_verification_failure
- test_checksum_verify_bytes
- test_streaming_matches_bulk
- test_checksum_display

### Integration Tests (Multi-File)
```bash
cd crates/rfsource-store
cargo test multi_file::tests::test_checksum
```

**Expected:** 4/4 tests pass
- test_checksum_during_write
- test_checksum_verification_on_read
- test_checksum_verification_detects_corruption
- test_rebuild_manifest_from_footers

---

## Step 6: Manual Validation

### Test Checksum Calculation
```rust
use std::fs::File;
use std::io::Write;
use rfsource_format::checksum::{ChecksumWriter, checksum_file};

// Write test file with checksum
let file = File::create("test.dat")?;
let mut writer = ChecksumWriter::new(file);
writer.write_all(b"Hello, RFSource!")?;
let (_, checksum, size) = writer.finalize();

println!("Checksum: {}", checksum);
println!("Size: {} bytes", size);

// Verify via file-level API
let checksum2 = checksum_file("test.dat")?;
assert_eq!(checksum, checksum2);
```

### Test Corruption Detection
```rust
// Create segment with checksum
let mut repo = MultiFileRepo::open_or_init("test_repo")?;
repo.append_frame(&test_frame)?;
repo.rotate_segment()?;

// Corrupt segment file
let mut file = OpenOptions::new()
    .append(true)
    .open("test_repo/segment_00000.rfsource")?;
file.write_all(b"CORRUPTED")?;

// Try to read - should fail
let result = repo.read_frames_with_verification(&[0]);
assert!(result.is_err());
```

### Test Manifest Rebuild
```rust
// Create repo with multiple segments
let mut repo = MultiFileRepo::open_or_init("test_repo")?;
// ... write many frames, rotate multiple times ...

// Delete manifest
std::fs::remove_file("test_repo/.rfsource.manifest")?;

// Rebuild from footers
let manifest = rebuild_manifest_from_footers(Path::new("test_repo"))?;
manifest.save(Path::new("test_repo/.rfsource.manifest"))?;

// Reopen repo - should work
let repo2 = MultiFileRepo::open_or_init("test_repo")?;
assert_eq!(repo2.manifest.segments.len(), repo.manifest.segments.len());
```

---

## Validation Criteria (Day 5)

### Must Pass:
- [ ] All 9 checksum unit tests pass
- [ ] All 4 checksum integration tests pass
- [ ] Checksums calculated during segment rotation
- [ ] Checksums stored in manifest and footers
- [ ] Verification detects corrupted data
- [ ] Manifest rebuild from footers succeeds
- [ ] No performance degradation (<2% overhead)

### Success Metrics:
- **Checksum calculation:** <100ms for 1GB segment
- **Verification overhead:** <5% on read path
- **Rebuild time:** <5 seconds for 100 segments

---

## Common Issues

### Issue: Checksum Mismatch After Write
**Cause:** Footer written after checksum calculation
**Fix:** Calculate checksum before writing footer, or exclude footer from checksum

### Issue: Slow Checksum Calculation
**Cause:** Small buffer size in checksum_file()
**Fix:** Use 1MB buffer (already in design)

### Issue: Rebuild Fails to Find Footers
**Cause:** Segment files without footers (old format)
**Fix:** Skip files without valid footers, log warnings

---

## Next Steps

After Day 5 validation passes:
- **Day 6:** Phase 2 integration testing + validation gate
- Verify end-to-end: write → rotate → verify → corrupt → detect
- Performance benchmarks with checksums enabled
- Documentation update

---

## Files Modified Summary

| File | Changes | Lines Added |
|------|---------|-------------|
| `Cargo.toml` | Add blake3, hex deps | 2 |
| `crates/rfsource-format/src/checksum.rs` | **NEW MODULE** | 400 |
| `crates/rfsource-format/src/lib.rs` | Export checksum | 2 |
| `crates/rfsource-store/src/multi_file.rs` | Integrate checksums | ~150 |
| **Total** | | **~554 lines** |

---

## Phase 2 Progress

- [x] Day 4: Footer format design + implementation
- [x] Day 5: BLAKE3 checksumming + manifest rebuild
- [ ] Day 6: Phase 2 integration testing + validation gate

**Phase 2 Completion:** 2/3 days (67%)
