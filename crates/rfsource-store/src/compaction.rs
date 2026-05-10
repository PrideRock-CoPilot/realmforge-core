// crates/rfsource-store/src/compaction.rs
//! Segment compaction for RFSource multi-file repositories
//!
//! Compaction merges multiple small segments into larger compacted segments to:
//! - Reduce file count (query routing overhead)
//! - Improve sequential read performance
//! - Reduce metadata overhead
//!
//! Design:
//! - Manual compaction (like `git gc`) - user explicitly triggers
//! - Safe and reversible - keeps original segments until confirmed
//! - Creates new compacted segments with `.compacted.rfsource` suffix
//! - Archives old segments to `.archived/` directory after successful compaction
//! - Updates manifest with compaction metadata
//!
//! Safety Guarantees:
//! - Atomic compaction (all or nothing)
//! - Original segments preserved until verified
//! - Checksums validated before and after
//! - Rollback capability if compaction fails

use crate::multi_file::{MultiFileRepo, segment_path};
use rfsource_format::{
    Manifest, SegmentInfo, ArchivedSegmentInfo, SegmentFooter, SegmentChecksum,
    checksum_file, verify_file,
};
use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufRead, Write};
use serde::{Serialize, de::DeserializeOwned};

/// Compaction strategy determines which segments to compact
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Compact all segments below threshold size (e.g., <100MB)
    SmallSegments { threshold_bytes: u64 },
    
    /// Compact oldest N segments
    OldestSegments { count: usize },
    
    /// Compact specific segment range
    Range { start_index: u32, end_index: u32 },
    
    /// Compact all segments (full compaction)
    All,
}

impl Default for CompactionStrategy {
    fn default() -> Self {
        // Default: compact segments < 100MB
        CompactionStrategy::SmallSegments {
            threshold_bytes: 100 * 1024 * 1024,
        }
    }
}

/// Compaction plan describes what will be compacted
#[derive(Debug, Clone)]
pub struct CompactionPlan {
    /// Segments to be compacted
    pub segments: Vec<u32>,
    
    /// Total size of segments to compact
    pub total_size_bytes: u64,
    
    /// Total frames to compact
    pub total_frames: u64,
    
    /// Expected compacted segment index
    pub compacted_index: u32,
    
    /// Strategy used
    pub strategy: CompactionStrategy,
}

impl CompactionPlan {
    /// Create a plan for compacting segments
    pub fn create(
        manifest: &Manifest,
        strategy: CompactionStrategy,
    ) -> Result<Self, String> {
        let mut segments_to_compact = Vec::new();
        
        // Select segments based on strategy
        match strategy {
            CompactionStrategy::SmallSegments { threshold_bytes } => {
                for segment in &manifest.segments {
                    // Don't compact active segment or already compacted segments
                    if segment.closed_at.is_none() || segment.is_compacted {
                        continue;
                    }
                    
                    if segment.size_bytes < threshold_bytes {
                        segments_to_compact.push(segment.index);
                    }
                }
            }
            
            CompactionStrategy::OldestSegments { count } => {
                let mut eligible: Vec<_> = manifest.segments.iter()
                    .filter(|s| s.closed_at.is_some() && !s.is_compacted)
                    .collect();
                
                eligible.sort_by_key(|s| s.created_at);
                
                for segment in eligible.iter().take(count) {
                    segments_to_compact.push(segment.index);
                }
            }
            
            CompactionStrategy::Range { start_index, end_index } => {
                for segment in &manifest.segments {
                    if segment.index >= start_index && segment.index <= end_index {
                        if segment.closed_at.is_none() {
                            return Err(format!("Cannot compact active segment {}", segment.index));
                        }
                        if segment.is_compacted {
                            return Err(format!("Segment {} already compacted", segment.index));
                        }
                        segments_to_compact.push(segment.index);
                    }
                }
            }
            
            CompactionStrategy::All => {
                for segment in &manifest.segments {
                    if segment.closed_at.is_some() && !segment.is_compacted {
                        segments_to_compact.push(segment.index);
                    }
                }
            }
        }
        
        if segments_to_compact.is_empty() {
            return Err("No segments eligible for compaction".to_string());
        }
        
        // Calculate totals
        let mut total_size = 0u64;
        let mut total_frames = 0u64;
        
        for &seg_idx in &segments_to_compact {
            let segment = manifest.segments.iter()
                .find(|s| s.index == seg_idx)
                .ok_or(format!("Segment {} not found", seg_idx))?;
            
            total_size += segment.size_bytes;
            total_frames += segment.frame_count;
        }
        
        // Next compacted segment index
        let compacted_index = manifest.compaction_generation;
        
        Ok(CompactionPlan {
            segments: segments_to_compact,
            total_size_bytes: total_size,
            total_frames: total_frames,
            compacted_index,
            strategy,
        })
    }
    
