//! Verification and checksums.

use sha2::{Digest, Sha256};

/// Compute SHA-256 checksum of data.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Verify a block's checksum.
pub fn verify_block(data: &[u8], expected_sha256: &str) -> bool {
    sha256_hex(data) == expected_sha256
}
