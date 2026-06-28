use owo_colors::OwoColorize;

// General methods
pub fn get_success_msg(msg: &str) -> String {
    format!("{} {}", "✔".bright_green(), msg)
}

pub fn get_warning_msg(msg: &str) -> String {
    format!("{} {}", "⚠".bright_yellow(), msg)
}

pub fn get_error_msg(msg: &str) -> String {
    format!("{} {}", "✖".bright_red(), msg)
}

// Dependencies
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

#[derive(Debug, Clone)]
pub struct LockDependency {
    /// The name of the dependency as written by uv.
    pub name: String,
    /// The normalised name of the dependency (per PEP 503).
    pub normalised_name: String,
    /// The version of the dependency.
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct MappedDependency {
    /// The dependency as read from pyproject.toml.
    pub pyproject: PyprojectDependency,
    /// The dependency as read from uv.lock.
    pub lock: LockDependency,
}

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
