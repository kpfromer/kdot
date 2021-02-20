#![allow(clippy::useless_format)]

#[macro_use]
extern crate log;
extern crate simplelog;

use anyhow::{bail, Context, Result};
use config::{ModuleConfig, PackageConfig};
use simplelog::*;
use std::{collections::HashMap, iter::FromIterator};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

mod config;
mod path;
mod symlink;

use path::*;

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(subcommand)]
    pattern: Command,

    #[structopt(short = "v", long = "verbose")]
    verbosity: Option<String>,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Links the module to the system.
    Link { modules: Vec<String> },
    /// Unlinks the module to the system.
    Unlink { modules: Vec<String> },
    /// Unlinks and then relinks the module to the system.
    Sync { modules: Vec<String> },
}

fn link_module(module: &ModuleConfig) -> Result<()> {
    let to = absolute_path(&Path::new(&module.location.to))?;
    let from = absolute_path(&Path::new(&module.location.from))?;

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

fn unlink_module(module: &ModuleConfig) -> Result<()> {
    info!("Unlinking {} module.", &module.name);
    symlink::unlink_folder(
        &PathBuf::from(&module.location.to),
        &PathBuf::from(&module.location.from),
        true,
    )?;

    Ok(())
}

// TODO: is the best way of handling this?
// should it be HashSet<String>?
fn get_matching_modules<'a>(
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

fn get_module_map<'a>(kdot_config: &'a PackageConfig) -> HashMap<String, &'a ModuleConfig> {
    let mut map: HashMap<String, &'a ModuleConfig> = HashMap::new();

    kdot_config.modules.iter().for_each(|module| {
        map.insert(module.name.clone(), module);
    });

    map
}

fn main() -> Result<()> {
    let args = Cli::from_args();

    let log_level = {
        if let Some(value) = args.verbosity {
            match value.as_str() {
                "verbose" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => {
                    bail!("Invalid \"verbose\" option.");
                }
            }
        } else {
            LevelFilter::Warn
        }
    };

    CombinedLogger::init(vec![
        TermLogger::new(log_level, Config::default(), TerminalMode::Mixed),
        // WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
    ])
    .unwrap();

    let kdot_config = config::load_package_config(&PathBuf::from("kdot.json"))?;
    let map = get_module_map(&kdot_config);

    match args.pattern {
        Command::Link {
            modules: modules_names,
        } => {
            for name in get_matching_modules(&kdot_config, &modules_names) {
                if let Some(module) = map.get(name) {
                    link_module(&module)?;
                } else {
                    bail!("Invalid module.");
                }
            }
        }
        Command::Unlink {
            modules: modules_names,
        } => {
            for name in get_matching_modules(&kdot_config, &modules_names) {
                if let Some(module) = map.get(name) {
                    unlink_module(&module)?;
                } else {
                    bail!("Invalid module.");
                }
            }
        }
        Command::Sync {
            modules: modules_names,
        } => {
            for name in get_matching_modules(&kdot_config, &modules_names) {
                if let Some(module) = map.get(name) {
                    // Try to unlink
                    unlink_module(*module)?;

                    // Relink
                    link_module(*module)?;

                    info!("Linked \"{}\" module.", module.name);
                } else {
                    bail!("Invalid module.");
                }
            }
        }
    }

    Ok(())
}
