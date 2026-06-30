use std::{path, process::Output};

use owo_colors::OwoColorize;

// Structs representing dependencies

/// A struct representing a dependency as read from `pyproject.toml`.
#[derive(Debug, Clone)]
pub struct PyprojectDependency {
    /// The name of the dependency as written by the user.
    pub name: String,
    /// The normalised name of the dependency (per PEP 503).
    pub normalised_name: String,
    /// The version of the dependency, if any.
    pub version: Option<String>,
    /// The operator of the dependency, if any (">=", "==", "~=", etc.).
    pub operator: Option<String>,
    /// The suffix of the dependency, if any (",<1.0" or ",!=1.0.0").
    pub suffix: Option<String>,
    /// The group of the dependency, if any.
    pub group: Option<String>,
}

/// A struct representing a dependency as read from `uv.lock`.
#[derive(Debug, Clone)]
pub struct LockDependency {
    /// The name of the dependency as written by uv.
    pub name: String,
    /// The normalised name of the dependency (per PEP 503).
    pub normalised_name: String,
    /// The version of the dependency.
    pub version: String,
}

/// A struct representing a dependency that has been mapped from `pyproject.toml` to `uv.lock`.
#[derive(Debug, Clone)]
pub struct MappedDependency {
    /// The dependency as read from pyproject.toml.
    pub pyproject: PyprojectDependency,
    /// The dependency as read from uv.lock.
    pub lock: LockDependency,
}

/// A struct representing a change in a dependency's version.
#[derive(Debug, Clone)]
pub struct DependencyChange {
    /// The name of the dependency.
    pub name: String,
    /// The operator of the dependency, if any (">=", "==", "~=", etc.).
    pub operator: Option<String>,
    /// The old version number of the dependency.
    pub old: String,
    /// The new version number of the dependency.
    pub new: String,
    /// The suffix of the dependency, if any (",<1.0" or ",!=1.0.0").
    pub suffix: Option<String>,
}

// General methods

/// Get a success message with a green checkmark.
pub fn get_success_msg(msg: &str) -> String {
    format!("{} {}", "✔".bright_green(), msg)
}

/// Get a warning message with a yellow warning sign.
pub fn get_warning_msg(msg: &str) -> String {
    format!("{} {}", "⚠".bright_yellow(), msg)
}

/// Get an error message with a red cross.
pub fn get_error_msg(msg: &str) -> String {
    format!("{} {}", "✖".bright_red(), msg)
}

/// Validate that the specified root directory exists and is a directory.
pub fn validate_root_directory_exists(root_path: &path::Path) -> Result<(), anyhow::Error> {
    if root_path.exists() && root_path.is_dir() {
        Ok(())
    } else {
        return Err(anyhow::anyhow!(get_error_msg(&format!(
            "The specified path does not exist or is not a directory: {}",
            root_path.display().bright_red()
        ))));
    }
}

/// Validate that the specified file exists.
pub fn validate_file_exists(filepath: &path::Path) -> Result<(), anyhow::Error> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| path::PathBuf::from("."));

    if filepath.exists() {
        Ok(())
    } else {
        return Err(anyhow::anyhow!(get_error_msg(&format!(
            "The required file '{}' does not exist at: {}",
            filepath.display().bright_red(),
            cwd.join(filepath).display().bright_red()
        ))));
    }
}

// Methods for handling `uv lock --upgrade`

/// Check `uv` command availability.
pub fn check_uv_command() -> Result<(), anyhow::Error> {
    match std::process::Command::new("uv").arg("--version").output() {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(anyhow::anyhow!(get_error_msg(&format!(
                "Failed to execute '{}'. Ensure it is installed and available in the PATH. Error: {}",
                "uv".bright_red(),
                e.to_string().bright_red()
            ))));
        }
    }
}

/// Run `uv lock --upgrade` command and return the output.
pub fn run_uv_lock_upgrade(update_command: &str) -> Result<Output, anyhow::Error> {
    let split_command = update_command.split_whitespace().collect::<Vec<&str>>();
    let output = std::process::Command::new(split_command[0])
        .args(&split_command[1..])
        .output()
        .map_err(|e| {
            anyhow::anyhow!(get_error_msg(&format!(
                "Failed to execute '{}'. Error: {}",
                update_command.bright_red(),
                e.to_string().bright_red()
            )))
        })?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(get_error_msg(&format!(
            "'{}' command failed with exit code: {}",
            update_command.bright_red(),
            output.status.code().unwrap_or(-1).to_string().bright_red()
        ))));
    }

    Ok(output)
}

/// Collect modified dependencies from output of `uv lock --upgrade`.
/// Returns a tuple of (updated, added, removed) package names.
pub fn parse_uv_update_output(output: &Output) -> (Vec<String>, Vec<String>, Vec<String>) {
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut updated = Vec::new();
    let mut added = Vec::new();
    let mut removed = Vec::new();

    for line in stderr.lines() {
        let trimmed = line.trim_start();
        if let Some(pkg) = trimmed.strip_prefix("Updated ") {
            updated.push(pkg.trim().to_string());
        } else if let Some(pkg) = trimmed.strip_prefix("Added ") {
            added.push(pkg.trim().to_string());
        } else if let Some(pkg) = trimmed.strip_prefix("Removed ") {
            removed.push(pkg.trim().to_string());
        }
    }

    (updated, added, removed)
}

