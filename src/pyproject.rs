use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::Path;
use toml::Value;
use toml_edit::{DocumentMut, Item};

use uv_align::{DependencyChange, PyprojectDependency, get_error_msg, normalise_dependency_name};

// Parsing methods

/// Struct representing a version constraint in a PEP 508 dependency string.
struct VersionConstraint {
    pub operator: Option<String>,
    pub version: Option<String>,
    pub suffix: Option<String>,
}

/// Parse a single version constraint string into its components.
///
/// Returns `(operator, version, suffix)`, where:
///   - `operator` is the leading specifier token e.g. `>=`, `==`
///   - `version`  is the lower-bound version number e.g. `0.110.0`
///   - `suffix`   is any trailing specifiers e.g. `,<1.0`
///
/// Returns `(None, None, None)` for:
///   - empty strings
///   - URL/git dependencies (`@ git+...`)
fn parse_version_constraint(s: &str) -> VersionConstraint {
    if s.is_empty() || s.starts_with('@') {
        return VersionConstraint {
            operator: None,
            version: None,
            suffix: None,
        };
    }

    // Operator: one of >=, <=, ==, ~=, !=, >, <
    let op_end = s.find(|c: char| c.is_ascii_digit()).unwrap_or(s.len());
    let operator = s[..op_end].to_string();
    let rest = &s[op_end..];

    // Version: everything up to the next comma (start of suffix) or end of string
    let version_end = rest.find(',').unwrap_or(rest.len());
    let version = rest[..version_end].trim().to_string();

    // Suffix: any trailing specifiers e.g. ",<1.0" - preserved verbatim on write-back
    let suffix = if version_end < rest.len() {
        Some(rest[version_end..].to_string())
    } else {
        None
    };

    VersionConstraint {
        operator: Some(operator),
        version: Some(version),
        suffix,
    }
}

/// Parse a single PEP 508 dependency string into a [`PyprojectDependency`].
///
/// Handles the common PEP 508 grammar:
/// ```text
/// name [extras] [version_spec] [; marker]
/// ```
///
/// Examples:
/// ```text
/// "requests"
/// "fastapi>=0.110.0"
/// "pydantic==2.6.1"
/// "black[d]>=23.0"
/// "httpx>=0.24,<1.0"
/// "mypy>=1.0 ; python_version >= '3.11'"
/// ```
///
/// Extras are discarded as they are not relevant to version bumping.
/// Environment markers are stripped.
/// Git/URL dependencies (`@ git+...`) produce `operator = None`, `version = None`.
fn parse_pep508_string(spec: &str, group: Option<String>) -> Option<PyprojectDependency> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }

    // Strip inline comments (uncommon in pyproject.toml but safe to handle)
    let spec = spec.split('#').next().unwrap_or("").trim();
    if spec.is_empty() {
        return None;
    }

    // Strip environment markers: everything from ';' onward
    let spec = spec.split(';').next().unwrap_or("").trim();

    // Name ends at the first '[' (extras), version operator, or whitespace.
    // Name characters are: letters, digits, '-', '_', '.'
    let name_end = spec
        .find(|c: char| {
            c == '['
                || c == '>'
                || c == '<'
                || c == '='
                || c == '~'
                || c == '!'
                || c.is_whitespace()
        })
        .unwrap_or(spec.len());

    let raw_name = &spec[..name_end];
    if raw_name.is_empty() {
        return None;
    }
    let normalised_name = normalise_dependency_name(raw_name);

    // Everything after the name (and optional extras) is the version specifier
    let rest = spec[name_end..].trim();

    // Skip optional extras block e.g. [standard], [dev,docs]
    let rest = if rest.starts_with('[') {
        match rest.find(']') {
            Some(idx) => rest[idx + 1..].trim(),
            None => rest, // malformed extras, keep going
        }
    } else {
        rest
    };

    // Parse the remaining constraint string e.g. ">=0.110.0" or ">=0.24,<1.0"
    let constraint = parse_version_constraint(rest);

    Some(PyprojectDependency {
        name: raw_name.to_string(),
        normalised_name,
        operator: constraint.operator,
        version: constraint.version,
        suffix: constraint.suffix,
        group,
    })
}

// Read methods

