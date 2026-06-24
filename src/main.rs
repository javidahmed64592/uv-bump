//! Update pyproject.toml dependency constraints using versions resolved by uv, with preview and interactive apply support.

mod cli;
mod diff;
mod lockfile;
mod pyproject;

use clap::Parser;
use cli::Cli;
use diff::print_diff;
use lockfile::read_lock_versions;
use owo_colors::OwoColorize;
use pyproject::read_dependencies;
use std::path::Path;
use uv_bump::{compute_dependency_changes, map_dependencies};

const PYPROJECT_FILENAME: &str = "pyproject.toml";
const LOCKFILE_FILENAME: &str = "uv.lock";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let root_path = cli.path.clone();
    let check_flag = cli.check;
    let yes_flag = cli.yes;
    let interactive_flag = cli.interactive;
    let upgrade_flag = cli.upgrade;

    // Ensure check and yes flags are not both specified
    if check_flag && yes_flag {
        eprintln!(
            "{} The {} and {} flags cannot be used together.",
            "✖".bright_red(),
            "--check".bright_green(),
            "--yes".bright_green()
        );
        std::process::exit(1);
    }

    // Ensure yes and interactive flags are not both specified
    if yes_flag && interactive_flag {
        eprintln!(
            "{} The {} and {} flags cannot be used together.",
            "✖".bright_red(),
            "--yes".bright_green(),
            "--interactive".bright_green()
        );
        std::process::exit(1);
    }

    // Check if the path exists and is a directory
    if !root_path.exists() || !root_path.is_dir() {
        eprintln!(
            "{} The specified path does not exist or is not a directory: {}",
            "✖".bright_red(),
            root_path.display().blue()
        );
        std::process::exit(1);
    }
    std::env::set_current_dir(&root_path)?;

    // Check if pyproject.toml and uv.lock exist in the specified path
    let pyproject_path = Path::new(PYPROJECT_FILENAME);
    let lockfile_path = Path::new(LOCKFILE_FILENAME);

    if !pyproject_path.exists() {
        eprintln!(
            "{} '{}' does not exist in the specified path: {}",
            "✖".bright_red(),
            PYPROJECT_FILENAME.blue(),
            root_path.display().blue()
        );
        std::process::exit(1);
    }

    if !lockfile_path.exists() {
        eprintln!(
            "{} '{}' does not exist in the specified path: {}",
            "✖".bright_red(),
            LOCKFILE_FILENAME.blue(),
            root_path.display().blue()
        );
        std::process::exit(1);
    }

    // TODO: Upgrade dependencies with uv if the upgrade flag is set
    if upgrade_flag {
        println!(
            "Updating dependencies in '{}' using 'uv'...",
            LOCKFILE_FILENAME.blue()
        );
        todo!("Implement uv upgrade functionality");
    }

    // Compute and print the diff of dependency changes
    let dependencies = read_dependencies(pyproject_path)?;
    let lock_versions = read_lock_versions(lockfile_path)?;

    let mapped_dependencies = map_dependencies(&dependencies, &lock_versions);
    let diff = compute_dependency_changes(&mapped_dependencies);

    print_diff(&diff);

    // If the check flag is set, exit after printing the diff
    if check_flag {
        return Ok(());
    }

    // If there are no changes, exit early
    if diff.is_empty() {
        return Ok(());
    }

    // TODO: Apply changes
    todo!("Implement apply functionality with --yes and --interactive flags");
}
