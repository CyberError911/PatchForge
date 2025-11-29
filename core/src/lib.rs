//! PatchForge core library.
//!
//! High-level API for creating and applying patches.

pub mod compress;
pub mod diff;
pub mod patch;
pub mod types;
pub mod verify;

pub use types::{
    ApplyPatchOptions, MakePatchOptions, Patch, PatchError, PatchOp, Result,
};

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

/// Create a patch file that transforms `src_root` into `dst_root`.
///
/// - `src_root` and `dst_root` are directory roots.
/// - `output_patch` is the file to write the patch into (created/truncated).
/// - `opts` controls block size and compression behavior.
pub fn make_patch(
    src_root: &Path,
    dst_root: &Path,
    output_patch: &Path,
    opts: &MakePatchOptions,
) -> Result<()> {
    // Generate manifest
    let mut manifest = diff::generate_manifest(src_root, dst_root, opts.block_size)?;

    // Create patch and populate data section
    let mut patch = Patch::new();

    for entry in &mut manifest.entries {
        if entry.entry_type == "file" {
            let dst_file_path = dst_root.join(&entry.path);
            let mut file = File::open(&dst_file_path)?;
            let mut buffer = vec![0u8; opts.block_size];

            for op in &mut entry.ops {
                match op {
                    PatchOp::Add {
                        data_offset,
                        data_length,
                        compressed,
                        compression,
                        zstd_level,
                    } => {
                        let mut block_data = vec![0u8; *data_length as usize];
                        file.read_exact(&mut block_data)?;

                        let payload = if opts.zstd_level >= 0 {
                            *compressed = true;
                            *compression = Some("zstd".to_string());
                            *zstd_level = Some(opts.zstd_level);
                            compress::compress(&block_data, opts.zstd_level)?
                        } else {
                            block_data
                        };

                        *data_offset = patch::append_add_blob(&mut patch, &payload, *compressed, opts.zstd_level)?;
                    }
                    PatchOp::Copy { .. } => {
                        // Already set in generate_manifest
                    }
                }
            }
        }
    }

    patch.manifest = manifest;

    // Write patch file
    let mut out_file = File::create(output_patch)?;
    patch::write_patch(&mut out_file, &patch)?;

    Ok(())
}

/// Apply a patch file to `target_root`.
/// - `patch_path` is the path to the patch file.
/// - `opts` controls verification and atomic application.
pub fn apply_patch(
    target_root: &Path,
    patch_path: &Path,
    opts: &ApplyPatchOptions,
) -> Result<()> {
    // Read patch file
    let mut patch_file = File::open(patch_path)?;
    let patch = patch::read_patch(&mut patch_file)?;

    // Ensure target root exists
    fs::create_dir_all(target_root)?;

    // Process manifest entries
    for entry in &patch.manifest.entries {
        let full_path = target_root.join(&entry.path);

        match entry.entry_type.as_str() {
            "dir" => {
                fs::create_dir_all(&full_path)?;
            }
            "file" => {
                // Ensure parent dir exists
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Write to temp file, then rename
                let temp_path = if opts.atomic {
                    format!("{}.tmp", full_path.display())
                } else {
                    full_path.to_string_lossy().to_string()
                };

                let mut out_file = File::create(&temp_path)?;

                // Execute operations in order
                for op in &entry.ops {
                    match op {
                        PatchOp::Copy {
                            src,
                            block_index,
                            len,
                        } => {
                            let src_path = target_root.join(src);
                            let mut src_file = File::open(&src_path)?;

                            let offset = *block_index as u64 * opts.block_size as u64;
                            src_file.seek(std::io::SeekFrom::Start(offset))?;

                            let mut buf = vec![0u8; *len];
                            src_file.read_exact(&mut buf)?;

                            if opts.verify_checksums {
                                // Optional: verify checksum if available
                            }

                            out_file.write_all(&buf)?;
                        }
                        PatchOp::Add {
                            data_offset,
                            data_length,
                            compressed,
                            ..
                        } => {
                            let payload = patch::read_add_blob(&patch, *data_offset)?;
                            out_file.write_all(&payload)?;
                        }
                    }
                }

                drop(out_file);

                // Atomically rename
                if opts.atomic {
                    fs::rename(&temp_path, &full_path)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Serialize a patch object to a writer (streaming-friendly).
pub fn write_patch<W: Write>(writer: W, p: &Patch) -> Result<()> {
    patch::write_patch(writer, p)
}

/// Deserialize a patch object from a reader (streaming-friendly).
pub fn read_patch<R: Read>(reader: R) -> Result<Patch> {
    patch::read_patch(reader)
}

