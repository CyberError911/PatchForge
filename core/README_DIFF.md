PatchForge — Simplified Design Document (core/README_DIFF.md)

1. Overview
-----------

This document describes a deliberately simple, minimal patch format for PatchForge. It uses fixed-size blocks (4096 bytes), no content-defined chunking, no block index section, no delete entries, no outer compression, and no cache system. The patch file contains a compact header, a JSON manifest describing files and simple operations, and a Data section containing ADD payloads (each ADD payload may be optionally compressed with zstd).

2. Design goals
---------------

- Minimal, easy-to-implement format.
- Deterministic and streamable.
- Fixed block size for simplicity: 4096 bytes.
- Only ADD and COPY operations; no DELETE entries.
- No global block index; ADD payloads are addressed by offset in the Data section.
- Optional per-chunk zstd compression for ADD payloads; no outer-frame compression.

3. Patch file layout (logical)
------------------------------

The patch file is a single binary file with these sections in order:

- Header:
  - Magic (8 bytes): ASCII `PATCHFG1` + one zero byte for padding (total 8 bytes).
  - Version (u32, big-endian).
  - Manifest length (u64, big-endian).

- Manifest payload (UTF-8 JSON): length as specified above.

- Data section: a sequence of ADD blobs concatenated; each ADD blob is prefixed by a small blob header (see Data section format below).

- (No block index section, no trailer checksum in this minimal design.)

All integers are stored big-endian (network order) for cross-platform determinism.

4. Manifest structure (JSON)
----------------------------

The manifest is a JSON object with the following minimal schema:

{
  "version": 1,
  "entries": [
    {
      "path": "relative/path/to/file.bin",
      "type": "file",
      "mode": 420,
      "size": 12345,
      "mtime": 169xxxxxxx,
      "sha256": "... optional checksum ...",
      "ops": [
        { "op": "COPY", "src": "relative/path/in/src", "block_index": 5, "len": 4096 },
        { "op": "ADD",  "data_offset": 12345, "data_length": 4096, "compressed": true, "compression": "zstd", "zstd_level": 3 }
      ]
    },
    { "path": "relative/path/to/dir", "type": "dir", "mode": 493 }
  ]
}

Field notes:
- `entries` is an ordered array. Directories should be created before files within them.
- For a `file` entry, `ops` is a sequence of operations to produce the destination file's bytes in order.
- `COPY` op: refers to a fixed-size block from the source tree. `block_index` is an integer index (0-based) referring to the block number in the source file; the source offset is computed as `block_index * 4096`. `len` is the number of bytes to copy (last block may be shorter than 4096).
- `ADD` op: refers to bytes stored in the patch Data section. The manifest gives the `data_offset` (u64) and `data_length` (u64) within the Data section (Data section offsets are measured from the start of the Data section). `compressed` is a boolean indicating whether the ADD payload is compressed; if `true`, `compression` is expected to be `"zstd"` and `zstd_level` indicates the compression level.

5. Data section format
----------------------

Each ADD blob placed into the Data section is encoded as:

- Blob header:
  - Compression flag (u8): 0 = none, 1 = zstd.
  - If compressed: zstd level (u32, big-endian). If not compressed, this field is zero.
  - Payload length (u64, big-endian): length of the following payload in bytes (compressed length if compressed, raw length if not compressed).

- Payload bytes: the payload itself (either raw bytes or zstd-compressed bytes).

The `data_offset` stored in the manifest points to the start of the Blob header for that ADD blob, measured from the start of the Data section.

6. Blocking rules and matching
------------------------------

- Fixed block size: 4096 bytes.
- When creating a patch, files are read in fixed 4096-byte blocks (the final block may be shorter).
- For each destination block, compute SHA-256 and compare against source file blocks' SHA-256 values. If a source block matches, emit a `COPY` op referencing the source path and `block_index`.
- If no matching source block is found, emit an `ADD` op and place the destination block into the Data section.

7. COPY and ADD semantics (apply-time)
--------------------------------------

