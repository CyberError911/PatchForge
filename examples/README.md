# PatchForge Examples

This folder contains example usage of PatchForge.

## Quick Start

### Example 1: Create a patch

```bash
patchforge make old_folder new_folder output.patch
```

### Example 2: Apply a patch

```bash
patchforge apply target_folder output.patch
```

### Example 3: With custom compression

```bash
patchforge make old_folder new_folder output.patch --zstd-level 22
```

## Using the GUI

```bash
python gui/main.py
```

A simple Tkinter interface will appear with:
- **Create Patch** tab — select old/new folders, generate patch file
- **Apply Patch** tab — select target folder and patch file to apply

## API Usage (Rust)

In your Rust project, add to `Cargo.toml`:

```toml
[dependencies]
core = { path = "path/to/patchforge/core" }
```

Then use the API:

```rust
use core::{make_patch, apply_patch, MakePatchOptions, ApplyPatchOptions};
use std::path::Path;

fn main() -> core::Result<()> {
    let opts = MakePatchOptions::default();
    make_patch(
        Path::new("old"),
        Path::new("new"),
        Path::new("output.patch"),
        &opts,
    )?;
    
    let apply_opts = ApplyPatchOptions::default();
    apply_patch(
        Path::new("target"),
        Path::new("output.patch"),
        &apply_opts,
    )?;
    
    Ok(())
}
```

## Patch Format

Patches are binary files with:
- 8-byte magic header (`PATCHFG1`)
- 4-byte version (u32, big-endian)
- 8-byte manifest length (u64, big-endian)
- JSON manifest (describes operations)
- Data section (ADD blobs with optional zstd compression)

See `core/README_DIFF.md` for the full technical specification.
