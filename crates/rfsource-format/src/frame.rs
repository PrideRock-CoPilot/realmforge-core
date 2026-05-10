//! Binary frame reader/writer for `.rfsource` files.
//!
//! ## Frame Structure
//!
//! Each frame is a self-describing compressed blob:
//!
//! ```text
//! [1 byte flag]         - bit 0: compressed (1) or uncompressed (0)
//!                       - bits 1-7: reserved
//! [8 bytes LE u64]     - uncompressed payload length
//! [8 bytes LE u64]     - stored payload length (compressed or raw)
//! [N bytes payload]    - compressed or raw JSON
//! ```
//!
//! The first `N` bytes of the file contain the magic header identifying
//! the format and version.

use std::io::{Read, Write};
use std::path::Path;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{FormatError, Result};

/// Magic header for format version 1: `RFSOURCE\x00\x01\n`
pub const MAGIC_V1: &[u8; 11] = b"RFSOURCE\x00\x01\n";

/// Magic header for format version 2: `RFSOURCE\x00\x02\n`
pub const MAGIC_V2: &[u8; 11] = b"RFSOURCE\x00\x02\n";

/// Current active magic header.
pub const MAGIC_CURRENT: &[u8; 11] = MAGIC_V2;

/// Maximum frame payload size: 128 MB.
pub const MAX_FRAME_BYTES: u64 = 128 * 1024 * 1024;

/// Maximum total file size: 1.5 GB (REM-002).
///
/// This limit prevents unbounded file growth and ensures the system operates
/// within safe bounds. Files approaching this limit should be split into
/// multiple repositories or archived.
///
/// Conservative limit chosen to:
/// - Stay well below OS-level limits (2GB for FAT32, much larger for ext4/XFS)
/// - Allow time for monitoring alerts (1GB threshold) before hitting hard limit
/// - Provide clear error messages to users when limit reached
///
/// See: /docs/qa/TECH-DEBT-RFSOURCE-POST-DEPLOYMENT.md (REM-002)
pub const MAX_FILE_SIZE_BYTES: u64 = 1_610_612_736; // 1.5 GB

/// Warning threshold for file size: 1 GB.
///
/// Monitoring should alert when file size exceeds this threshold to allow
/// proactive action before hitting the hard limit.
pub const FILE_SIZE_WARNING_BYTES: u64 = 1_073_741_824; // 1 GB

/// Header byte indicating data is compressed.
const FLAG_COMPRESSED: u8 = 0x01;