/// Print the summary of modified dependencies after running `uv lock --upgrade`.
pub fn print_uv_modified_dependencies(
    updated: Vec<String>,
    added: Vec<String>,
    removed: Vec<String>,
    verbose: bool,
) {
    let updated_count = updated.len();
    let added_count = added.len();
    let removed_count = removed.len();

    // Print the summary of changes
    if updated_count == 0 && added_count == 0 && removed_count == 0 {
        println!("{}", get_success_msg("Dependencies already up to date!\n"));
    } else {
        let mut parts = Vec::new();
        if updated_count > 0 {
            parts.push(format!("{} updated", updated_count.to_string().bold()));
        }
        if added_count > 0 {
            parts.push(format!("{} added", added_count.to_string().bold()));
        }
        if removed_count > 0 {
            parts.push(format!("{} removed", removed_count.to_string().bold()));
        }
        println!(
            "{}",
            get_success_msg(&format!("Dependencies: {}!\n", parts.join(", ")))
        );

        if verbose {
            println!("Updated dependencies:");
            for dep in updated {
                println!("  {} {}", "~".bright_yellow().bold(), dep);
            }
            println!("Added dependencies:");
            for dep in added {
                println!("  {} {}", "+".bright_green().bold(), dep);
            }
            println!("Removed dependencies:");
            for dep in removed {
                println!("  {} {}", "-".bright_red().bold(), dep);
            }
            println!();
        }
    }
}

// Dependency parsing

/// Normalise a package name per PEP 503:
/// lowercase and collapse runs of [-_.] into a single '-'.
pub fn normalize_name(name: &str) -> String {
    let lower = name.to_lowercase();
    // Replace any run of [-_.] with a single '-'
    let mut result = String::with_capacity(lower.len());
    let mut prev_was_sep = false;
    for ch in lower.chars() {
        if ch == '-' || ch == '_' || ch == '.' {
            if !prev_was_sep {
                result.push('-');
            }
            prev_was_sep = true;
        } else {
            result.push(ch);
            prev_was_sep = false;
        }
    }
    result
}

/// Normalise a version string by stripping trailing `.0` components
fn normalize_version(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    let trimmed = parts.iter().rev().skip_while(|&&p| p == "0").count();
    parts[..trimmed.max(1)].join(".")
}

/// Map dependencies from pyproject.toml to uv.lock based on their normalised names.
pub fn map_dependencies(
    pyproject_deps: &[PyprojectDependency],
    lock_deps: &[LockDependency],
) -> Vec<MappedDependency> {
    let mut mapped = Vec::new();

    for py_dep in pyproject_deps {
        if let Some(lock_dep) = lock_deps
            .iter()
            .find(|lock_dep| lock_dep.normalised_name == py_dep.normalised_name)
        {
            mapped.push(MappedDependency {
                pyproject: py_dep.clone(),
                lock: lock_dep.clone(),
            });
        }
    }

    mapped
}

/// Check which dependencies in pyproject.toml have different versions in uv.lock and return a list of changes.
pub fn compute_dependency_changes(mapped_deps: &[MappedDependency]) -> Vec<DependencyChange> {
    let mut changes = Vec::new();

    for mapped in mapped_deps {
        if let Some(pyproject_version) = &mapped.pyproject.version {
            let lock_version = &mapped.lock.version;

            if normalize_version(pyproject_version) != normalize_version(lock_version) {
                changes.push(DependencyChange {
                    name: mapped.pyproject.name.clone(),
                    operator: mapped.pyproject.operator.clone(),
                    old: pyproject_version.clone(),
                    new: lock_version.clone(),
                    suffix: mapped.pyproject.suffix.clone(),
                });
            }
        }
    }

    changes
}

/// Print the differences between the old and new versions of dependencies.
pub fn print_diff(changes: &[DependencyChange]) {
    for change in changes {
        println!(
            "{} {:<16} {}{}{}",
            "-".bright_red(),
            change.name.bold(),
            change.operator.clone().unwrap_or_default().bright_red(),
            change.old.bright_red().underline(),
            change.suffix.clone().unwrap_or_default().bright_red(),
        );
        println!(
            "{} {:<16} {}{}{}",
            "+".bright_green(),
            change.name.bold(),
            change.operator.clone().unwrap_or_default().bright_green(),
            change.new.bright_green().underline(),
            change.suffix.clone().unwrap_or_default().bright_green()
        );
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── normalize_name ───────────────────────────────────────────────────────

    #[test]
    fn test_normalize_basic() {
        assert_eq!(normalize_name("requests"), "requests");
    }

    #[test]
    fn test_normalize_underscores() {
        assert_eq!(normalize_name("my_package"), "my-package");
    }

    #[test]
    fn test_normalize_dots_and_dashes() {
        assert_eq!(normalize_name("My.Cool-Package"), "my-cool-package");
    }

    #[test]
    fn test_normalize_consecutive_separators() {
        // PEP 503: runs of [-_.] collapse to a single '-'
        assert_eq!(normalize_name("weird___name"), "weird-name");
    }
}
