#![allow(clippy::useless_format)]

#[macro_use]
extern crate log;
extern crate simplelog;

use anyhow::{bail, Context, Result};
use config::{ModuleConfig, PackageConfig};
use simplelog::*;
use std::{
    fs,
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
    Link {
        #[structopt(parse(from_os_str))]
        name: PathBuf,
    },
    /// Unlinks the module to the system.
    Unlink {
        #[structopt(parse(from_os_str))]
        name: PathBuf,
    },
    /// Unlinks and then relinks the module to the system.
    Sync {
        #[structopt(parse(from_os_str))]
        name: PathBuf,
    },
}

fn load_module_by_name(config: PackageConfig, name: &str) -> Option<ModuleConfig> {
    config
        .modules
        .into_iter()
        .find(|module| module.name == name)
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

    match args.pattern {
        Command::Link { name } => {
            let kdot_config =
                config::load_package_config(&fs::canonicalize(&PathBuf::from("./kdot.json"))?)?;

            let name = name.as_os_str().to_str().unwrap();

            for module in kdot_config
                .modules
                .into_iter()
                .filter(|module| module.name == name)
            {
                link_module(&module)?;
            }
        }
        Command::Unlink { name } => {
            let kdot_config = config::load_package_config(&PathBuf::from("kdot.json"))?;

            if let Some(module) = load_module_by_name(kdot_config, &name.to_str().unwrap()) {
                unlink_module(&module)?;
            } else {
                bail!("Invalid module.");
            }
        }
        Command::Sync { name } => {
            let kdot_config = config::load_package_config(&PathBuf::from("kdot.json"))?;

            for module in kdot_config
                .modules
                .into_iter()
                .filter(|module| module.name == name.as_os_str().to_str().unwrap())
            {
                // Try to unlink
                unlink_module(&module)?;

                // Relink
                link_module(&module)?;

                info!("Linked \"{}\" module.", module.name);
            }
        }
    }

    Ok(())
}
