use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkLocation {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub deps: Option<Vec<String>>,
    pub location: LinkLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageConfig {
    pub modules: Vec<ModuleConfig>,
    // locations: Option<HashMap<String, String>>,
}

pub fn load_package_config(file: &PathBuf) -> Result<PackageConfig> {
    let data = std::fs::read_to_string(&file).with_context(|| {
        format!(
            "Failed to load file \"{}\".",
            file.as_os_str().to_str().unwrap()
        )
    })?;

    let package_config: PackageConfig = serde_json::from_str::<PackageConfig>(&data)
        .with_context(|| format!("Invalid package configuration."))?;

    Ok(package_config)
}
