//! Update pyproject.toml dependency constraints using versions resolved by uv.

mod cli;
mod diff;
mod lockfile;
mod pyproject;

use anyhow::Context;
use clap::Parser;
use cli::Cli;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use diff::print_diff;
use lockfile::read_lock_versions;
use owo_colors::OwoColorize;
use pyproject::{apply_changes, read_dependencies};
use std::path::Path;
use uv_align::{
    compute_dependency_changes, get_error_msg, get_success_msg, get_warning_msg, map_dependencies,
};

const PYPROJECT_FILENAME: &str = "pyproject.toml";
const LOCKFILE_FILENAME: &str = "uv.lock";
const UPDATE_COMMAND: &str = "uv lock --upgrade";

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
        std::process::exit(2);
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
        std::process::exit(2);
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
        std::process::exit(2);
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
        std::process::exit(2);
    }

    if upgrade_flag {
        println!(
            "Updating dependencies in '{}' using: {}",
            LOCKFILE_FILENAME.bright_blue(),
            UPDATE_COMMAND.bright_green()
        );

        // Check that uv is installed and available in the PATH
        if let Err(e) = std::process::Command::new("uv").arg("--version").output() {
            eprintln!(
                "{}",
                get_error_msg(&format!(
                    "Failed to execute '{}'. Ensure it is installed and available in the PATH. Error: {}",
                    "uv".bright_green(),
                    e.to_string().bright_red()
                ))
            );
            std::process::exit(127);
        }

        // Run `uv lock --upgrade` to update the lockfile
        let split_command = UPDATE_COMMAND.split_whitespace().collect::<Vec<&str>>();
        let status = std::process::Command::new(split_command[0])
            .args(&split_command[1..])
            .status()
            .with_context(|| {
                get_error_msg(&format!(
                    "Failed to execute: '{}'",
                    UPDATE_COMMAND.bright_green()
                ))
            })
            .unwrap_or_else(|e| {
                eprintln!(
                    "{}",
                    get_error_msg(&format!(
                        "Failed to update dependencies using '{}'. Error: {}",
                        UPDATE_COMMAND.bright_green(),
                        e.to_string().bright_red()
                    ))
                );
                std::process::exit(126);
            });

        if !status.success() {
            eprintln!(
                "{}",
                get_error_msg(&format!(
                    "'{}' command failed with exit code: {}",
                    UPDATE_COMMAND.bright_green(),
                    status.code().unwrap_or(-1).to_string().bright_red()
                ))
            );
            std::process::exit(1);
        } else {
            println!(
                "{}",
                get_success_msg("Dependencies updated successfully!\n")
            );
        }
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
            "{} dependency are out of sync in: {}\n",
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
                "uv-align".bright_green(),
                root_path.display().to_string().bright_green(),
                "--check".bright_green()
            ))
        );
        std::process::exit(1);
    }

    // Confirm before applying changes
    if !yes_flag {
        print!("{}", "Apply these changes? [y/N]: ".bright_yellow());
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
            println!("{}", "y".bold());
        } else {
            println!("{}", "N".bold());
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
