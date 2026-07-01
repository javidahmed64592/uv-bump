//! Align `pyproject.toml` dependency constraints with versions resolved by `uv`

mod cli;
mod lockfile;
mod pyproject;

use cli::{parse_cli_args, validate_conflicting_flags};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use lockfile::read_lock_versions;
use owo_colors::OwoColorize;
use pyproject::{apply_changes, read_dependencies};
use std::path::Path;
use uv_align::{
    check_uv_command, compute_dependency_changes, get_success_msg, get_warning_msg,
    map_dependencies, parse_uv_update_output, print_diff, print_uv_modified_dependencies,
    run_uv_lock_upgrade, validate_file_exists, validate_root_directory_exists,
};

const PYPROJECT_FILENAME: &str = "pyproject.toml";
const LOCKFILE_FILENAME: &str = "uv.lock";
const UPDATE_COMMAND: &str = "uv lock --upgrade";

fn main() -> anyhow::Result<()> {
    // Get CLI arguments
    let cli = parse_cli_args();

    if let Err(error) = validate_conflicting_flags(cli.check, cli.yes, "--check", "-y / --yes") {
        eprintln!("{}", error);
        std::process::exit(2);
    }

    // Validate the root directory and required files
    if let Err(error) = validate_root_directory_exists(&cli.path) {
        eprintln!("{}", error);
        std::process::exit(2);
    }
    std::env::set_current_dir(&cli.path)?;

    let pyproject_path = Path::new(PYPROJECT_FILENAME);
    let lockfile_path = Path::new(LOCKFILE_FILENAME);

    if let Err(error) = validate_file_exists(pyproject_path) {
        eprintln!("{}", error);
        std::process::exit(2);
    }

    if let Err(error) = validate_file_exists(lockfile_path) {
        eprintln!("{}", error);
        std::process::exit(2);
    }

    // Upgrade dependencies in `uv.lock` if the upgrade flag is set
    if cli.upgrade {
        println!(
            "Updating dependencies in '{}' using: {}",
            LOCKFILE_FILENAME.bright_blue(),
            UPDATE_COMMAND.bright_green()
        );

        if let Err(error) = check_uv_command() {
            eprintln!("{}", error);
            std::process::exit(127);
        }

        match run_uv_lock_upgrade(UPDATE_COMMAND) {
            Ok(output) => {
                let (updated, added, removed) = parse_uv_update_output(&output);
                print_uv_modified_dependencies(updated, added, removed, cli.verbose);
            }
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(126);
            }
        }
    }

    // Compute and print the diff of dependency changes
    let dependencies = read_dependencies(pyproject_path)?;
    let lock_versions = read_lock_versions(lockfile_path)?;

    let mapped_dependencies = map_dependencies(&dependencies, &lock_versions);
    let diff = compute_dependency_changes(&mapped_dependencies);

    if diff.is_empty() {
        println!("{}", get_warning_msg("Dependencies are already in sync!"));
        return Ok(());
    } else {
        println!("{}", "Changes:\n".underline());
        print_diff(&diff);
        println!(
            "{}{} out of sync in: {}",
            diff.len().to_string().bright_red(),
            if diff.len() == 1 {
                format!("{} is", " dependency".bright_red())
            } else {
                format!("{} are", " dependencies".bright_red())
            },
            PYPROJECT_FILENAME.bright_blue()
        );
    }

    // Exit after printing the diff if the check flag is set
    if cli.check {
        println!(
            "{}",
            get_success_msg(&format!(
                "Run '{} {}' without the '{}' flag to apply changes.",
                "uv-align".bright_green(),
                cli.path.display().to_string().bright_green(),
                "--check".bright_green()
            ))
        );
        std::process::exit(1);
    }

    // Confirm before applying changes if the yes flag is not set
    if !cli.yes {
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

        if confirmed {
            println!("y");
        } else {
            println!("N");
            println!("{}", get_warning_msg("Aborting changes..."));
            return Ok(());
        }
    }

    // Apply the changes to `pyproject.toml`
    apply_changes(pyproject_path, &diff, &dependencies)?;
    println!("{}", get_success_msg("Changes applied successfully!"));
    Ok(())
}
