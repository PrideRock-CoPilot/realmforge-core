// crates/rfsource-format/src/checksum.rs
//! BLAKE3 checksumming for RFSource segments
//!
//! Provides streaming checksum calculation during writes and verification during reads.
//! BLAKE3 is fast enough (>1 GB/s single-threaded) to calculate checksums inline without
//! impacting write performance.

use blake3::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::Path;
use std::fs::File;

/// BLAKE3 hash wrapper for RFSource segments
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentChecksum {
    /// BLAKE3 hash as 32-byte array
    hash: [u8; 32],
}

impl SegmentChecksum {
    /// Create from existing hash bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { hash: bytes }
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        if hex.len() != 64 {
            return Err(format!("Invalid hex length: expected 64, got {}", hex.len()));
        }
        
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&hex[i*2..i*2+2], 16)
                .map_err(|e| format!("Invalid hex character: {}", e))?;
        }
        Ok(Self { hash: bytes })
    }

    /// Get hash as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.hash)
    }

    /// Get hash bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.hash
    }

    /// Verify that this checksum matches the given bytes
    pub fn verify(&self, data: &[u8]) -> bool {
        let computed = blake3::hash(data);
        computed.as_bytes() == &self.hash
    }
}

impl From<Hash> for SegmentChecksum {
    fn from(hash: Hash) -> Self {
        Self { hash: *hash.as_bytes() }
    }
}

impl std::fmt::Display for SegmentChecksum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Streaming checksum writer that calculates BLAKE3 hash while writing
pub struct ChecksumWriter<W: Write> {
    writer: W,
    hasher: Hasher,
    bytes_written: u64,
}

impl<W: Write> ChecksumWriter<W> {
    /// Create a new checksum writer wrapping the given writer
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            hasher: Hasher::new(),
            bytes_written: 0,
        }
    }

    /// Finish writing and return the checksum and total bytes written
    pub fn finalize(self) -> (W, SegmentChecksum, u64) {
        let checksum = SegmentChecksum::from(self.hasher.finalize());
        (self.writer, checksum, self.bytes_written)
    }

    /// Get reference to inner writer
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Get mutable reference to inner writer
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Get bytes written so far
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

impl<W: Write> Write for ChecksumWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        self.bytes_written += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Streaming checksum reader that verifies BLAKE3 hash while reading
pub struct ChecksumReader<R: Read> {
    reader: R,
    hasher: Hasher,
    expected_checksum: Option<SegmentChecksum>,
    bytes_read: u64,
}

impl<R: Read> ChecksumReader<R> {
    /// Create a new checksum reader wrapping the given reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            hasher: Hasher::new(),
            expected_checksum: None,
            bytes_read: 0,
        }
    }

    /// Create a new checksum reader with expected checksum for verification
    pub fn new_with_expected(reader: R, expected: SegmentChecksum) -> Self {
        Self {
            reader,
            hasher: Hasher::new(),
            expected_checksum: Some(expected),
            bytes_read: 0,
        }
    }

    /// Verify the checksum after reading all data
    pub fn verify(self) -> Result<(R, u64), ChecksumError> {
        let computed = SegmentChecksum::from(self.hasher.finalize());
        
        if let Some(expected) = self.expected_checksum {
            if computed != expected {
                return Err(ChecksumError::Mismatch {
                    expected: expected.to_hex(),
                    computed: computed.to_hex(),
                    bytes_read: self.bytes_read,
                });
            }
        }
        
        Ok((self.reader, self.bytes_read))
    }

    /// Get the computed checksum so far
    pub fn computed_checksum(&self) -> SegmentChecksum {
        SegmentChecksum::from(self.hasher.finalize())
    }

    /// Get bytes read so far
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
}

impl<R: Read> Read for ChecksumReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.hasher.update(&buf[..n]);
        self.bytes_read += n as u64;
        Ok(n)
    }
}

/// Errors that can occur during checksumming
#[derive(Debug)]
pub enum ChecksumError {
    /// Checksum mismatch between expected and computed
    Mismatch {
        expected: String,
        computed: String,
        bytes_read: u64,
    },
    /// I/O error during checksum calculation
    Io(io::Error),
}

impl std::fmt::Display for ChecksumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChecksumError::Mismatch { expected, computed, bytes_read } => {
                write!(f, "Checksum mismatch after {} bytes: expected {}, got {}", 
                       bytes_read, expected, computed)
            }
            ChecksumError::Io(e) => write!(f, "I/O error during checksum: {}", e),
        }
    }
}

