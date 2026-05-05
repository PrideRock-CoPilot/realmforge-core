use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Errors that can occur during signing and verification.
#[derive(Debug, thiserror::Error)]
pub enum SigningError {
    #[error("invalid key: {0}")]
    InvalidKey(String),
    #[error("signature verification failed")]
    VerificationFailed,
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A development key pair for bundle signing.
/// In production, keys would be managed by an HSM or key management service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyPair {
    pub private_key_hex: String,
    pub public_key_hex: String,
}

/// Generate a development key pair using SHA-256 as a shared-secret signing mechanism.
///
/// Both `private_key_hex` and `public_key_hex` contain the same shared-secret key.
/// This is HMAC-SHA256 semantics — both signing and verification use the same key.
/// In production, replace with ed25519-dalek or similar asymmetric crypto with proper
/// key separation between signing and verification.
pub fn generate_key_pair() -> KeyPair {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_be_bytes();

    // Use the same key for both "private" and "public" fields — they are the
    // same shared secret in development mode (HMAC-style symmetric signing).
    let key = Sha256::digest(seed);
    let key_hex = hex::encode(key);

    KeyPair {
        private_key_hex: key_hex.clone(),
        public_key_hex: key_hex,
    }
}

/// Sign a payload string with the given private key.
/// Returns a hex-encoded signature.
pub fn sign_payload(payload: &str, private_key_hex: &str) -> Result<String, SigningError> {
    let private_key = hex::decode(private_key_hex)
        .map_err(|e| SigningError::InvalidKey(format!("invalid hex: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(&private_key);
    hasher.update(payload.as_bytes());
    let signature = hasher.finalize();
    Ok(hex::encode(signature))
}

/// Verify a signature against a payload using the given public key.
pub fn verify_signature(
    payload: &str,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, SigningError> {
    let public_key = hex::decode(public_key_hex)
        .map_err(|e| SigningError::InvalidKey(format!("invalid hex: {}", e)))?;
    let signature_bytes = hex::decode(signature_hex)
        .map_err(|e| SigningError::InvalidKey(format!("invalid hex: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(&public_key);
    hasher.update(payload.as_bytes());
    let expected = hasher.finalize().to_vec();

    Ok(expected == signature_bytes)
}

/// Helper: compute the SHA-256 hex digest of arbitrary bytes.
pub fn compute_hash(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = generate_key_pair();
        let payload = "test-payload-123";

        let sig = sign_payload(payload, &kp.private_key_hex).unwrap();
        assert!(verify_signature(payload, &sig, &kp.public_key_hex).unwrap());
    }

    #[test]
    fn wrong_key_fails_verification() {
        let kp1 = generate_key_pair();
        let kp2 = generate_key_pair();
        let payload = "test-payload";

        let sig = sign_payload(payload, &kp1.private_key_hex).unwrap();
        assert!(!verify_signature(payload, &sig, &kp2.public_key_hex).unwrap());
    }

    #[test]
    fn tampered_payload_fails_verification() {
        let kp = generate_key_pair();
        let payload = "original-payload";

        let sig = sign_payload(payload, &kp.private_key_hex).unwrap();
        assert!(!verify_signature("tampered-payload", &sig, &kp.public_key_hex).unwrap());
    }

    #[test]
    fn compute_hash_is_consistent() {
        let data = b"hello world";
        let h1 = compute_hash(data);
        let h2 = compute_hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn invalid_hex_returns_error() {
        let result = sign_payload("test", "not-hex");
        assert!(result.is_err());
    }
}
