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
use tracing::instrument;

use crate::error::{FormatError, Result};

/// Magic header for format version 1: `RFSOURCE\x00\x01\n`
pub const MAGIC_V1: &[u8; 11] = b"RFSOURCE\x00\x01\n";

/// Magic header for format version 2: `RFSOURCE\x00\x02\n`
pub const MAGIC_V2: &[u8; 11] = b"RFSOURCE\x00\x02\n";

/// Current active magic header.
pub const MAGIC_CURRENT: &[u8; 11] = MAGIC_V2;

/// Maximum frame payload size: 128 MB.
pub const MAX_FRAME_BYTES: u64 = 128 * 1024 * 1024;

/// Header byte indicating data is compressed.
const FLAG_COMPRESSED: u8 = 0x01;

/// Initialize a new `.rfsource` file with the current magic header.
///
/// Creates the file and writes the magic header. Returns an error if the
/// file already exists.
#[instrument]
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

/// Append a single serializable frame to the `.rfsource` file.
///
/// Serializes `value` to JSON, compresses it with zlib (deflate), and
/// appends a frame header + compressed payload to the file.
#[instrument(skip(value))]
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
pub fn append_frames<T: Serialize>(path: impl AsRef<Path>, values: &[T]) -> Result<()> {
    for value in values {
        append_frame(path.as_ref(), value)?;
    }
    Ok(())
}

/// Write a raw frame with explicit flag and payload.
///
/// Used when the caller has already serialized/compressed the payload.
pub fn write_frame(
    path: impl AsRef<Path>,
    flag: u8,
    uncompressed_len: u64,
    payload: &[u8],
) -> Result<()> {
    let stored_len = payload.len() as u64;

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
    let mut file_bytes = std::fs::read(path.as_ref())?;
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
}
