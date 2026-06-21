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
