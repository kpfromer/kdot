use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

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

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn get_config_files(folder: &String) -> Result<()> {
    let walker = WalkDir::new(folder).into_iter();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        let file = entry.path();

        // TODO: refactor
        if file.is_file() {
            if let Some(ext) = file.extension() {
                if ext == "json" {
                    println!("{}", file.display());
                }
            }
        }
    }

    Ok(())
}