- `COPY` op: during apply, open the source file indicated by `src` (relative to the original `src_root` used to create the patch). Read from `offset = block_index * 4096` for `len` bytes and write those bytes into the destination output (streaming). Implementations must check bounds and may verify a checksum if provided.
- `ADD` op: during apply, seek to `data_offset` in the Data section, read the Blob header, read the payload (decompress with zstd if indicated), and write the resulting bytes into the destination output.
- Files are written to temporary paths and atomically renamed into place when all ops succeed.
- No deletions: the applier does not remove files that exist in the target but are absent from the manifest.

8. Folder-walk and patch creation algorithm
------------------------------------------

- Walk `src_root` and `dst_root` deterministically (lexical sort of paths).
- For each path present in `dst_root`:
  - If not present in `src_root`: emit a file entry whose `ops` is a sequence of `ADD` ops for each 4096-byte block (placed in Data section).
  - If present in both and are files: read both files in 4096-byte blocks, compute per-block SHA-256; for each destination block, if a matching source block exists, emit `COPY` referencing the source path and block index; otherwise emit `ADD` and store block in Data.
- Directories are emitted as `type: "dir"` entries. No `DELETE` entries are emitted for paths missing from `dst_root`.

9. Checksums and verification
-----------------------------

- Per-block SHA-256 values may be stored in the manifest (optional) for additional verification; if included, apply should verify copied blocks against these checksums.
- The manifest may include an overall file SHA-256 for final validation of reconstructed files (optional).

10. Compression
---------------

- Only per-ADD-blob zstd compression is allowed (optional). Each ADD blob records whether it is compressed and the zstd level used.
- No outer-file compression is used in this minimal design.

11. API signatures (Rust)
-------------------------

Include the exact API signatures for `make_patch`, `apply_patch`, `write_patch`, and `read_patch` as originally requested. These signatures are intentionally explicit and use standard library types where practical.

```rust
use std::path::Path;

/// Error type for core operations.
pub enum PatchError { /* as documented in README_DIFF */ }
pub type Result<T> = std::result::Result<T, PatchError>;

/// Options to control patch creation.
pub struct MakePatchOptions {
    pub block_size_avg: usize, // recommended default 4096
    pub min_block: usize,      // recommended 2048
    pub max_block: usize,      // recommended 16384
    pub zstd_level: i32,       // zstd compression level, -1 for no compression
}

/// Options controlling patch application.
pub struct ApplyPatchOptions {
    pub verify_checksums: bool,
    pub keep_backups: bool,
}

/// Create a patch file that transforms `src_root` into `dst_root`.
///
/// - `src_root` and `dst_root` are directory roots.
/// - `output_patch` is the file to write the patch into (created/truncated).
/// - `opts` controls chunking and compression behavior.
pub fn make_patch(
    src_root: &Path,
    dst_root: &Path,
    output_patch: &Path,
    opts: &MakePatchOptions,
) -> Result<()>;

/// Apply a patch file to `target_root`.
/// - `patch_path` may be a file path or stream path.
/// - `opts` controls verification and fallback semantics.
pub fn apply_patch(
    target_root: &Path,
    patch_path: &Path,
    opts: &ApplyPatchOptions,
) -> Result<()>;

/// Serialize a patch object to a writer (streaming-friendly). The writer can be any type implementing `std::io::Write`.
pub fn write_patch<W: std::io::Write>(writer: W, patch: &Patch) -> Result<()>;

/// Deserialize a patch object from a reader (streaming-friendly). The reader must offer `std::io::Read`.
pub fn read_patch<R: std::io::Read>(reader: R) -> Result<Patch>;
```

Notes:
- The above signatures are included for parity with the higher-level design; implementations may ignore CDC-related options (e.g., `min_block`/`max_block`) but keep the signatures unchanged to remain compatible with future changes.

12. Limitations and omissions (intentional)
------------------------------------------

- No content-defined chunking: fixed 4096-byte blocks only.
- No block index: ADD blobs are located by offset in the Data section.
- No DELETE entries: patches do not remove files on apply.
- No outer-file compression or global checksums in this minimal design.

13. Next steps for implementation
---------------------------------

- Implement fixed-block reader/writer, block hashing (SHA-256), manifest serializer, ADD blob writer (with optional zstd), and simple `make_patch`/`apply_patch` orchestrators following these rules.

