//! Patch serialization and deserialization.

use crate::types::*;
use std::io::{Read, Write};

const PATCH_HEADER_SIZE: usize = 20;

/// Write a patch to a writer.
pub fn write_patch<W: Write>(mut writer: W, patch: &Patch) -> Result<()> {
    // Write header
    let manifest_json = patch.manifest.to_json()?;
    let manifest_bytes = manifest_json.as_bytes();
    let header = PatchHeader::new(manifest_bytes.len() as u64);
    writer.write_all(&header.to_bytes())?;

    // Write manifest
    writer.write_all(manifest_bytes)?;

    // Write data section
    writer.write_all(&patch.data)?;

    Ok(())
}

/// Read a patch from a reader.
pub fn read_patch<R: Read>(mut reader: R) -> Result<Patch> {
    // Read header
    let mut header_buf = [0u8; PATCH_HEADER_SIZE];
    reader.read_exact(&mut header_buf)?;

    let (header, _) = PatchHeader::from_bytes(&header_buf)?;

    // Read manifest
    let mut manifest_buf = vec![0u8; header.manifest_len as usize];
    reader.read_exact(&mut manifest_buf)?;
    let manifest_json = String::from_utf8(manifest_buf)
        .map_err(|e| PatchError::Format(format!("Invalid UTF-8 in manifest: {}", e)))?;
    let manifest = Manifest::from_json(&manifest_json)?;

    // Read data section
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

    Ok(Patch { manifest, data })
}

/// Append ADD blob to patch data section and return its offset.
pub fn append_add_blob(
    patch: &mut Patch,
    payload: &[u8],
    compressed: bool,
    zstd_level: i32,
) -> Result<u64> {
    let offset = patch.data.len() as u64;

    let header = BlobHeader {
        compressed,
        zstd_level: if compressed { zstd_level as u32 } else { 0 },
        payload_len: payload.len() as u64,
    };

    patch.data.extend_from_slice(&header.to_bytes());
    patch.data.extend_from_slice(payload);

    Ok(offset)
}

/// Read ADD blob from patch data section.
pub fn read_add_blob(patch: &Patch, offset: u64) -> Result<Vec<u8>> {
    let offset = offset as usize;
    if offset >= patch.data.len() {
        return Err(PatchError::Format("ADD blob offset out of range".to_string()));
    }

    let (header, header_size) = BlobHeader::from_bytes(&patch.data[offset..])?;
    let payload_offset = offset + header_size;
    let payload_end = payload_offset + header.payload_len as usize;

    if payload_end > patch.data.len() {
        return Err(PatchError::Format("ADD blob payload out of range".to_string()));
    }

    let payload = &patch.data[payload_offset..payload_end];

    if header.compressed {
        decompress_blob(payload)
    } else {
        Ok(payload.to_vec())
    }
}

/// Compress a blob with zstd.
pub fn compress_blob(data: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::encode_all(data, level as i32).map_err(|e| {
        PatchError::Compression(format!("Zstd compression failed: {}", e))
    })
}

/// Decompress a blob with zstd.
pub fn decompress_blob(data: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(data).map_err(|e| {
        PatchError::Compression(format!("Zstd decompression failed: {}", e))
    })
}
