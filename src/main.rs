//! Update pyproject.toml dependency constraints using versions resolved by uv, with preview and interactive apply support.

mod cli;
mod diff;

use clap::Parser;
use cli::{Cli, Commands};
use diff::print_diff;
use uv_bump::DependencyChange;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => {
            let example_changes = vec![
                DependencyChange {
                    name: "example-dep".to_string(),
                    old: ">=1.0.0".to_string(),
                    new: ">=1.1.0".to_string(),
                },
                DependencyChange {
                    name: "another-dep".to_string(),
                    old: "==2.3.4".to_string(),
                    new: "==2.4.0".to_string(),
                },
            ];
            print_diff(&example_changes);
        }

        Commands::Apply { yes, interactive } => {
            println!("Running: apply");
            println!("yes={yes}, interactive={interactive}");

            // TODO: apply logic
        }

        Commands::Update { yes, interactive } => {
            println!("Running: update");
            println!("yes={yes}, interactive={interactive}");

            // TODO:
            // 1. uv lock --upgrade
            // 2. compute diff
            // 3. apply changes
        }
    }
}
