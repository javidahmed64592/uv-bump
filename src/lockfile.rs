use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::Path;
use toml::Value;
use uv_bump::{LockDependency, get_error_msg};

pub fn read_lock_versions(path: &Path) -> Result<Vec<LockDependency>> {
    let raw = std::fs::read_to_string(path).with_context(|| {
        get_error_msg(&format!("Failed to read: {}", path.display().bright_blue()))
    })?;

    let doc: Value = toml::from_str(&raw).with_context(|| {
        get_error_msg(&format!(
            "Failed to parse TOML in: {}",
            path.display().bright_blue()
        ))
    })?;

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
            Some(LockDependency {
                name,
                normalised_name,
                version,
            })
        })
        .collect();

    Ok(versions)
}
