//! Update pyproject.toml dependency constraints using versions resolved by uv, with preview and interactive apply support.

mod cli;
mod diff;
mod lockfile;
mod pyproject;

use clap::Parser;
use cli::{Cli, Commands};
use diff::print_diff;
use lockfile::read_lock_versions;
use pyproject::read_dependencies;
use std::path::Path;
use uv_bump::{compute_dependency_changes, map_dependencies};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path } => {
            std::env::set_current_dir(&path)?;
            let dependencies = read_dependencies(Path::new("pyproject.toml"))?;
            let lock_versions = read_lock_versions(Path::new("uv.lock"))?;

            let mapped_dependencies = map_dependencies(&dependencies, &lock_versions);
            let changes = compute_dependency_changes(&mapped_dependencies);

            print_diff(&changes, path);
        }

        Commands::Apply {
            path,
            yes,
            interactive,
        } => {
            println!("Running: apply");
            println!("yes={yes}, interactive={interactive}");

            std::env::set_current_dir(&path)?;

            // TODO: apply logic
        }

        Commands::Update {
            path,
            yes,
            interactive,
        } => {
            println!("Running: update");
            println!("yes={yes}, interactive={interactive}");

            std::env::set_current_dir(&path)?;

            // TODO:
            // 1. uv lock --upgrade
            // 2. compute diff
            // 3. apply changes
        }
    }
    Ok(())
}