impl std::error::Error for ChecksumError {}

impl From<io::Error> for ChecksumError {
    fn from(e: io::Error) -> Self {
        ChecksumError::Io(e)
    }
}

/// Calculate BLAKE3 checksum of a file
pub fn checksum_file<P: AsRef<Path>>(path: P) -> io::Result<SegmentChecksum> {
    let file = File::open(path)?;
    let mut reader = ChecksumReader::new(file);
    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
    
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
    }
    
    Ok(reader.computed_checksum())
}

/// Verify a file against an expected checksum
pub fn verify_file<P: AsRef<Path>>(path: P, expected: &SegmentChecksum) -> Result<(), ChecksumError> {
    let file = File::open(path).map_err(ChecksumError::Io)?;
    let mut reader = ChecksumReader::new_with_expected(file, expected.clone());
    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
    
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
    }
    
    reader.verify()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_checksum_from_hex() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let checksum = SegmentChecksum::from_hex(hex).unwrap();
        assert_eq!(checksum.to_hex(), hex);
    }

    #[test]
    fn test_checksum_from_hex_invalid() {
        // Too short
        assert!(SegmentChecksum::from_hex("0123").is_err());
        
        // Invalid character
        assert!(SegmentChecksum::from_hex("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").is_err());
    }

    #[test]
    fn test_checksum_writer() {
        let data = b"Hello, RFSource!";
        let cursor = Cursor::new(Vec::new());
        let mut writer = ChecksumWriter::new(cursor);
        
        writer.write_all(data).unwrap();
        let (cursor, checksum, bytes_written) = writer.finalize();
        
        assert_eq!(bytes_written, data.len() as u64);
        
        // Verify checksum matches direct hash
        let expected = blake3::hash(data);
        assert_eq!(checksum.as_bytes(), expected.as_bytes());
        
        // Verify written data
        assert_eq!(cursor.into_inner(), data);
    }

    #[test]
    fn test_checksum_reader_no_verification() {
        let data = b"Hello, RFSource!";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = ChecksumReader::new(cursor);
        
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        
        let (_, bytes_read) = reader.verify().unwrap();
        assert_eq!(bytes_read, data.len() as u64);
        assert_eq!(buffer, data);
    }

    #[test]
    fn test_checksum_reader_with_verification_success() {
        let data = b"Hello, RFSource!";
        let expected_hash = blake3::hash(data);
        let expected = SegmentChecksum::from(expected_hash);
        
        let cursor = Cursor::new(data.to_vec());
        let mut reader = ChecksumReader::new_with_expected(cursor, expected);
        
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        
        // Should succeed
        reader.verify().unwrap();
    }

    #[test]
    fn test_checksum_reader_with_verification_failure() {
        let data = b"Hello, RFSource!";
        let wrong_data = b"Wrong data";
        let wrong_hash = blake3::hash(wrong_data);
        let wrong_checksum = SegmentChecksum::from(wrong_hash);
        
        let cursor = Cursor::new(data.to_vec());
        let mut reader = ChecksumReader::new_with_expected(cursor, wrong_checksum);
        
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        
        // Should fail
        match reader.verify() {
            Err(ChecksumError::Mismatch { .. }) => {},
            _ => panic!("Expected checksum mismatch"),
        }
    }

    #[test]
    fn test_checksum_verify_bytes() {
        let data = b"Hello, RFSource!";
        let hash = blake3::hash(data);
        let checksum = SegmentChecksum::from(hash);
        
        assert!(checksum.verify(data));
        assert!(!checksum.verify(b"Wrong data"));
    }

    #[test]
    fn test_streaming_matches_bulk() {
        let data = b"The quick brown fox jumps over the lazy dog";
        
        // Compute via streaming writer
        let cursor = Cursor::new(Vec::new());
        let mut writer = ChecksumWriter::new(cursor);
        writer.write_all(data).unwrap();
        let (_, checksum_streaming, _) = writer.finalize();
        
        // Compute via bulk hash
        let checksum_bulk = SegmentChecksum::from(blake3::hash(data));
        
        assert_eq!(checksum_streaming, checksum_bulk);
    }

    #[test]
    fn test_checksum_display() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let checksum = SegmentChecksum::from_hex(hex).unwrap();
        assert_eq!(format!("{}", checksum), hex);
    }
}
