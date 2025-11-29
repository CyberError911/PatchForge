//! Diff engine: folder walking, block hashing, operation generation.

use crate::types::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Hash all blocks in a file.
pub fn hash_file_blocks(file_path: &Path, block_size: usize) -> Result<Vec<BlockHash>> {
    let mut file = File::open(file_path)?;
    let mut blocks = Vec::new();
    let mut buffer = vec![0u8; block_size];
    let mut block_index = 0u64;

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        let chunk = &buffer[..n];
        let mut hasher = Sha256::new();
        hasher.update(chunk);
        let sha256_hex = format!("{:x}", hasher.finalize());

        blocks.push(BlockHash {
            sha256: sha256_hex,
            file_path: file_path.to_path_buf(),
            block_index,
            len: n,
        });

        block_index += 1;
    }

    Ok(blocks)
}

/// Walk a directory and collect all file blocks with their paths.
pub fn scan_tree(root: &Path, block_size: usize) -> Result<HashMap<String, Vec<BlockHash>>> {
    let mut result = HashMap::new();

    let mut entries: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    entries.sort_by(|a, b| a.path().cmp(b.path()));

    for entry in entries {
        let full_path = entry.path();
        let rel_path = full_path
            .strip_prefix(root)
            .map_err(|e| PatchError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let rel_key = rel_path.to_string_lossy().to_string();
        let blocks = hash_file_blocks(full_path, block_size)?;
        result.insert(rel_key, blocks);
    }

    Ok(result)
}

/// Generate a manifest for transforming src_tree into dst_tree.
pub fn generate_manifest(
    src_root: &Path,
    dst_root: &Path,
    block_size: usize,
) -> Result<Manifest> {
    // Scan both trees
    let src_blocks = scan_tree(src_root, block_size)?;
    let dst_files = list_files_sorted(dst_root)?;

    // Build a map: sha256 -> (file_path, block_index, len)
    let mut src_block_map: HashMap<String, Vec<(String, u64, usize)>> = HashMap::new();
    for (file_path, blocks) in &src_blocks {
        for block in blocks {
            src_block_map
                .entry(block.sha256.clone())
                .or_insert_with(Vec::new)
                .push((file_path.clone(), block.block_index, block.len));
        }
    }

    let mut manifest = Manifest::new();

    // Process each file in destination
    for dst_file_rel in dst_files {
        let dst_full_path = dst_root.join(&dst_file_rel);
        if !dst_full_path.is_file() {
            continue;
        }

        let mut entry = ManifestEntry {
            path: dst_file_rel.clone(),
            entry_type: "file".to_string(),
            mode: 0o644,
            mtime: 0,
            sha256: None,
            ops: Vec::new(),
        };

        // Read and block-hash the destination file
        let dst_blocks = hash_file_blocks(&dst_full_path, block_size)?;

        for block_hash in dst_blocks {
            if let Some(src_matches) = src_block_map.get(&block_hash.sha256) {
                // Found a matching block in source
                if let Some((src_file, src_idx, src_len)) = src_matches.first() {
                    entry.ops.push(PatchOp::Copy {
                        src: src_file.clone(),
                        block_index: *src_idx,
                        len: *src_len,
                    });
                    continue;
                }
            }

            // No match: we'll add this block later (placeholder for now)
            entry.ops.push(PatchOp::Add {
                data_offset: 0,      // Will be updated during write
                data_length: block_hash.len as u64,
                compressed: false,
                compression: None,
                zstd_level: None,
            });
        }

        manifest.entries.push(entry);
    }

    // Also add directories
    for entry in list_dirs_sorted(dst_root)? {
        manifest.entries.push(ManifestEntry {
            path: entry,
            entry_type: "dir".to_string(),
            mode: 0o755,
            mtime: 0,
            sha256: None,
            ops: Vec::new(),
        });
    }

    Ok(manifest)
}

/// List all files in a directory sorted lexically.
fn list_files_sorted(root: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|e| PatchError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        files.push(rel.to_string_lossy().to_string());
    }

    files.sort();
    Ok(files)
}

/// List all directories in a directory sorted lexically.
fn list_dirs_sorted(root: &Path) -> Result<Vec<String>> {
    let mut dirs = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let path = entry.path();
        if path == root {
            continue; // Skip root itself
        }
        let rel = path
            .strip_prefix(root)
            .map_err(|e| PatchError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        dirs.push(rel.to_string_lossy().to_string());
    }

    dirs.sort();
    Ok(dirs)
}
