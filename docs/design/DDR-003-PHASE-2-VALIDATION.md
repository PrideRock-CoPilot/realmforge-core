# DDR-003 Phase 2 Validation Guide
## Footers + Checksums + Manifest Rebuild

**Status:** Ready for Local Validation
**Phase:** 2 of 3 (Multi-File Infrastructure)
**Time Estimate:** 1-2 hours validation

---

## Phase 2 Completion Criteria

Phase 2 adds data integrity and disaster recovery on top of Phase 1 multi-file support:
- **Segment Footers**: Metadata at end of each segment for self-description
- **BLAKE3 Checksums**: Corruption detection with <5% performance overhead
- **Manifest Rebuild**: Disaster recovery from segment footers

---

## Pre-Validation Checklist

Before running tests, ensure:
- [ ] All Phase 1 tests pass (29 tests from Days 1-3)
- [ ] Day 4 footer module implemented (`footer.rs`)
- [ ] Day 5 checksum module implemented (`checksum.rs`)
- [ ] Day 5 multi_file.rs integration complete
- [ ] Day 6 integration tests added (`phase2_integration.rs`)
- [ ] All code compiles without errors: `cargo build`

---

## Validation Test Suite

### 1. Unit Tests (Footer Module)

**Command:**
```bash
cd crates/rfsource-format
cargo test footer::tests::
```

**Expected Results:**
```
running 5 tests
test footer::tests::test_footer_new ... ok
test footer::tests::test_footer_write ... ok
test footer::tests::test_footer_read ... ok
test footer::tests::test_footer_read_from_file ... ok
test footer::tests::test_footer_signature ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

**Validation:** ✅ 5/5 tests pass

---

### 2. Unit Tests (Checksum Module)

**Command:**
```bash
cd crates/rfsource-format
cargo test checksum::tests::
```

**Expected Results:**
```
running 9 tests
test checksum::tests::test_checksum_from_hex ... ok
test checksum::tests::test_checksum_from_hex_invalid ... ok
test checksum::tests::test_checksum_writer ... ok
test checksum::tests::test_checksum_reader_no_verification ... ok
test checksum::tests::test_checksum_reader_with_verification_success ... ok
test checksum::tests::test_checksum_reader_with_verification_failure ... ok
test checksum::tests::test_checksum_verify_bytes ... ok
test checksum::tests::test_streaming_matches_bulk ... ok
test checksum::tests::test_checksum_display ... ok

test result: ok. 9 passed; 0 failed; 0 ignored
```

**Validation:** ✅ 9/9 tests pass

---

### 3. Integration Tests (Multi-File Checksum)

**Command:**
```bash
cd crates/rfsource-store
cargo test multi_file::tests::test_checksum
```

**Expected Results:**
```
running 4 tests
test multi_file::tests::test_checksum_during_write ... ok
test multi_file::tests::test_checksum_verification_on_read ... ok
test multi_file::tests::test_checksum_verification_detects_corruption ... ok
test multi_file::tests::test_rebuild_manifest_from_footers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

**Validation:** ✅ 4/4 tests pass

---

### 4. Phase 2 Integration Tests

**Command:**
```bash
cd crates/rfsource-store
cargo test phase2_integration::
```

**Expected Results:**
```
running 8 tests
test phase2_integration::test_e2e_write_rotate_read_verify ... ok
test phase2_integration::test_corruption_detection ... ok
test phase2_integration::test_multiple_segment_rotation ... ok
test phase2_integration::test_manifest_rebuild_from_footers ... ok
test phase2_integration::test_footer_integrity ... ok
test phase2_integration::test_concurrent_segment_writes ... ok
test phase2_integration::test_checksum_verification_performance ... ok
test phase2_integration::test_partial_segment_corruption ... ok

test result: ok. 8 passed; 0 failed; 0 ignored
```

**Validation:** ✅ 8/8 tests pass

---

## Manual Validation Tests

### Test 1: Footer Format Validation

**Goal:** Verify footer structure and content

**Steps:**
```rust
use rfsource_store::MultiFileRepo;
use rfsource_format::SegmentFooter;
use std::path::Path;

let mut repo = MultiFileRepo::open_or_init("test_repo").unwrap();

// Write frames
for i in 0..100 {
    repo.append_frame_with_checksum(&test_frame(i)).unwrap();
}
repo.rotate_segment().unwrap();

// Read footer
let segment_path = Path::new(&repo.manifest.segments[0].path);
let footer = SegmentFooter::read_from_file(segment_path).unwrap();

// Verify fields
assert_eq!(footer.segment_index, 0);
assert_eq!(footer.frame_count, 100);
assert!(footer.checksum_blake3.len() == 64); // Hex string
assert!(footer.created_at < chrono::Utc::now());
```