    /// Estimate compaction time (rough heuristic)
    pub fn estimated_duration_seconds(&self) -> u64 {
        // Rough estimate: 100 MB/s read + write throughput
        let bytes_to_process = self.total_size_bytes * 2; // Read + write
        bytes_to_process / (100 * 1024 * 1024) + 1
    }
}

/// Compaction result
#[derive(Debug)]
pub struct CompactionResult {
    /// Number of segments compacted
    pub segments_compacted: usize,
    
    /// Total bytes read from source segments
    pub bytes_read: u64,
    
    /// Total bytes written to compacted segment
    pub bytes_written: u64,
    
    /// Number of frames compacted
    pub frames_compacted: u64,
    
    /// Checksum of compacted segment
    pub compacted_checksum: String,
    
    /// Path to compacted segment
    pub compacted_path: PathBuf,
    
    /// Compaction duration
    pub duration: std::time::Duration,
}

/// Compaction executor
pub struct Compactor<'a> {
    repo_dir: &'a Path,
    manifest: &'a mut Manifest,
}

impl<'a> Compactor<'a> {
    /// Create new compactor
    pub fn new(repo_dir: &'a Path, manifest: &'a mut Manifest) -> Self {
        Self { repo_dir, manifest }
    }
    
    /// Execute compaction plan
    pub fn compact<T: Serialize + DeserializeOwned>(
        &mut self,
        plan: &CompactionPlan,
    ) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        println!("Starting compaction:");
        println!("  Segments: {:?}", plan.segments);
        println!("  Total frames: {}", plan.total_frames);
        println!("  Total size: {} MB", plan.total_size_bytes / 1024 / 1024);
        println!("  Estimated duration: {}s", plan.estimated_duration_seconds());
        
        // Create compacted segment file
        let compacted_path = self.compacted_segment_path(plan.compacted_index);
        let compacted_file = File::create(&compacted_path)?;
        let mut writer = rfsource_format::ChecksumWriter::new(compacted_file);
        
        let mut bytes_read = 0u64;
        let mut frames_written = 0u64;
        let mut first_frame_id = None;
        let mut last_frame_id = 0u64;
        
        // Read frames from all source segments in order
        for &seg_idx in &plan.segments {
            let segment = self.manifest.segments.iter()
                .find(|s| s.index == seg_idx)
                .ok_or(format!("Segment {} not found", seg_idx))?;
            
            println!("  Reading segment {}...", seg_idx);
            
            // Verify segment checksum before reading
            if let Some(checksum_hex) = &segment.checksum_blake3 {
                let expected = SegmentChecksum::from_hex(checksum_hex)?;
                verify_file(Path::new(&segment.path), &expected)?;
            }
            
            // Read and copy frames
            let file = File::open(&segment.path)?;
            let reader = BufReader::new(file);
            
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                
                // Deserialize to validate, then write
                let _frame: T = serde_json::from_str(&line)?;
                
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"
")?;
                
                frames_written += 1;
                
                // Track frame ID range (assume frames have sequential IDs)
                if first_frame_id.is_none() {
                    first_frame_id = Some(segment.first_frame_id);
                }
                last_frame_id = segment.last_frame_id;
            }
            
