pub struct Dependency {
    /// The name of the dependency.
    pub name: String,

    /// The version constraint of the dependency i.e. >=, <=, ==, etc.
    pub constraint: String,

    /// The group of the dependency, if any.
    pub group: Option<String>,
}

pub struct LockVersion {
    /// The name of the dependency.
    pub name: String,
    /// The version of the dependency.
    pub version: String,
}

#[derive(Debug)]
pub struct DependencyChange {
    /// The name of the dependency.
    pub name: String,
    /// The old version number and constraint of the dependency.
    pub old: String,
    /// The new version number and constraint of the dependency.
    pub new: String,
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