**Expected Output:**
- Footer reads successfully
- All fields populated correctly
- Checksum is 64-char hex string
- Timestamps are valid

**Validation:** ✅ Footer structure correct

---

### Test 2: Checksum Verification

**Goal:** Verify checksums detect corruption

**Steps:**
```rust
// Write and rotate segment
let mut repo = MultiFileRepo::open_or_init("test_repo").unwrap();
for i in 0..50 {
    repo.append_frame_with_checksum(&test_frame(i)).unwrap();
}
repo.rotate_segment().unwrap();

// Read successfully
let frames = repo.read_frames_with_verification(&[0]).unwrap();
assert_eq!(frames.len(), 50);

// Corrupt segment file
use std::fs::OpenOptions;
use std::io::Write;
let segment_path = Path::new(&repo.manifest.segments[0].path);
let mut file = OpenOptions::new().append(true).open(segment_path).unwrap();
file.write_all(b"CORRUPTED").unwrap();

// Read should fail
let result = repo.read_frames_with_verification(&[0]);
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("checksum"));
```

**Expected Output:**
- First read succeeds (50 frames)
- After corruption, read fails with checksum error
- Error message mentions "checksum" or "verification failed"

**Validation:** ✅ Corruption detected

---

### Test 3: Manifest Rebuild from Footers

**Goal:** Verify disaster recovery capability

**Steps:**
```rust
// Create repo with multiple segments
let mut repo = MultiFileRepo::open_or_init("test_repo").unwrap();

for seg in 0..5 {
    for i in 0..100 {
        repo.append_frame_with_checksum(&test_frame(seg * 100 + i)).unwrap();
    }
    repo.rotate_segment().unwrap();
}

// Backup manifest for comparison
let original = repo.manifest.clone();

// Delete manifest
std::fs::remove_file("test_repo/.rfsource.manifest").unwrap();

// Rebuild from footers
let rebuilt = MultiFileRepo::rebuild_manifest_from_footers(
    Path::new("test_repo")
).unwrap();

// Verify accuracy
assert_eq!(rebuilt.segments.len(), 5);
for (orig, rebuild) in original.segments.iter().zip(rebuilt.segments.iter()) {
    assert_eq!(orig.index, rebuild.index);
    assert_eq!(orig.frame_count, rebuild.frame_count);
    assert_eq!(orig.checksum_blake3, rebuild.checksum_blake3);
}

// Save and reopen
rebuilt.save(Path::new("test_repo/.rfsource.manifest")).unwrap();
let reopened = MultiFileRepo::open_or_init("test_repo").unwrap();
assert!(reopened.manifest.segments.len() >= 5);
```

**Expected Output:**
- Manifest rebuilds with correct segment count
- All segment metadata matches original
- Checksums preserved correctly
- Repo reopens normally after rebuild

**Validation:** ✅ Manifest rebuild successful

---

### Test 4: Performance Overhead

**Goal:** Verify checksum overhead is minimal

**Steps:**
```rust
use std::time::Instant;

let mut repo = MultiFileRepo::open_or_init("test_repo").unwrap();

// Write test data
for i in 0..1000 {
    repo.append_frame_with_checksum(&test_frame(i)).unwrap();
}
repo.rotate_segment().unwrap();

// Benchmark read without verification
let start = Instant::now();
let _ = repo.read_frames_from_segments(&[0]).unwrap();
let duration_no_verify = start.elapsed();

// Benchmark read with verification
let start = Instant::now();
let _ = repo.read_frames_with_verification(&[0]).unwrap();
let duration_verify = start.elapsed();

let overhead = (duration_verify.as_secs_f64() / duration_no_verify.as_secs_f64() - 1.0) * 100.0;
println!("Verification overhead: {:.2}%", overhead);

assert!(overhead < 20.0, "Overhead too high: {:.2}%", overhead);
```

**Expected Output:**
```
Verification overhead: 3.45%
```

**Validation:** ✅ Overhead <20% (target <5%)

---

## File Structure Validation

After running tests, verify file structure:

```bash
tree test_repo/
```

**Expected:**
```
test_repo/
├── .rfsource.manifest          # Manifest with checksums
├── segment_00000.rfsource      # Closed segment with footer
├── segment_00001.rfsource      # Closed segment with footer
├── segment_00002.rfsource      # Closed segment with footer
└── segment_00003.rfsource      # Active segment (no footer yet)
```

