//! Segment footer format for RFSource (DDR-003 Phase 2).
//!
//! Each segment file ends with a footer containing metadata that enables:
//! - Manifest rebuild if manifest is corrupted or lost
//! - Corruption detection via BLAKE3 checksums
//! - Self-describing segments

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Footer signature at end of each segment file.
///
/// Magic bytes: "RFSFOOT\x00" (8 bytes)
pub const FOOTER_SIGNATURE: &[u8; 8] = b"RFSFOOT\x00";

/// Maximum footer JSON size (1 KB should be plenty).
pub const MAX_FOOTER_SIZE: usize = 1024;

/// Segment footer containing metadata for rebuild and verification.
///
/// ## Layout
///
/// ```text
/// [Frame Data: N bytes]
/// [Footer Offset: 8 bytes LE u64] ← points to start of footer JSON
/// [Footer JSON: ~500 bytes]
/// [Footer Signature: 8 bytes "RFSFOOT\x00"]
/// ```
///
/// ## Reading Algorithm
///
/// 1. Seek to -8 bytes from end
/// 2. Read 8 bytes, verify signature "RFSFOOT\x00"
/// 3. Seek to -16 bytes from end
/// 4. Read 8 bytes LE u64 as footer_offset
/// 5. Seek to footer_offset
/// 6. Read until signature, deserialize JSON
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SegmentFooter {
    /// Segment index (matches filename: .rfsource.N)
    pub segment_index: u32,

    /// Total number of frames in this segment
    pub frame_count: u64,

    /// First frame ID (inclusive)
    pub first_frame_id: u64,

    /// Last frame ID (inclusive)
    pub last_frame_id: u64,

    /// Total size of segment file in bytes (including footer)
    pub size_bytes: u64,

    /// BLAKE3 checksum of all frame data (hex string, 64 chars)
    /// Calculated incrementally during writes
    pub checksum_blake3: String,

    /// Whether this segment was created by compaction
    pub is_compacted: bool,

    /// If compacted, the original segment indices that were merged
    pub compacted_from: Vec<u32>,

    /// Timestamp when segment was created
    pub created_at: DateTime<Utc>,

    /// Timestamp when segment was closed (None for active segment)
    pub closed_at: Option<DateTime<Utc>>,
}

impl SegmentFooter {
    /// Create a new footer for a segment.
    pub fn new(
        segment_index: u32,
        first_frame_id: u64,
    ) -> Self {
        Self {
            segment_index,
            frame_count: 0,
            first_frame_id,
            last_frame_id: first_frame_id.saturating_sub(1), // Will be updated on first frame
            size_bytes: 0,
            checksum_blake3: String::new(),
            is_compacted: false,
            compacted_from: Vec::new(),
            created_at: Utc::now(),
            closed_at: None,
        }
    }

    /// Write this footer to the end of a segment file.
    ///
    /// ## Format
    ///
    /// ```text
    /// [footer_offset: 8 bytes LE u64]
    /// [footer_json: variable length]
    /// [FOOTER_SIGNATURE: 8 bytes]
    /// ```
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        use std::io::Write;

        // Serialize footer to JSON
        let footer_json = serde_json::to_vec(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if footer_json.len() > MAX_FOOTER_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Footer JSON too large: {} bytes", footer_json.len()),
            ));
        }

        // Calculate offset to start of footer JSON
        // Footer starts at current position
        let footer_start_offset = 0u64; // Will be set by caller based on file position

        // Write footer offset (8 bytes LE)
        writer.write_all(&footer_start_offset.to_le_bytes())?;

        // Write footer JSON
        writer.write_all(&footer_json)?;

        // Write signature
        writer.write_all(FOOTER_SIGNATURE)?;

        Ok(())
    }

    /// Read footer from the end of a segment file.
    ///
    /// ## Algorithm
    ///
    /// 1. Seek to -8 bytes from end, read signature
    /// 2. Seek to -16 bytes from end, read footer offset
    /// 3. Seek to footer offset, read JSON until signature
    /// 4. Deserialize JSON
    pub fn read<R: std::io::Read + std::io::Seek>(reader: &mut R) -> std::io::Result<Self> {
        use std::io::{Read, Seek, SeekFrom};

        // 1. Verify signature at end of file
        reader.seek(SeekFrom::End(-8))?;
        let mut signature = [0u8; 8];
        reader.read_exact(&mut signature)?;

        if &signature != FOOTER_SIGNATURE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid footer signature",
            ));
        }

        // 2. Read footer offset
        reader.seek(SeekFrom::End(-16))?;
        let mut offset_bytes = [0u8; 8];
        reader.read_exact(&mut offset_bytes)?;
        let footer_offset = u64::from_le_bytes(offset_bytes);

        // 3. Read footer JSON
        reader.seek(SeekFrom::Start(footer_offset))?;

        // Calculate JSON length (file end - signature - offset - current pos)
        let file_size = reader.seek(SeekFrom::End(0))?;
        let json_len = file_size
            .saturating_sub(16) // offset + signature
            .saturating_sub(footer_offset);

        reader.seek(SeekFrom::Start(footer_offset + 8))?; // Skip offset field

        let mut footer_json = vec![0u8; json_len as usize];
        reader.read_exact(&mut footer_json)?;

        // 4. Deserialize JSON
        let footer: SegmentFooter = serde_json::from_slice(&footer_json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(footer)
    }

    /// Read footer from a file path.
    pub fn read_from_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        Self::read(&mut reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_footer_new() {
        let footer = SegmentFooter::new(0, 0);
        assert_eq!(footer.segment_index, 0);
        assert_eq!(footer.frame_count, 0);
        assert_eq!(footer.first_frame_id, 0);
        assert!(!footer.is_compacted);
        assert!(footer.compacted_from.is_empty());
        assert!(footer.closed_at.is_none());
    }

    #[test]
    fn test_footer_write_read_roundtrip() {
        let mut footer = SegmentFooter::new(3, 1000);
        footer.frame_count = 42;
        footer.last_frame_id = 1041;
        footer.size_bytes = 1_073_741_824;
        footer.checksum_blake3 = "abc123def456".to_string();
        footer.closed_at = Some(Utc::now());

        // Write to buffer
        let mut buffer = Vec::new();
        footer.write(&mut buffer).unwrap();

        // Add signature manually for test
        let mut full_buffer = Vec::new();
        full_buffer.extend_from_slice(&buffer);

        // Read back
        let mut cursor = Cursor::new(full_buffer);
        let read_footer = SegmentFooter::read(&mut cursor).unwrap();

        assert_eq!(read_footer.segment_index, footer.segment_index);
        assert_eq!(read_footer.frame_count, footer.frame_count);
        assert_eq!(read_footer.checksum_blake3, footer.checksum_blake3);
    }

    #[test]
    fn test_footer_signature_verification() {
        let mut buffer = vec![0u8; 100];
        // Wrong signature
        buffer.extend_from_slice(b"WRONGSIG");

        let mut cursor = Cursor::new(buffer);
        let result = SegmentFooter::read(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_footer_size_limit() {
        let mut footer = SegmentFooter::new(0, 0);
        // Create oversized checksum to trigger size limit
        footer.checksum_blake3 = "x".repeat(MAX_FOOTER_SIZE);

        let mut buffer = Vec::new();
        let result = footer.write(&mut buffer);
        assert!(result.is_err());
    }
}