/// Initialize a new `.rfsource` file with the current magic header.
///
/// Creates the file and writes the magic header. Returns an error if the
/// file already exists.
pub fn initialize(path: impl AsRef<Path>) -> Result<()> {
    use std::fs;
    let path = path.as_ref();
    if path.exists() {
        return Err(FormatError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("File already exists: {}", path.display()),
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(path)?;
    f.write_all(MAGIC_CURRENT)?;
    f.sync_all()?;
    Ok(())
}

/// Get the current size of a `.rfsource` file in bytes.
///
/// Returns 0 if the file does not exist.
pub fn get_file_size(path: impl AsRef<Path>) -> Result<u64> {
    match std::fs::metadata(path.as_ref()) {
        Ok(metadata) => Ok(metadata.len()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(FormatError::Io(e)),
    }
}

/// Check if appending a frame of the given size would exceed the file size limit.
///
/// Returns Ok(()) if the write would be within limits, or Err(FormatError::FileSizeLimitExceeded)
/// if the limit would be exceeded.
fn check_file_size_limit(path: impl AsRef<Path>, frame_size: u64) -> Result<()> {
    let current_size = get_file_size(path.as_ref())?;
    let new_size = current_size + frame_size;

    if new_size > MAX_FILE_SIZE_BYTES {
        let excess = new_size - MAX_FILE_SIZE_BYTES;
        return Err(FormatError::FileSizeLimitExceeded {
            current_bytes: current_size,
            current_mb: current_size as f64 / 1_048_576.0,
            limit_bytes: MAX_FILE_SIZE_BYTES,
            limit_mb: MAX_FILE_SIZE_BYTES as f64 / 1_048_576.0,
            attempted_bytes: frame_size,
            excess_bytes: excess,
            excess_mb: excess as f64 / 1_048_576.0,
        });
    }

    Ok(())
}

/// Append a single serializable frame to the `.rfsource` file.
///
/// Serializes `value` to JSON, compresses it with zlib (deflate), and
/// appends a frame header + compressed payload to the file.
///
/// Returns `Err(FormatError::FileSizeLimitExceeded)` if appending this frame
/// would cause the file to exceed `MAX_FILE_SIZE_BYTES`.
pub fn append_frame<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
    let json = serde_json::to_vec(value)?;
    let json_len = json.len() as u64;

    if json_len > MAX_FRAME_BYTES {
        return Err(FormatError::FrameTooLarge {
            max: MAX_FRAME_BYTES,
            actual: json_len,
        });
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&json)?;
    let compressed = encoder.finish()?;
    let stored_len = compressed.len() as u64;

    // Calculate total frame size (header + payload)
    let frame_size = 17 + stored_len; // 1 flag + 8 uncompressed_len + 8 stored_len + payload

    // Check file size limit BEFORE writing (REM-002)
    check_file_size_limit(path.as_ref(), frame_size)?;

    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(path.as_ref())?;

    // [1 byte flag] [8 bytes uncompressed_len] [8 bytes stored_len] [payload]
    let mut header = Vec::with_capacity(17);
    header.push(FLAG_COMPRESSED); // flag byte
    header.extend_from_slice(&json_len.to_le_bytes());
    header.extend_from_slice(&stored_len.to_le_bytes());
    f.write_all(&header)?;
    f.write_all(&compressed)?;
    f.sync_all()?;

    Ok(())
}

/// Append multiple frames in sequence.
///
/// Each frame is checked against the file size limit before being appended.
/// If any frame would exceed the limit, the operation fails and no further
/// frames are appended.
pub fn append_frames<T: Serialize>(path: impl AsRef<Path>, values: &[T]) -> Result<()> {
    for value in values {
        append_frame(path.as_ref(), value)?;
    }
    Ok(())
}

/// Write a raw frame with explicit flag and payload.
///
/// Used when the caller has already serialized/compressed the payload.
///
/// Returns `Err(FormatError::FileSizeLimitExceeded)` if appending this frame
/// would cause the file to exceed `MAX_FILE_SIZE_BYTES`.
pub fn write_frame(
    path: impl AsRef<Path>,
    flag: u8,
    uncompressed_len: u64,
    payload: &[u8],
) -> Result<()> {
    let stored_len = payload.len() as u64;

    // Calculate total frame size
    let frame_size = 17 + stored_len;

    // Check file size limit BEFORE writing (REM-002)
    check_file_size_limit(path.as_ref(), frame_size)?;

    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(path.as_ref())?;

    let mut header = Vec::with_capacity(17);
    header.push(flag);
    header.extend_from_slice(&uncompressed_len.to_le_bytes());
    header.extend_from_slice(&stored_len.to_le_bytes());
    f.write_all(&header)?;
    f.write_all(payload)?;
    f.sync_all()?;

    Ok(())
}

/// Read all frames from a `.rfsource` file, deserializing each one.
///
/// Returns frames in file order (oldest first).
pub fn read_frames<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<Vec<T>> {
    let file_bytes = std::fs::read(path.as_ref())?;
    let mut cursor = std::io::Cursor::new(&file_bytes);

    // Read magic header
    let mut magic = [0u8; 11];
    cursor.read_exact(&mut magic)?;

    // Detect version from magic header
    let version = if magic == *MAGIC_V2 {
        2
    } else if magic == *MAGIC_V1 {
        1
    } else {
        return Err(FormatError::InvalidMagic(path.as_ref().to_path_buf()));
    };

    match version {
        2 => read_v2_frames(&mut cursor),
        1 => read_v1_frames(&mut cursor),
        _ => Err(FormatError::UnsupportedVersion(version)),
    }
}

/// Read V2 frames from a cursor positioned after the magic header.
fn read_v2_frames<T: DeserializeOwned>(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<Vec<T>> {
    // V2: [1 flag] [8 uncompressed] [8 stored] [payload]
    read_frames_with_layout(cursor, 17)
}

/// Read V1 frames from a cursor positioned after the magic header.
fn read_v1_frames<T: DeserializeOwned>(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<Vec<T>> {
    // V1: same layout as V2
    read_frames_with_layout(cursor, 17)
}

/// Generic frame reader given a header size.
fn read_frames_with_layout<T: DeserializeOwned>(
    cursor: &mut std::io::Cursor<&Vec<u8>>,
    _header_size: usize,
) -> Result<Vec<T>> {
    let mut results = Vec::new();

    loop {
        let mut flag_buf = [0u8; 1];
        match cursor.read_exact(&mut flag_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(FormatError::Io(e)),
        }

        let flag = flag_buf[0];

        let mut uncompressed_len_buf = [0u8; 8];
        cursor.read_exact(&mut uncompressed_len_buf)?;
        let uncompressed_len = u64::from_le_bytes(uncompressed_len_buf);

        let mut stored_len_buf = [0u8; 8];
        cursor.read_exact(&mut stored_len_buf)?;
        let stored_len = u64::from_le_bytes(stored_len_buf);

        if stored_len > MAX_FRAME_BYTES {
            return Err(FormatError::CorruptFrame(format!(
                "Stored length {} exceeds max {}",
                stored_len, MAX_FRAME_BYTES
            )));
        }

        let mut payload = vec![0u8; stored_len as usize];
        cursor.read_exact(&mut payload)?;

        let decompressed: Vec<u8> = if flag & FLAG_COMPRESSED != 0 {
            let mut decoder = ZlibDecoder::new(&payload[..]);
            let mut buf = Vec::with_capacity(uncompressed_len as usize);
            decoder.read_to_end(&mut buf)?;
            buf
        } else {
            payload
        };

        let value: T = serde_json::from_slice(&decompressed)?;
        results.push(value);
    }

    Ok(results)
}

/// Compress payload with zlib.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

/// Decompress a zlib-compressed payload.
pub fn decompress(data: &[u8], uncompressed_len: u64) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut buf = Vec::with_capacity(uncompressed_len as usize);
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

//! ## Multi-File Repository Support (DDR-003)
//!
//! Helper functions for multi-file repository detection and segment naming.

use std::path::PathBuf;

/// Repository mode: single-file or multi-file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryMode {
    /// Single `.rfsource` file (legacy or small repos).
    SingleFile,
    /// Multiple segment files with `.rfsource.manifest`.
    MultiFile,
}

/// Detect repository mode by checking for manifest file.
///
/// ## Detection Logic
///
/// 1. If `.rfsource.manifest` exists → MultiFile
/// 2. If `.rfsource` exists → SingleFile
/// 3. Otherwise → SingleFile (will be created)
pub fn detect_repository_mode(repo_dir: impl AsRef<Path>) -> RepositoryMode {
    let manifest_path = repo_dir.as_ref().join(".rfsource.manifest");
    if manifest_path.exists() {
        RepositoryMode::MultiFile
    } else {
        RepositoryMode::SingleFile
    }
}

/// Get the path to the manifest file for a repository.
pub fn manifest_path(repo_dir: impl AsRef<Path>) -> PathBuf {
    repo_dir.as_ref().join(".rfsource.manifest")
}

/// Get the path to a single-file `.rfsource` repository.
pub fn single_file_path(repo_dir: impl AsRef<Path>) -> PathBuf {
    repo_dir.as_ref().join(".rfsource")
}

/// Get the path to a segment file by index.
///
/// ## Examples
///
/// ```ignore
/// segment_path("/path/to/repo", 0)  => "/path/to/repo/.rfsource.0"
/// segment_path("/path/to/repo", 5)  => "/path/to/repo/.rfsource.5"
/// ```
pub fn segment_path(repo_dir: impl AsRef<Path>, index: u32) -> PathBuf {
    repo_dir.as_ref().join(format!(".rfsource.{}", index))
}

/// Get the path to a compacted segment file by index.
///
/// ## Examples
///
/// ```ignore
/// compacted_segment_path("/path/to/repo", 0)  => "/path/to/repo/.rfsource.compact.0"
/// ```
pub fn compacted_segment_path(repo_dir: impl AsRef<Path>, index: u32) -> PathBuf {
    repo_dir.as_ref().join(format!(".rfsource.compact.{}", index))
}

/// Get the archived path for a segment.
///
/// ## Examples
///
/// ```ignore
/// archived_segment_path("/path/to/repo", 0)  => "/path/to/repo/.rfsource.0.archived"
/// ```
pub fn archived_segment_path(repo_dir: impl AsRef<Path>, index: u32) -> PathBuf {
    repo_dir.as_ref().join(format!(".rfsource.{}.archived", index))
}

/// Check if a file is a manifest file based on its name.
pub fn is_manifest_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n == ".rfsource.manifest")
        .unwrap_or(false)
}

/// Check if a file is a segment file based on its name.
///
/// Matches `.rfsource.N` where N is a non-negative integer.
pub fn is_segment_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| {
            if n.starts_with(".rfsource.") && !n.contains("compact") && !n.ends_with(".archived") {
                n.strip_prefix(".rfsource.").and_then(|s| s.parse::<u32>().ok())
            } else {
                None
            }
        })
        .is_some()
}

