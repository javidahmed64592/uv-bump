use anyhow::{Context, Result};
use std::path::Path;
use toml::Value;

use uv_bump::LockVersion;

pub fn read_lock_versions(path: &Path) -> Result<Vec<LockVersion>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let doc: Value = toml::from_str(&raw)
        .with_context(|| format!("Failed to parse TOML in {}", path.display()))?;

    let packages = match doc.get("package") {
        Some(Value::Array(arr)) => arr,
        _ => return Ok(vec![]),
    };

    let versions = packages
        .iter()
        .filter_map(|pkg| {
            let name = pkg.get("name")?.as_str()?.to_string();
            let normalised_name = uv_bump::normalize_name(&name);
            let version = pkg.get("version")?.as_str()?.to_string();
            Some(LockVersion {
                name: name,
                normalised_name: normalised_name,
                version: version,
            })
        })
        .collect();

    Ok(versions)
}
