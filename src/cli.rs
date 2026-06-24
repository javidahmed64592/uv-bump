//! Command-line interface for uv-bump

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "uv-bump",
    about = "Update dependency constraints using versions resolved by `uv`"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check for dependency updates and show a diff
    Check {
        /// Path to folder containing pyproject.toml and uv.lock
        #[arg(default_value = ".")]
        path: String,
    },

    /// Apply dependency updates to pyproject.toml
    Apply {
        /// Path to folder containing pyproject.toml and uv.lock
        #[arg(default_value = ".")]
        path: String,

        /// Automatically apply all changes without prompting
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Prompt interactively for each change
        #[arg(short = 'i', long = "interactive")]
        interactive: bool,
    },

    /// Upgrade dependencies with uv and apply updates
    Update {
        /// Path to folder containing pyproject.toml and uv.lock
        #[arg(default_value = ".")]
        path: String,

        /// Automatically apply all changes without prompting
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Prompt interactively for each change
        #[arg(short = 'i', long = "interactive")]
        interactive: bool,
    },
}