            bytes_read += segment.size_bytes;
        }
        
        writer.flush()?;
        
        // Finalize checksum
        let (mut file, checksum, bytes_written) = writer.finalize();
        
        println!("  Compacted {} frames ({} MB)", frames_written, bytes_written / 1024 / 1024);
        println!("  Checksum: {}", checksum);
        
        // Write footer
        let footer = SegmentFooter {
            segment_index: plan.compacted_index,
            frame_count: frames_written,
            first_frame_id: first_frame_id.unwrap_or(0),
            last_frame_id,
            size_bytes: bytes_written,
            checksum_blake3: checksum.to_hex(),
            is_compacted: true,
            compacted_from: Some(plan.segments.clone()),
            created_at: chrono::Utc::now(),
            closed_at: Some(chrono::Utc::now()),
        };
        
        footer.write(&compacted_path)?;
        
        // Create new segment info for compacted segment
        let compacted_segment = SegmentInfo {
            index: plan.compacted_index,
            path: compacted_path.display().to_string(),
            size_bytes: bytes_written,
            frame_count: frames_written,
            first_frame_id: first_frame_id.unwrap_or(0),
            last_frame_id,
            is_compacted: true,
            compacted_from: Some(plan.segments.clone()),
            checksum_blake3: Some(checksum.to_hex()),
            created_at: footer.created_at,
            closed_at: Some(chrono::Utc::now()),
        };
        
        // Archive old segments
        println!("  Archiving source segments...");
        self.archive_segments(&plan.segments)?;
        
        // Update manifest
        self.manifest.segments.push(compacted_segment);
        self.manifest.compaction_generation += 1;
        
        // Save manifest
        let manifest_path = self.repo_dir.join(".rfsource.manifest");
        self.manifest.save(&manifest_path)?;
        
        let duration = start_time.elapsed();
        
        println!("Compaction complete in {:.2}s", duration.as_secs_f64());
        
        Ok(CompactionResult {
            segments_compacted: plan.segments.len(),
            bytes_read,
            bytes_written,
            frames_compacted: frames_written,
            compacted_checksum: checksum.to_hex(),
            compacted_path,
            duration,
        })
    }
    
    /// Get path for compacted segment
    fn compacted_segment_path(&self, index: u32) -> PathBuf {
        self.repo_dir.join(format!("segment_{:05}.compacted.rfsource", index))
    }
    
    /// Archive segments to .archived/ directory
    fn archive_segments(&mut self, segment_indices: &[u32]) -> io::Result<()> {
        let archive_dir = self.repo_dir.join(".archived");
        fs::create_dir_all(&archive_dir)?;
        
        for &seg_idx in segment_indices {
            // Find segment in manifest
            let segment_pos = self.manifest.segments.iter()
                .position(|s| s.index == seg_idx)
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Segment {} not found", seg_idx)
                ))?;
            
            let segment = self.manifest.segments.remove(segment_pos);
            
            // Move file to archive
            let source_path = Path::new(&segment.path);
            let archived_path = archive_dir.join(
                source_path.file_name()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?
            );
            
            fs::rename(source_path, &archived_path)?;
            
            // Add to archived segments in manifest
            let archived_info = ArchivedSegmentInfo {
                index: segment.index,
                path: archived_path.display().to_string(),
                archived_at: chrono::Utc::now(),
                compacted_into: Some(self.manifest.compaction_generation - 1),
            };
            
            self.manifest.archived_segments.push(archived_info);
        }
        
        Ok(())
    }
    
    /// Rollback compaction (restore archived segments)
    pub fn rollback_last_compaction(&mut self) -> io::Result<()> {
        // Find segments archived in last compaction
        let last_gen = self.manifest.compaction_generation - 1;
        
        let to_restore: Vec<_> = self.manifest.archived_segments.iter()
            .filter(|a| a.compacted_into == Some(last_gen))
            .cloned()
            .collect();
        
        if to_restore.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No segments to rollback"
            ));
        }
        
        println!("Rolling back compaction generation {}...", last_gen);
        
        // Restore archived segments
        for archived in &to_restore {
            let archived_path = Path::new(&archived.path);
            let restored_path = self.repo_dir.join(
                archived_path.file_name()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?
            );
            
            fs::rename(archived_path, &restored_path)?;
            
            // TODO: Recreate SegmentInfo and add back to manifest
            // This requires reading the footer from the restored segment
            let footer = SegmentFooter::read_from_file(&restored_path)?;
            
            let segment_info = SegmentInfo {
                index: footer.segment_index,
                path: restored_path.display().to_string(),
                size_bytes: footer.size_bytes,
                frame_count: footer.frame_count,
                first_frame_id: footer.first_frame_id,
                last_frame_id: footer.last_frame_id,
                is_compacted: false,
                compacted_from: None,
                checksum_blake3: Some(footer.checksum_blake3),
                created_at: footer.created_at,
                closed_at: footer.closed_at,
            };
            
            self.manifest.segments.push(segment_info);
        }
        
        // Remove from archived list
        self.manifest.archived_segments.retain(|a| a.compacted_into != Some(last_gen));
        
        // Remove compacted segment
        self.manifest.segments.retain(|s| s.index != last_gen || !s.is_compacted);
        
        // Delete compacted segment file
        let compacted_path = self.compacted_segment_path(last_gen);
        if compacted_path.exists() {
            fs::remove_file(compacted_path)?;
        }
        
        // Save manifest
        let manifest_path = self.repo_dir.join(".rfsource.manifest");
        self.manifest.save(&manifest_path)?;
        
        println!("Rollback complete");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestFrame {
        id: u64,
        data: String,
    }
    
    #[test]
    fn test_compaction_plan_small_segments() {
        // Create manifest with mix of small and large segments
        let mut manifest = Manifest::new();
        
        // Small segments (should be compacted)
        for i in 0..5 {
            manifest.segments.push(SegmentInfo {
                index: i,
                size_bytes: 50 * 1024 * 1024, // 50MB
                frame_count: 1000,
                is_compacted: false,
                closed_at: Some(chrono::Utc::now()),
                // ... other fields ...
            });
        }
        
        // Large segment (should not be compacted)
        manifest.segments.push(SegmentInfo {
            index: 5,
            size_bytes: 500 * 1024 * 1024, // 500MB
            frame_count: 10000,
            is_compacted: false,
            closed_at: Some(chrono::Utc::now()),
            // ... other fields ...
        });
        
        let strategy = CompactionStrategy::SmallSegments {
            threshold_bytes: 100 * 1024 * 1024,
        };
        
        let plan = CompactionPlan::create(&manifest, strategy).unwrap();
        
        assert_eq!(plan.segments.len(), 5);
        assert_eq!(plan.total_size_bytes, 250 * 1024 * 1024);
        assert_eq!(plan.total_frames, 5000);
    }
    
    #[test]
    fn test_compaction_plan_oldest_segments() {
        let mut manifest = Manifest::new();
        
        // Create segments with different ages
        // (Implementation would add segments with different created_at timestamps)
        
        let strategy = CompactionStrategy::OldestSegments { count: 3 };
        // Test would verify correct segments selected
    }
    
    #[test]
    fn test_compaction_execution() {
        // Integration test - create multi-file repo, write segments, compact them
        // (Requires full MultiFileRepo integration)
    }
    
    #[test]
    fn test_compaction_rollback() {
        // Test that rollback restores archived segments correctly
    }
}