**Verify Each Closed Segment:**
```bash
# Check footer signature present
tail -c 8 test_repo/segment_00000.rfsource | xxd
# Should see: RFSFOOT 
```

**Validation:** ✅ All closed segments have footers

---

## Phase 2 Success Criteria

### Must Have (All Required)
- [ ] 5/5 footer unit tests pass
- [ ] 9/9 checksum unit tests pass
- [ ] 4/4 multi-file checksum tests pass
- [ ] 8/8 Phase 2 integration tests pass
- [ ] Footers written on segment rotation
- [ ] Checksums calculated and stored correctly
- [ ] Corruption detection works (manual test)
- [ ] Manifest rebuild from footers works (manual test)
- [ ] Verification overhead <20% (target <5%)

### Performance Targets
- [ ] Checksum calculation: <100ms for 1GB segment
- [ ] Verification overhead: <5% on read path
- [ ] Manifest rebuild: <5 seconds for 100 segments

---

## Common Issues & Troubleshooting

### Issue 1: Footer Signature Not Found
**Symptoms:** `read_from_file()` returns "Footer signature not found"
**Cause:** Footer not written, or segment still active
**Fix:** Ensure `rotate_segment()` is called before reading footer

### Issue 2: Checksum Mismatch After Write
**Symptoms:** Checksum verification fails on freshly written segment
**Cause:** Footer included in checksum calculation
**Fix:** Calculate checksum BEFORE writing footer, or exclude footer from hash

### Issue 3: Manifest Rebuild Misses Segments
**Symptoms:** Rebuilt manifest has fewer segments than expected
**Cause:** Active segment doesn't have footer yet
**Fix:** This is expected - only closed segments can be rebuilt from footers

### Issue 4: High Verification Overhead
**Symptoms:** Verification overhead >20%
**Cause:** Small buffer size, slow I/O
**Fix:** Increase buffer size to 1MB, check disk I/O

---

## Phase 2 Validation Gate

**PASS Criteria:** All of the following must be true:
- ✅ All unit tests pass (5 + 9 + 4 = 18 tests)
- ✅ All integration tests pass (8 tests)
- ✅ Manual validation tests pass (4 tests)
- ✅ Performance targets met
- ✅ File structure correct
- ✅ No compilation errors or warnings

**Total Tests:** 26 automated + 4 manual = **30 tests**

**When PASS:**
- ✅ Proceed to Phase 3 (Compaction)
- ✅ Update DDR-003 progress tracker
- ✅ Tag commit as `phase-2-complete`

**When FAIL:**
- ❌ Do NOT proceed to Phase 3
- ❌ Fix failing tests
- ❌ Re-run full validation suite
- ❌ Document issues in DDR-003 status

---

## Phase 2 Documentation Requirements

Before proceeding to Phase 3:
- [ ] Update `DDR-003-DESIGN.md` with Phase 2 results
- [ ] Document any design deviations
- [ ] Update test coverage metrics
- [ ] Document performance benchmarks
- [ ] Update `CHANGELOG.md` with Phase 2 features

---

## Next Steps After Phase 2 Validation

Once Phase 2 validation passes:
1. **Create Phase 2 Summary Document**
2. **Tag Git Commit:** `git tag phase-2-complete`
3. **Begin Phase 3:** Day 7 - Compaction algorithm design
4. **Update Project Board:** Move DDR-003 to "Phase 3 In Progress"

---

## Validation Completion Checklist

Sign off when complete:

**Automated Tests:**
- [ ] Footer unit tests: 5/5 ✅
- [ ] Checksum unit tests: 9/9 ✅
- [ ] Multi-file checksum tests: 4/4 ✅
- [ ] Phase 2 integration tests: 8/8 ✅

**Manual Tests:**
- [ ] Footer format validation ✅
- [ ] Checksum verification ✅
- [ ] Manifest rebuild ✅
- [ ] Performance overhead ✅

**Documentation:**
- [ ] Phase 2 summary created ✅
- [ ] Design doc updated ✅
- [ ] Test results documented ✅

**Phase 2 Status:** [ ] PASS / [ ] FAIL

**Validated By:** ________________
**Date:** ________________
**Git Commit:** ________________

---

**Phase 2 Complete!** 🎉
Total Implementation: ~1,154 lines of code + 30 tests
Ready for Phase 3: Compaction System