/// Read `pyproject.toml` from `path` and return all dependencies across every group.
///
/// Reads from:
///   - `[project.dependencies]`            → `group = None`
///   - `[project.optional-dependencies]`   → `group = Some("<extra-name>")`
///   - `[dependency-groups]`               → `group = Some("<group-name>")` (PEP 735 / uv)
///
/// Git/URL dependencies and bare names (no version constraint) are included with
/// `operator = None` and `version = None`, and will be skipped during diff computation.
pub fn read_dependencies(path: &Path) -> Result<Vec<PyprojectDependency>> {
    let raw = std::fs::read_to_string(path).with_context(|| {
        get_error_msg(&format!("Failed to read: {}", path.display().bright_blue()))
    })?;

    let doc: Value = toml::from_str(&raw).with_context(|| {
        get_error_msg(&format!(
            "Failed to parse TOML in: {}",
            path.display().bright_blue()
        ))
    })?;

    let mut deps: Vec<PyprojectDependency> = Vec::new();

    // ── [project.dependencies] ──────────────────────────────────────────────
    if let Some(project) = doc.get("project") {
        if let Some(Value::Array(arr)) = project.get("dependencies") {
            for item in arr {
                if let Value::String(s) = item
                    && let Some(dep) = parse_pep508_string(s, None)
                {
                    deps.push(dep);
                }
            }
        }

        // ── [project.optional-dependencies] ─────────────────────────────────
        if let Some(Value::Table(opt_deps)) = project.get("optional-dependencies") {
            for (group_name, group_value) in opt_deps {
                if let Value::Array(arr) = group_value {
                    for item in arr {
                        if let Value::String(s) = item
                            && let Some(dep) = parse_pep508_string(s, Some(group_name.clone()))
                        {
                            deps.push(dep);
                        }
                    }
                }
            }
        }
    }

    // ── [dependency-groups] (PEP 735 - supported by uv) ────────────────────
    // Values can be plain strings OR inline tables e.g. `{ include-group = "..." }`.
    // Only string entries are package specifiers; table entries are skipped.
    if let Some(Value::Table(dg)) = doc.get("dependency-groups") {
        for (group_name, group_value) in dg {
            if let Value::Array(arr) = group_value {
                for item in arr {
                    match item {
                        Value::String(s) => {
                            if let Some(dep) = parse_pep508_string(s, Some(group_name.clone())) {
                                deps.push(dep);
                            }
                        }
                        Value::Table(_) => {} // { include-group = "other" } - not a package
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(deps)
}

// Write methods

/// Find the entry in a TOML array whose string value contains `name`, and replace
/// it with `new_spec`, preserving the original entry's surrounding whitespace and
/// comments via `toml_edit`'s decoration API.
fn replace_in_array(item: &mut Item, name: &str, new_spec: &str) {
    let Some(array) = item.as_array_mut() else {
        return;
    };

    let Some(idx) = array
        .iter()
        .position(|v| v.as_str().is_some_and(|s| s.contains(name)))
    else {
        return;
    };

    array.replace(idx, new_spec);
}

/// Write dependency version changes back to `pyproject.toml` at `path`.
///
/// Uses [`toml_edit`] to perform format-preserving edits - only the version
/// numbers are modified; all comments, whitespace, and key ordering are retained.
///
/// Each changed dependency is looked up in `deps` to determine its group
/// (which TOML array to update) and to reconstruct the full PEP 508 string
/// with the new version, preserving the original operator and any suffix
/// constraints e.g. `,<1.0`.
pub fn apply_changes(
    path: &Path,
    changes: &[DependencyChange],
    deps: &[PyprojectDependency],
) -> Result<()> {
    let raw = std::fs::read_to_string(path).with_context(|| {
        get_error_msg(&format!("Failed to read: {}", path.display().bright_blue()))
    })?;

    let mut doc: DocumentMut = raw.parse().with_context(|| {
        get_error_msg(&format!(
            "Failed to parse TOML in: {}",
            path.display().bright_blue()
        ))
    })?;

    for change in changes {
        let Some(dep) = deps.iter().find(|d| d.name == change.name) else {
            continue;
        };

        // Rebuild the full PEP 508 string with the updated version, preserving
        // the original operator and any suffix constraints e.g. ",<1.0"
        let new_spec = format!(
            "{}{}{}{}",
            dep.name,
            change.operator.as_deref().unwrap_or(""),
            change.new,
            dep.suffix.as_deref().unwrap_or("")
        );

        match &dep.group {
            // [project.dependencies]
            None => {
                replace_in_array(&mut doc["project"]["dependencies"], &dep.name, &new_spec);
            }
            // [project.optional-dependencies.<group>]
            Some(group) => {
                replace_in_array(
                    &mut doc["project"]["optional-dependencies"][group],
                    &dep.name,
                    &new_spec,
                );
            }
        }
    }

    std::fs::write(path, doc.to_string()).with_context(|| {
        get_error_msg(&format!(
            "Failed to write: {}",
            path.display().bright_blue()
        ))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Parsing methods

    #[test]
    fn test_parse_version_constraint() {
        let vc = parse_version_constraint(">=0.110.0");
        assert_eq!(vc.operator, Some(">=".to_string()));
        assert_eq!(vc.version, Some("0.110.0".to_string()));
        assert_eq!(vc.suffix, None);

        let vc2 = parse_version_constraint(">=0.24,<1.0");
        assert_eq!(vc2.operator, Some(">=".to_string()));
        assert_eq!(vc2.version, Some("0.24".to_string()));
        assert_eq!(vc2.suffix, Some(",<1.0".to_string()));

        let vc3 = parse_version_constraint("@ git+https://github.com/user/repo.git");
        assert_eq!(vc3.operator, None);
        assert_eq!(vc3.version, None);
        assert_eq!(vc3.suffix, None);
    }

    #[test]
    fn test_parse_bare_name() {
        let dep = parse_pep508_string("requests", None).unwrap();
        assert_eq!(dep.name, "requests");
        assert_eq!(dep.operator, None);
        assert_eq!(dep.version, None);
        assert_eq!(dep.suffix, None);
        assert!(dep.group.is_none());
    }

    #[test]
    fn test_parse_gte_constraint() {
        let dep = parse_pep508_string("fastapi>=0.110.0", None).unwrap();
        assert_eq!(dep.name, "fastapi");
        assert_eq!(dep.operator, Some(">=".to_string()));
        assert_eq!(dep.version, Some("0.110.0".to_string()));
        assert_eq!(dep.suffix, None);
    }

    #[test]
    fn test_parse_eq_constraint() {
        let dep = parse_pep508_string("pydantic==2.6.1", None).unwrap();
        assert_eq!(dep.name, "pydantic");
        assert_eq!(dep.operator, Some("==".to_string()));
        assert_eq!(dep.version, Some("2.6.1".to_string()));
        assert_eq!(dep.suffix, None);
    }

    #[test]
    fn test_parse_compatible_release() {
        let dep = parse_pep508_string("numpy~=1.24", None).unwrap();
        assert_eq!(dep.name, "numpy");
        assert_eq!(dep.operator, Some("~=".to_string()));
        assert_eq!(dep.version, Some("1.24".to_string()));
        assert_eq!(dep.suffix, None);
    }

    #[test]
    fn test_parse_not_equal() {
        let dep = parse_pep508_string("celery!=4.0", None).unwrap();
        assert_eq!(dep.name, "celery");
        assert_eq!(dep.operator, Some("!=".to_string()));
        assert_eq!(dep.version, Some("4.0".to_string()));
        assert_eq!(dep.suffix, None);
    }

    #[test]
    fn test_parse_multiple_specifiers() {
        let dep = parse_pep508_string("httpx>=0.24,<1.0", None).unwrap();
        assert_eq!(dep.name, "httpx");
        assert_eq!(dep.operator, Some(">=".to_string()));
        assert_eq!(dep.version, Some("0.24".to_string()));
        assert_eq!(dep.suffix, Some(",<1.0".to_string()));
    }
    #[test]
    fn test_parse_extras_ignored() {
        let dep = parse_pep508_string("black[d]>=23.0", None).unwrap();
        assert_eq!(dep.name, "black");
        assert_eq!(dep.operator, Some(">=".to_string()));
        assert_eq!(dep.version, Some("23.0".to_string()));
        assert_eq!(dep.suffix, None);
    }

    #[test]
    fn test_parse_marker_stripped() {
        let dep = parse_pep508_string("tomli>=2.0 ; python_version < '3.11'", None).unwrap();
        assert_eq!(dep.name, "tomli");
        assert_eq!(dep.operator, Some(">=".to_string()));
        assert_eq!(dep.version, Some("2.0".to_string()));
        assert_eq!(dep.suffix, None);
    }

    #[test]
    fn test_parse_group_propagated() {
        let dep = parse_pep508_string("pytest>=7.0", Some("dev".into())).unwrap();
        assert_eq!(dep.group, Some("dev".to_string()));
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_pep508_string("", None).is_none());
        assert!(parse_pep508_string("   ", None).is_none());
    }

    // Read methods

    #[test]
    fn test_read_full_pyproject() {
        use std::io::Write;
        let toml = r#"
[project]
name = "myapp"
version = "0.1.0"
dependencies = [
    "requests>=2.28",
    "fastapi>=0.110.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "mypy>=1.0",
]
docs = [
    "sphinx>=6.0",
]

[dependency-groups]
lint = [
    "ruff>=0.1",
    { include-group = "dev" },
]
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pyproject.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(toml.as_bytes()).unwrap();

        let deps = read_dependencies(&path).unwrap();

        // Collect into a map for easy assertion
        let map: HashMap<String, &PyprojectDependency> =
            deps.iter().map(|d| (d.name.clone(), d)).collect();

        assert_eq!(map["requests"].operator, Some(">=".to_string()));
        assert_eq!(map["requests"].version, Some("2.28".to_string()));
        assert_eq!(map["requests"].suffix, None);
        assert_eq!(map["fastapi"].operator, Some(">=".to_string()));
        assert_eq!(map["fastapi"].version, Some("0.110.0".to_string()));
        assert_eq!(map["fastapi"].suffix, None);
        assert_eq!(map["pytest"].group, Some("dev".to_string()));
        assert_eq!(map["mypy"].operator, Some(">=".to_string()));
        assert_eq!(map["mypy"].version, Some("1.0".to_string()));
        assert_eq!(map["mypy"].suffix, None);
        assert_eq!(map["sphinx"].group, Some("docs".to_string()));
        assert_eq!(map["ruff"].group, Some("lint".to_string()));

        // include-group table entry should NOT produce a dependency
        assert!(!map.contains_key("include-group"));
    }
}
