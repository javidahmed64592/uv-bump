//! Command-line interface for uv-align

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "uv-align",
    about = "Update dependency constraints using versions resolved by `uv`"
)]
pub struct Cli {
    /// Path to folder containing `pyproject.toml` and `uv.lock` files
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Show a diff of dependency updates without applying them
    #[arg(long)]
    pub check: bool,

    /// Automatically apply all changes without prompting
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Upgrade dependencies in `uv.lock` with `uv`
    #[arg(short = 'u', long = "upgrade")]
    pub upgrade: bool,

    /// Show detailed information about dependency updates
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}
