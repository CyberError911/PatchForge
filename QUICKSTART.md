# PatchForge — Quick Start Guide

## Installation

### Windows/Linux/macOS
```bash
# Build the project
cargo build --release

# Binary location:
# Windows: target\release\patchforge.exe
# Linux/macOS: target/release/patchforge
```

## Usage Examples

### Example 1: Create a simple patch

```bash
# Create old and new folders
mkdir old new

# Add files to old
echo "version 1" > old/config.txt
echo "data" > old/file.bin

# Modify for new
echo "version 2" > new/config.txt
echo "data updated" > new/file.bin
mkdir new/subfolder

# Create patch
patchforge make old new my_patch.patch

# Result: my_patch.patch (small binary file with all changes)
```

### Example 2: Apply a patch

```bash
# Assuming you have a copy of the old folder
cp -r old target_folder

# Apply patch to bring it to new state
patchforge apply target_folder my_patch.patch

# Now target_folder matches new/
```

### Example 3: Use the GUI

```bash
python gui/main.py
```

A window appears:
1. Click "Create Patch" tab
2. Select old folder
3. Select new folder
4. Choose output file
5. Click "Create Patch"

Or use "Apply Patch" tab to apply patches.

### Example 4: Custom compression

```bash
# No compression (fastest, largest patch)
patchforge make old new fast.patch --zstd-level -1

# Maximum compression (slower, smallest patch)
patchforge make old new tiny.patch --zstd-level 22

# Balanced (default)
patchforge make old new balanced.patch --zstd-level 3
```

## How It Works (Simple Explanation)

1. **Scan both folders** — read all files block-by-block (4096 bytes each)
2. **Find matching blocks** — compare blocks using SHA-256 hashes
3. **Create patch** — 
   - COPY blocks: reference to source file + block number (tiny size)
   - ADD blocks: new/changed data (compressed with zstd)
4. **Apply patch** — replay operations to recreate new folder state

## Patch File Size

Typical compression ratios:
- **File updates**: 1-10% of original file size (mostly COPYs)
- **New files**: depends on content, zstd level (usually 30-60%)
- **Large repos**: usually 50-200 MB patches instead of GB downloads

## Tips & Tricks

### Generate a patch between two game versions
```bash
patchforge make game_v1.0 game_v1.1 update.patch
patchforge apply my_game update.patch  # Now you have v1.1
```

### Backup differential
```bash
patchforge make /old/backup /new/backup incremental.patch
```

### Test if patch is correct
```bash
cp -r old test_dir
patchforge apply test_dir my_patch.patch
# Verify test_dir matches new/ (should be identical)
```

### List patch contents
Since patches are binary, use the core library to inspect:
```rust
let mut file = std::fs::File::open("my_patch.patch")?;
let patch = core::read_patch(&mut file)?;
println!("{}", serde_json::to_string_pretty(&patch.manifest)?);
```

## Troubleshooting

### "Patch creation failed"
- Ensure both folders exist
- Check disk space
- Verify read permissions

### "Patch application failed"
- Ensure source folder structure matches patch assumptions
- Try with `--no-verify` flag
- Check disk space on target

### Patch file is too large
- Increase zstd compression level: `--zstd-level 22`
- Ensure you're not including build artifacts (use `.gitignore` patterns)

## Integration with Your Code

### Rust library usage
```rust
use core::{make_patch, apply_patch, MakePatchOptions, ApplyPatchOptions};
use std::path::Path;

let opts = MakePatchOptions::default();
make_patch(Path::new("old"), Path::new("new"), Path::new("out.patch"), &opts)?;
```

### Call CLI from your app
```bash
your_app -> patchforge make old new output.patch -> output.patch
```

### Use patch file directly
Patch files are self-contained binary blobs. Distribute them over any channel (HTTP, P2P, USB, etc.)

## Performance

- **Small patches (<10 MB)**: < 1 second
- **Medium patches (10-100 MB)**: 1-5 seconds
- **Large patches (>100 MB)**: 5-60 seconds (depends on zstd level)

On modern hardware:
- **Creation**: ~10-50 MB/sec throughput
- **Application**: ~50-100 MB/sec throughput

## Next Steps

1. Try the examples above
2. Read `core/README_DIFF.md` for technical details
3. Check `examples/README.md` for more use cases
4. Integrate into your project using the Rust library
5. Contribute improvements on GitHub!

---

**Questions?** See the full documentation:
- `README.md` — project overview
- `core/README_DIFF.md` — technical specification
- `examples/README.md` — API examples
- `PROJECT_SUMMARY.md` — implementation details
