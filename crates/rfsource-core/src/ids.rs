//! Typed identifiers for the rfsource domain model.
//!
//! All IDs follow the pattern: `{prefix}_{sha256_hex(input)[..16]}`
//! where `prefix` is a domain-specific short code.

use sha2::{Digest, Sha256};

/// Compute the SHA-256 hex digest of `input`.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a short content-addressed identifier with the given prefix.
///
/// # Examples
///
/// ```
/// let id = short_id("art", "src/main.rs at HEAD");
/// assert!(id.starts_with("art_"));
/// assert_eq!(id.len(), 20); // "art_" + 16 hex chars
/// ```
pub fn short_id(prefix: &str, input: &str) -> String {
    format!("{}_{}", prefix, &sha256_hex(input)[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_id_format() {
        let id = short_id("art", "src/main.rs");
        assert!(id.starts_with("art_"));
        assert_eq!(id.len(), 20);
    }

    #[test]
    fn test_deterministic_hash() {
        let a = short_id("cmt", "hello");
        let b = short_id("cmt", "hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_different_inputs() {
        let a = short_id("sym", "foo");
        let b = short_id("sym", "bar");
        assert_ne!(a, b);
    }
}
