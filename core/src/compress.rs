//! Compression utilities.

use crate::types::Result;

/// Compress data with zstd at given level.
pub fn compress(data: &[u8], level: i32) -> Result<Vec<u8>> {
    if level < 0 {
        return Ok(data.to_vec());
    }
    zstd::encode_all(data, level).map_err(|e| {
        crate::types::PatchError::Compression(format!("Compression failed: {}", e))
    })
}

/// Decompress data with zstd.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(data).map_err(|e| {
        crate::types::PatchError::Compression(format!("Decompression failed: {}", e))
    })
}