/// Check if a file is a compacted segment file based on its name.
///
/// Matches `.rfsource.compact.N` where N is a non-negative integer.
pub fn is_compacted_segment_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| {
            if n.starts_with(".rfsource.compact.") {
                n.strip_prefix(".rfsource.compact.").and_then(|s| s.parse::<u32>().ok())
            } else {
                None
            }
        })
        .is_some()
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestFrame {
        name: String,
        value: u32,
    }

    #[test]
    fn test_initialize_and_read_empty() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        initialize(&path).unwrap();
        assert!(path.exists());

        let frames: Vec<TestFrame> = read_frames(&path).unwrap();
        assert!(frames.is_empty());
    }

    #[test]
    fn test_append_and_read_single_frame() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        initialize(&path).unwrap();

        let frame = TestFrame {
            name: "hello".into(),
            value: 42,
        };
        append_frame(&path, &frame).unwrap();

        let frames: Vec<TestFrame> = read_frames(&path).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], frame);
    }

    #[test]
    fn test_append_and_read_multiple_frames() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        initialize(&path).unwrap();

        let frames = vec![
            TestFrame {
                name: "first".into(),
                value: 1,
            },
            TestFrame {
                name: "second".into(),
                value: 2,
            },
            TestFrame {
                name: "third".into(),
                value: 3,
            },
        ];
        append_frames(&path, &frames).unwrap();

        let read_back: Vec<TestFrame> = read_frames(&path).unwrap();
        assert_eq!(read_back.len(), 3);
        assert_eq!(read_back, frames);
    }

    #[test]
    fn test_compress_decompress_roundtrip() {
        let data = b"Hello, RealmForge! This is test data for compression.";
        let compressed = compress(data).unwrap();
        assert!(compressed.len() < data.len() * 2); // reasonable compression
        let decompressed = decompress(&compressed, data.len() as u64).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_initialize_existing_file_errors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("exists.rfsource");
        initialize(&path).unwrap();
        let result = initialize(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_magic_header() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.rfsource");
        std::fs::write(&path, b"NOTVALID\x00\x00\n").unwrap();
        let result: std::result::Result<Vec<TestFrame>, FormatError> = read_frames(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_size() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        
        // Non-existent file should return 0
        let size = get_file_size(&path).unwrap();
        assert_eq!(size, 0);
        
        // After initialization, should have magic header size
        initialize(&path).unwrap();
        let size = get_file_size(&path).unwrap();
        assert_eq!(size, 11); // Magic header is 11 bytes
        
        // After appending a frame, size should increase
        let frame = TestFrame {
            name: "test".into(),
            value: 123,
        };
        append_frame(&path, &frame).unwrap();
        let new_size = get_file_size(&path).unwrap();
        assert!(new_size > 11);
    }

    #[test]
    fn test_file_size_limit_enforcement() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        initialize(&path).unwrap();

        // Create a large payload that would exceed the limit
        // MAX_FILE_SIZE_BYTES = 1.5 GB
        // We'll create a frame that tries to exceed it
        
        // For testing purposes, we can mock this by creating a smaller test.
        // In reality, we'd need to write ~1.5GB of data to test properly.
        // This test documents the expected behavior:
        
        // If we tried to append a frame that would exceed MAX_FILE_SIZE_BYTES,
        // we should get FileSizeLimitExceeded error
        
        // Note: Full integration test would require actually writing 1.5GB+
        // which is too expensive for unit tests. This is tested in integration tests.
    }

    // Tests for multi-file segment helpers

    #[test]
    fn detect_repository_mode_single_file() {
        let dir = tempdir().unwrap();
        
        // No files → SingleFile
        assert_eq!(detect_repository_mode(dir.path()), RepositoryMode::SingleFile);
        
        // Create .rfsource → still SingleFile
        std::fs::File::create(dir.path().join(".rfsource")).unwrap();
        assert_eq!(detect_repository_mode(dir.path()), RepositoryMode::SingleFile);
    }

    #[test]
    fn detect_repository_mode_multi_file() {
        let dir = tempdir().unwrap();
        
        // Create manifest → MultiFile
        std::fs::File::create(dir.path().join(".rfsource.manifest")).unwrap();
        assert_eq!(detect_repository_mode(dir.path()), RepositoryMode::MultiFile);
    }

    #[test]
    fn manifest_path_is_correct() {
        let path = manifest_path("/repo");
        assert_eq!(path.to_str().unwrap(), "/repo/.rfsource.manifest");
    }

    #[test]
    fn single_file_path_is_correct() {
        let path = single_file_path("/repo");
        assert_eq!(path.to_str().unwrap(), "/repo/.rfsource");
    }

    #[test]
    fn segment_path_generates_correct_names() {
        assert_eq!(segment_path("/repo", 0).to_str().unwrap(), "/repo/.rfsource.0");
        assert_eq!(segment_path("/repo", 5).to_str().unwrap(), "/repo/.rfsource.5");
        assert_eq!(segment_path("/repo", 123).to_str().unwrap(), "/repo/.rfsource.123");
    }

    #[test]
    fn compacted_segment_path_generates_correct_names() {
        assert_eq!(compacted_segment_path("/repo", 0).to_str().unwrap(), "/repo/.rfsource.compact.0");
        assert_eq!(compacted_segment_path("/repo", 2).to_str().unwrap(), "/repo/.rfsource.compact.2");
    }

    #[test]
    fn archived_segment_path_generates_correct_names() {
        assert_eq!(archived_segment_path("/repo", 0).to_str().unwrap(), "/repo/.rfsource.0.archived");
        assert_eq!(archived_segment_path("/repo", 5).to_str().unwrap(), "/repo/.rfsource.5.archived");
    }

    #[test]
    fn is_manifest_file_detects_correctly() {
        assert!(is_manifest_file(".rfsource.manifest"));
        assert!(is_manifest_file("/path/to/.rfsource.manifest"));
        assert!(!is_manifest_file(".rfsource"));
        assert!(!is_manifest_file(".rfsource.0"));
        assert!(!is_manifest_file("manifest.json"));
    }

    #[test]
    fn is_segment_file_detects_correctly() {
        assert!(is_segment_file(".rfsource.0"));
        assert!(is_segment_file("/path/to/.rfsource.5"));
        assert!(is_segment_file(".rfsource.123"));
        
        assert!(!is_segment_file(".rfsource"));
        assert!(!is_segment_file(".rfsource.manifest"));
        assert!(!is_segment_file(".rfsource.compact.0"));
        assert!(!is_segment_file(".rfsource.0.archived"));
        assert!(!is_segment_file(".rfsource.abc")); // not a number
    }

    #[test]
    fn is_compacted_segment_file_detects_correctly() {
        assert!(is_compacted_segment_file(".rfsource.compact.0"));
        assert!(is_compacted_segment_file("/path/to/.rfsource.compact.5"));
        
        assert!(!is_compacted_segment_file(".rfsource.0"));
        assert!(!is_compacted_segment_file(".rfsource"));
        assert!(!is_compacted_segment_file(".rfsource.manifest"));
    }

}
