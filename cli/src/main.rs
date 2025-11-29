use clap::{Parser, Subcommand};
use core::{ApplyPatchOptions, MakePatchOptions};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "patchforge")]
#[command(about = "Create and apply binary patches for any folder", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a patch file
    Make {
        /// Old/source folder
        #[arg(value_name = "SRC")]
        src: PathBuf,

        /// New/destination folder
        #[arg(value_name = "DST")]
        dst: PathBuf,

        /// Output patch file
        #[arg(value_name = "PATCH")]
        patch: PathBuf,

        /// Zstd compression level (0-22, -1 for no compression)
        #[arg(short, long, default_value = "3")]
        zstd_level: i32,
    },

    /// Apply a patch file
    Apply {
        /// Target folder to patch
        #[arg(value_name = "TARGET")]
        target: PathBuf,

        /// Patch file
        #[arg(value_name = "PATCH")]
        patch: PathBuf,

        /// Skip checksum verification
        #[arg(short, long)]
        no_verify: bool,
    },
}

fn main() -> core::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Make {
            src,
            dst,
            patch,
            zstd_level,
        } => {
            println!("Creating patch: {} -> {}", src.display(), dst.display());
            println!("Output: {}", patch.display());

            let opts = MakePatchOptions {
                block_size: 4096,
                zstd_level,
                verify_checksums: true,
            };

            core::make_patch(&src, &dst, &patch, &opts)?;
            println!("✓ Patch created successfully!");
        }

        Commands::Apply {
            target,
            patch,
            no_verify,
        } => {
            println!("Applying patch: {}", patch.display());
            println!("Target: {}", target.display());

            let opts = ApplyPatchOptions {
                verify_checksums: !no_verify,
                atomic: true,
            };

            core::apply_patch(&target, &patch, &opts)?;
            println!("✓ Patch applied successfully!");
        }
    }

    Ok(())
}

