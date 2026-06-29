//! Command-line interface for uv-align

use std::path::PathBuf;

use clap::Parser;
use owo_colors::OwoColorize;
use uv_align::get_error_msg;

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

pub struct CliArgs {
    pub path: PathBuf,
    pub check: bool,
    pub yes: bool,
    pub upgrade: bool,
    pub verbose: bool,
}

pub fn parse_cli_args() -> CliArgs {
    let cli = Cli::parse();
    CliArgs {
        path: cli.path.clone(),
        check: cli.check,
        yes: cli.yes,
        upgrade: cli.upgrade,
        verbose: cli.verbose,
    }
}

pub fn validate_conflicting_flags(
    flag_1: bool,
    flag_2: bool,
    flag_1_name: &str,
    flag_2_name: &str,
) -> anyhow::Result<()> {
    if flag_1 && flag_2 {
        eprintln!(
            "{}",
            get_error_msg(&format!(
                "The '{}' and '{}' flags cannot be used together.",
                flag_1_name.bright_green(),
                flag_2_name.bright_green()
            ))
        );
        std::process::exit(2);
    }
    Ok(())
}
