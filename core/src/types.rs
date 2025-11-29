//! Shared types for PatchForge core library.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Error type for core operations.
#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Zstd error: {0}")]
    Compression(String),

    #[error("Format error: {0}")]
    Format(String),

    #[error("Verification failed: {0}")]
    Verification(String),

    #[error("Unsupported: {0}")]
    Unsupported(String),
}

/// Result alias for core operations.
pub type Result<T> = std::result::Result<T, PatchError>;

/// Options for creating a patch.
#[derive(Debug, Clone)]
pub struct MakePatchOptions {
    pub block_size: usize,      // Fixed block size (4096)
    pub zstd_level: i32,        // -1 for no compression, 0-22 for levels
    pub verify_checksums: bool, // Validate blocks during creation
}

impl Default for MakePatchOptions {
    fn default() -> Self {
        Self {
            block_size: 4096,
            zstd_level: 3,
            verify_checksums: true,
        }
    }
}

/// Options for applying a patch.
#[derive(Debug, Clone)]
pub struct ApplyPatchOptions {
    pub verify_checksums: bool,
    pub atomic: bool, // Use temp files and atomic renames
}

impl Default for ApplyPatchOptions {
    fn default() -> Self {
        Self {
            verify_checksums: true,
            atomic: true,
        }
    }
}

/// A single operation in a file's reconstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    /// Copy a block from source file.
    #[serde(rename = "COPY")]
    Copy {
        src: String,           // Relative path in src
        block_index: u64,      // Block number (offset = block_index * 4096)
        len: usize,            // Bytes to copy
    },
    /// Add new data from patch file.
    #[serde(rename = "ADD")]
    Add {
        data_offset: u64,   // Offset in patch Data section
        data_length: u64,   // Length of payload
        compressed: bool,   // Is payload compressed?
        compression: Option<String>, // Compression type ("zstd")
        zstd_level: Option<i32>,     // Compression level if compressed
    },
}

/// File or directory entry in the patch manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub path: String,              // Relative path
    pub entry_type: String,        // "file" or "dir"
    #[serde(default)]
    pub mode: u32,                 // Unix permissions
    #[serde(default)]
    pub mtime: u64,                // Modification time (optional)
    #[serde(default)]
    pub sha256: Option<String>,    // Optional checksum
    #[serde(default)]
    pub ops: Vec<PatchOp>,         // Operations to create this file
}

/// The patch manifest (JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
        }
    }

    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string(self).map_err(PatchError::Json)
    }

    pub fn from_json(json: &str) -> crate::Result<Self> {
        serde_json::from_str(json).map_err(PatchError::Json)
    }
}

/// In-memory patch representation.
#[derive(Debug)]
pub struct Patch {
    pub manifest: Manifest,
    pub data: Vec<u8>, // Raw data section bytes
}

impl Patch {
    pub fn new() -> Self {
        Self {
            manifest: Manifest::new(),
            data: Vec::new(),
        }
    }
}

/// Block hash info for deduplication.
#[derive(Debug, Clone)]
pub struct BlockHash {
    pub sha256: String,       // SHA-256 hex
    pub file_path: PathBuf,   // Which file it came from
    pub block_index: u64,     // Which block in that file
    pub len: usize,           // Actual length (may be < block_size for last block)
}

/// Patch header (binary).
#[derive(Debug)]
pub struct PatchHeader {
    pub magic: [u8; 8],      // "PATCHFG1\0" (8 bytes)
    pub version: u32,        // Version number (big-endian)
    pub manifest_len: u64,   // Length of manifest (big-endian)
}

impl PatchHeader {
    pub fn new(manifest_len: u64) -> Self {
        let mut magic = [0u8; 8];
        magic[..7].copy_from_slice(b"PATCHFG1");
        Self {
            magic,
            version: 1,
            manifest_len,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.magic);
        buf.extend_from_slice(&self.version.to_be_bytes());
        buf.extend_from_slice(&self.manifest_len.to_be_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> crate::Result<(Self, usize)> {
        if buf.len() < 20 {
            return Err(PatchError::Format("Header too short".to_string()));
        }
        let magic = [
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ];
        if &magic[..7] != b"PATCHFG1" {
            return Err(PatchError::Format("Invalid magic".to_string()));
        }
        let version = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let manifest_len = u64::from_be_bytes([
            buf[12], buf[13], buf[14], buf[15], buf[16], buf[17], buf[18], buf[19],
        ]);
        Ok((
            Self {
                magic,
                version,
                manifest_len,
            },
            20,
        ))
    }
}

/// Blob header for ADD payloads in Data section.
#[derive(Debug)]
pub struct BlobHeader {
    pub compressed: bool,   // 0 = none, 1 = zstd
    pub zstd_level: u32,    // Compression level (big-endian u32)
    pub payload_len: u64,   // Payload length (big-endian u64)
}

impl BlobHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(if self.compressed { 1 } else { 0 });
        buf.extend_from_slice(&self.zstd_level.to_be_bytes());
        buf.extend_from_slice(&self.payload_len.to_be_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> crate::Result<(Self, usize)> {
        if buf.len() < 13 {
            return Err(PatchError::Format("Blob header too short".to_string()));
        }
        let compressed = buf[0] != 0;
        let zstd_level = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        let payload_len = u64::from_be_bytes([
            buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11], buf[12],
        ]);
        Ok((
            Self {
                compressed,
                zstd_level,
                payload_len,
            },
            13,
        ))
    }
}

/// File block data used during diff/patch.
#[derive(Debug, Clone)]
pub struct FileBlock {
    pub data: Vec<u8>,
    pub sha256: String,
    pub offset: u64, // Byte offset in file
}
