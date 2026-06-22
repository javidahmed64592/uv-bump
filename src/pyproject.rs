use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use toml::Value;

use uv_bump::Dependency;

/// Parse a single PEP 508 dependency string into a `Dependency`.
///
/// PEP 508 grammar (simplified, covering the common cases):
///   name [extras] [version_spec] [; marker]
///
/// Examples:
///   "requests"
///   "fastapi>=0.110.0"
///   "pydantic==2.6.1"
///   "black[d]>=23.0"
///   "httpx>=0.24,<1.0"
///   "mypy>=1.0 ; python_version >= '3.11'"
///
/// We extract:
///   - name  : everything up to the first '[', '>', '<', '=', '~', '!', ';', or whitespace
///   - constraint : the version specifier(s), excluding markers
///   - extras are intentionally discarded (not relevant to version bumping)
pub fn parse_pep508(spec: &str, group: Option<String>) -> Option<Dependency> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }

    // Strip inline comments (uncommon in pyproject but safe to handle)
    let spec = spec.split('#').next().unwrap_or("").trim();
    if spec.is_empty() {
        return None;
    }

    // Strip environment markers: everything from ';' onward
    let spec = spec.split(';').next().unwrap_or("").trim();

    // Find where the name ends. Name characters: letters, digits, '-', '_', '.'
    // It ends at the first '[' (extras), version operator, or whitespace.
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
    let name = uv_bump::normalize_name(raw_name);

    // Everything after the name (and optional extras) is the version specifier.
    let rest = spec[name_end..].trim();

    // Skip optional extras block: [extra1,extra2]
    let rest = if rest.starts_with('[') {
        match rest.find(']') {
            Some(idx) => rest[idx + 1..].trim(),
            None => rest, // malformed but keep going
        }
    } else {
        rest
    };

    // What remains (possibly empty) is the version constraint, e.g. ">=0.110.0" or ">=0.24,<1.0"
    let constraint = rest.to_string();

    Some(Dependency {
        name,
        constraint,
        group,
    })
}

/// Read `pyproject.toml` from `path` and return all dependencies across every group.
///
/// Reads from:
///   [project.dependencies]              → group = None
///   [project.optional-dependencies]     → group = Some("<extra-name>")
///   [dependency-groups]                 → group = Some("<group-name>")   (PEP 735 / uv)
pub fn read_dependencies(path: &Path) -> Result<Vec<Dependency>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let doc: Value = toml::from_str(&raw)
        .with_context(|| format!("Failed to parse TOML in {}", path.display()))?;

    let mut deps: Vec<Dependency> = Vec::new();

    // ── [project.dependencies] ──────────────────────────────────────────────
    if let Some(project) = doc.get("project") {
        if let Some(Value::Array(arr)) = project.get("dependencies") {
            for item in arr {
                if let Value::String(s) = item {
                    if let Some(dep) = parse_pep508(s, None) {
                        deps.push(dep);
                    }
                }
            }
        }

        // ── [project.optional-dependencies] ─────────────────────────────────
        if let Some(Value::Table(opt_deps)) = project.get("optional-dependencies") {
            for (group_name, group_value) in opt_deps {
                if let Value::Array(arr) = group_value {
                    for item in arr {
                        if let Value::String(s) = item {
                            if let Some(dep) = parse_pep508(s, Some(group_name.clone())) {
                                deps.push(dep);
                            }
                        }
                    }
                }
            }
        }
    }

    // ── [dependency-groups]  (PEP 735 — supported by uv) ───────────────────
    // Values can be plain strings OR inline tables { include-group = "..." }.
    // We only care about the string entries.
    if let Some(Value::Table(dg)) = doc.get("dependency-groups") {
        for (group_name, group_value) in dg {
            if let Value::Array(arr) = group_value {
                for item in arr {
                    match item {
                        Value::String(s) => {
                            if let Some(dep) = parse_pep508(s, Some(group_name.clone())) {
                                deps.push(dep);
                            }
                        }
                        // { include-group = "other" } — skip, not a package specifier
                        Value::Table(_) => {}
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(deps)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_pep508 ─────────────────────────────────────────────────────────

    #[test]
    fn test_parse_bare_name() {
        let dep = parse_pep508("requests", None).unwrap();
        assert_eq!(dep.name, "requests");
        assert_eq!(dep.constraint, "");
        assert!(dep.group.is_none());
    }

    #[test]
    fn test_parse_gte_constraint() {
        let dep = parse_pep508("fastapi>=0.110.0", None).unwrap();
        assert_eq!(dep.name, "fastapi");
        assert_eq!(dep.constraint, ">=0.110.0");
    }

    #[test]
    fn test_parse_eq_constraint() {
        let dep = parse_pep508("pydantic==2.6.1", None).unwrap();
        assert_eq!(dep.name, "pydantic");
        assert_eq!(dep.constraint, "==2.6.1");
    }

    #[test]
    fn test_parse_multiple_specifiers() {
        let dep = parse_pep508("httpx>=0.24,<1.0", None).unwrap();
        assert_eq!(dep.name, "httpx");
        assert_eq!(dep.constraint, ">=0.24,<1.0");
    }

    #[test]
    fn test_parse_extras_ignored() {
        let dep = parse_pep508("black[d]>=23.0", None).unwrap();
        assert_eq!(dep.name, "black");
        assert_eq!(dep.constraint, ">=23.0");
    }

    #[test]
    fn test_parse_marker_stripped() {
        let dep = parse_pep508("tomli>=2.0 ; python_version < '3.11'", None).unwrap();
        assert_eq!(dep.name, "tomli");
        assert_eq!(dep.constraint, ">=2.0");
    }

    #[test]
    fn test_parse_normalizes_name() {
        let dep = parse_pep508("my_package>=1.0", None).unwrap();
        assert_eq!(dep.name, "my-package");
    }

    #[test]
    fn test_parse_group_propagated() {
        let dep = parse_pep508("pytest>=7.0", Some("dev".into())).unwrap();
        assert_eq!(dep.group, Some("dev".to_string()));
    }

    #[test]
    fn test_parse_compatible_release() {
        let dep = parse_pep508("numpy~=1.24", None).unwrap();
        assert_eq!(dep.name, "numpy");
        assert_eq!(dep.constraint, "~=1.24");
    }

    #[test]
    fn test_parse_not_equal() {
        let dep = parse_pep508("celery!=4.0", None).unwrap();
        assert_eq!(dep.name, "celery");
        assert_eq!(dep.constraint, "!=4.0");
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_pep508("", None).is_none());
        assert!(parse_pep508("   ", None).is_none());
    }

    // ── read_dependencies integration ────────────────────────────────────────

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
        let map: HashMap<String, &Dependency> = deps.iter().map(|d| (d.name.clone(), d)).collect();

        assert_eq!(map["requests"].constraint, ">=2.28");
        assert_eq!(map["fastapi"].constraint, ">=0.110.0");
        assert_eq!(map["pytest"].group, Some("dev".to_string()));
        assert_eq!(map["mypy"].constraint, ">=1.0");
        assert_eq!(map["sphinx"].group, Some("docs".to_string()));
        assert_eq!(map["ruff"].group, Some("lint".to_string()));

        // include-group table entry should NOT produce a dependency
        assert!(!map.contains_key("include-group"));
    }
}
