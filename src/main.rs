//! Update dependencies in `pyproject.toml` using versions resolved by `uv`

mod cli;
mod diff;
mod lockfile;
mod pyproject;

use clap::Parser;
use cli::Cli;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use diff::print_diff;
use lockfile::read_lock_versions;
use owo_colors::OwoColorize;
use pyproject::{apply_changes, read_dependencies};
use std::path::Path;
use uv_bump::{
    compute_dependency_changes, get_error_msg, get_success_msg, get_warning_msg, map_dependencies,
};

const PYPROJECT_FILENAME: &str = "pyproject.toml";
const LOCKFILE_FILENAME: &str = "uv.lock";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let root_path = cli.path.clone();
    let check_flag = cli.check;
    let yes_flag = cli.yes;
    let upgrade_flag = cli.upgrade;

    // Ensure check and yes flags are not both specified
    if check_flag && yes_flag {
        eprintln!(
            "{}",
            get_error_msg(&format!(
                "The '{}' and '{}' flags cannot be used together.",
                "--check".bright_green(),
                "--yes".bright_green()
            ))
        );
        std::process::exit(1);
    }

    // Check if the path exists and is a directory
    if !root_path.exists() || !root_path.is_dir() {
        eprintln!(
            "{}",
            get_error_msg(&format!(
                "The specified path does not exist or is not a directory: {}",
                root_path.display().bright_blue()
            ))
        );
        std::process::exit(1);
    }
    std::env::set_current_dir(&root_path)?;

    // Check if pyproject.toml and uv.lock exist in the specified path
    let pyproject_path = Path::new(PYPROJECT_FILENAME);
    let lockfile_path = Path::new(LOCKFILE_FILENAME);

    if !pyproject_path.exists() {
        eprintln!(
            "{}",
            get_error_msg(&format!(
                "'{}' does not exist in the specified path: {}",
                PYPROJECT_FILENAME.bright_blue(),
                root_path.display().bright_blue()
            ))
        );
        std::process::exit(1);
    }

    if !lockfile_path.exists() {
        eprintln!(
            "{}",
            get_error_msg(&format!(
                "'{}' does not exist in the specified path: {}",
                LOCKFILE_FILENAME.bright_blue(),
                root_path.display().bright_blue()
            ))
        );
        std::process::exit(1);
    }

    // TODO: Upgrade dependencies with uv if the upgrade flag is set
    if upgrade_flag {
        println!(
            "Updating dependencies in '{}' using '{}'...",
            LOCKFILE_FILENAME.bright_blue(),
            "uv".bright_green()
        );
        todo!("Implement uv upgrade functionality");
    }

    // Compute and print the diff of dependency changes
    let dependencies = read_dependencies(pyproject_path)?;
    let lock_versions = read_lock_versions(lockfile_path)?;

    let mapped_dependencies = map_dependencies(&dependencies, &lock_versions);
    let diff = compute_dependency_changes(&mapped_dependencies);

    // If there are no changes, exit early
    if diff.is_empty() {
        println!("{}", get_success_msg("Dependencies are already in sync!"));
        return Ok(());
    } else {
        println!("{}", "Changes:\n".bold().underline());
        print_diff(&diff);
        println!(
            "{} dependency are out of sync in: {}",
            diff.len().to_string().bold(),
            PYPROJECT_FILENAME.bright_blue()
        );
    }

    // If the check flag is set, exit after printing the diff
    if check_flag {
        println!(
            "{}",
            get_success_msg(&format!(
                "Run '{} {}' without the '{}' flag to apply changes.",
                "uv-bump".bright_green(),
                root_path.display().to_string().bright_green(),
                "--check".bright_green()
            ))
        );
        return Ok(());
    }

    // Confirm before applying changes
    if !yes_flag {
        print!("{}", "Apply these changes? (y/N) ".bright_yellow());
        std::io::Write::flush(&mut std::io::stdout())?;

        terminal::enable_raw_mode()?;
        let confirmed = loop {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                break matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y'));
            }
        };
        terminal::disable_raw_mode()?;

        // Print the key the user pressed so the line feels complete
        if confirmed {
            println!("y");
        } else {
            println!("N");
        }

        if !confirmed {
            println!("{}", get_warning_msg("Aborting changes..."));
            return Ok(());
        }
    }

    println!("Applying changes...");
    apply_changes(pyproject_path, &diff, &dependencies)?;
    println!("{}", get_success_msg("Changes applied successfully!"));
    Ok(())
}
