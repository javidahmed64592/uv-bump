//! Command-line interface for the tool

use std::path::PathBuf;

use clap::Parser;
use owo_colors::OwoColorize;
use uv_align::get_error_msg;

#[derive(Parser, Debug)]
#[command(
    name = "uv-align",
    about = "Align `pyproject.toml` dependency constraints with versions resolved by `uv`"
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

    /// Upgrade dependencies in `uv.lock` with `uv lock --upgrade`
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

/// Parse command-line arguments and return a `CliArgs` struct
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

/// Validate that two flags are not used together
pub fn validate_conflicting_flags(
    flag_1: bool,
    flag_2: bool,
    flag_1_name: &str,
    flag_2_name: &str,
) -> anyhow::Result<()> {
    if flag_1 && flag_2 {
        return Err(anyhow::anyhow!(get_error_msg(&format!(
            "The '{}' and '{}' flags cannot be used together.",
            flag_1_name.bright_red(),
            flag_2_name.bright_red()
        ))));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_conflicting_flags() {
        let result = validate_conflicting_flags(true, true, "flag1", "flag2");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_non_conflicting_flags() {
        let result = validate_conflicting_flags(true, false, "flag1", "flag2");
        assert!(result.is_ok());
    }
}
