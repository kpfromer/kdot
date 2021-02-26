use crate::{
    config::{ModuleConfig, PackageConfig},
    path::absolute_path,
    symlink,
};
use anyhow::{Context, Result};
use std::{collections::HashMap, iter::FromIterator};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

// TODO: check is valid to link (no overrides)

pub fn link_module(module: &ModuleConfig) -> Result<()> {
    let location = module.get_link_location();
    let to = absolute_path(&Path::new(&location.to))?;
    let from = absolute_path(&Path::new(&location.from))?;

    info!(
        "Linking \"{}\" to \"{}\"",
        to.as_os_str().to_str().unwrap(),
        from.as_os_str().to_str().unwrap(),
    );

    symlink::link_folder(&to, &from, true)
        .with_context(|| format!("Failed to link \"{}\" module.", module.name))?;

    info!("Linked \"{}\" module.", module.name);

    Ok(())
}

pub fn unlink_module(module: &ModuleConfig) -> Result<()> {
    info!("Unlinking {} module.", &module.name);
    let location = module.get_link_location();
    symlink::unlink_folder(
        &PathBuf::from(&location.to),
        &PathBuf::from(&location.from),
        true,
    )?;

    Ok(())
}

// TODO: is the best way of handling this?
// should it be HashSet<String>?
pub fn get_matching_modules<'a>(
    kdot_config: &'a PackageConfig,
    modules_names: &'a [String],
) -> HashSet<&'a String> {
    let config_module_names: HashSet<&'a String> = kdot_config
        .modules
        .iter()
        .map(|module| &module.name)
        .collect();

    let modules_names: HashSet<&'a String> = HashSet::from_iter(modules_names);

    modules_names
        .intersection(&config_module_names)
        .cloned()
        .collect()
}

pub fn get_module_map<'a>(kdot_config: &'a PackageConfig) -> HashMap<String, &'a ModuleConfig> {
    let mut map: HashMap<String, &'a ModuleConfig> = HashMap::new();

    kdot_config.modules.iter().for_each(|module| {
        map.insert(module.name.clone(), module);
    });

    map
}
