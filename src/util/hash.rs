//! Hashing utilities using blake3

use crate::dag::commit::Hash;

/// Compute blake3 hash of content
pub fn compute_hash(content: &[u8]) -> Hash {
    let hash = blake3::hash(content);
    hash.to_hex().to_string()
}

/// Verify content matches hash
pub fn verify_hash(content: &[u8], expected: &Hash) -> bool {
    compute_hash(content) == *expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let content = b"Hello, world!";
        let hash = compute_hash(content);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // blake3 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_verify_hash() {
        let content = b"test content";
        let hash = compute_hash(content);
        assert!(verify_hash(content, &hash));
        assert!(!verify_hash(b"different content", &hash));
    }

    #[test]
    fn test_deterministic() {
        let content = b"deterministic test";
        let hash1 = compute_hash(content);
        let hash2 = compute_hash(content);
        assert_eq!(hash1, hash2);
    }
}
