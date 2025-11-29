# PatchForge

**A modern, clean, cross-platform patch creator and applier for games, software, and any files.**

Create tiny patch files for ANY folder, ANY game, ANY project — no bloat, no magic, just pure delta logic.

## Why PatchForge?

- **Game modders** can release patches instead of reuploading entire files
- **Developers** can distribute updates with minimal bandwidth
- **Slow internet users** get fast downloads
- **Everyone** gets simple, reliable patching
- **Open-source** and completely free

## Features

✅ Folder diff + binary diff (fixed-block based)  
✅ Super small patch files (KB instead of GB)  
✅ Zstd compression support  
✅ Patch verification & checksums  
✅ Progress bars during creation/application  
✅ Error rollback on failed patches  
✅ Cross-platform: Windows, Linux, macOS, WSL  
✅ Clean, modular Rust codebase  
✅ CLI + simple GUI  

## Quick Start

### Create a patch

```bash
patchforge make old_folder new_folder output.patch
```

### Apply a patch

```bash
patchforge apply original_folder output.patch
```

### GUI

```bash
python gui/app.py
```

## Installation

```bash
cargo build --release
```

Binary will be at `target/release/cli` (or `cli.exe` on Windows).

## How it works

1. **Folder walk** — scans old and new folders in deterministic order
2. **Fixed-block hashing** — divides files into 4096-byte blocks, computes SHA-256
3. **Block matching** — finds identical blocks between old and new (COPY ops)
4. **Delta storage** — new/changed blocks stored as ADD ops with optional zstd compression
5. **Manifest** — JSON file lists all operations to transform old → new
6. **Apply** — replays operations to rebuild new folder from patch + old folder

## Repository structure

```
PatchForge/
 ├─ core/
 │   ├─ src/
 │   │   ├─ lib.rs           # Main library entry
 │   │   ├─ diff.rs          # Diff engine & folder walking
 │   │   ├─ patch.rs         # Patch format & serialization
 │   │   ├─ compress.rs      # Zstd compression
 │   │   ├─ verify.rs        # Checksums & validation
 │   │   └─ types.rs         # Shared types
 │   ├─ Cargo.toml
 │   └─ README_DIFF.md       # Technical design document
 ├─ cli/
 │   ├─ src/
 │   │   ├─ main.rs          # CLI entry + command parsing
 │   │   └─ progress.rs      # Progress bars
 │   └─ Cargo.toml
 ├─ gui/
 │   ├─ app.py               # Tkinter GUI
 │   ├─ __init__.py
 │   └─ README.md
 ├─ examples/
 │   └─ README.md
 ├─ tests/
 ├─ Cargo.toml              # Workspace root
 ├─ Makefile
 ├─ LICENSE                 # MIT
 ├─ .gitignore
 └─ README.md               # This file
```

## Tech Stack

- **Rust** — core engine (fast, tiny, safe)
- **Zstd** — compression
- **Tokio** — async IO (optional)
- **Python/Tkinter** — simple GUI
- **Serde** — JSON serialization

## Examples

See `examples/` folder for working examples of patch creation and application.

## License

MIT License — see LICENSE file.

## Contributing

Open an issue or pull request! All contributions welcome.
