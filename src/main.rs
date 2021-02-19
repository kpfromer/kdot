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
                let to = absolute_path(&Path::new(&module.location.to))?;
                let from = absolute_path(&Path::new(&module.location.from))?;

                info!(
                    "Linking \"{}\" to \"{}\"",
                    to.as_os_str().to_str().unwrap(),
                    from.as_os_str().to_str().unwrap(),
                );

                // TODO: better error message
                symlink::link_folder(&to, &from, true)
                    .with_context(|| format!("Failed to link module."))?;

                info!("Linked \"{}\" module.", module.name);
            }
        }
        Command::Unlink { name } => {
            let kdot_config = config::load_package_config(&PathBuf::from("kdot.json"))?;

            if let Some(module) = load_module_by_name(kdot_config, &name.to_str().unwrap()) {
                info!("Unlinking {} module.", &module.name);
                symlink::unlink_folder(
                    &PathBuf::from(module.location.to),
                    &PathBuf::from(module.location.from),
                    true,
                )?;
            } else {
                bail!("Invalid module.");
            }
        }
        Command::Sync { name: _ } => {

            // Check if the module exists, if so unlink it

            // Then relink

            // let kdot_config = config::load_package_config(&PathBuf::from("kdot.json"))?;

            // // TODO: dedupe this code with the above code

            // // Try to unlink
            // if let Some(module) = load_module_by_name(&kdot_config, &name.to_str().unwrap()) {
            //     symlink::unlink_folder(
            //         &PathBuf::from(module.location.to),
            //         &PathBuf::from(module.location.from),
            //         false,
            //     )?;
            // } else {
            //     bail!("Invalid module.");
            // }

            // // Relink

            // for module in kdot_config
            //     .modules
            //     .into_iter()
            //     .filter(|module| module.name == name.as_os_str().to_str().unwrap())
            // {
            //     // TODO: Fix
            //     // TODO: better error message
            //     symlink::link_folder(
            //         &PathBuf::from(module.location.to),
            //         &PathBuf::from(module.location.from),
            //         false,
            //     )
            //     .with_context(|| format!("Failed to link module."))?;

            //     println!("Linked \"{}\" module.", module.name);
            // }
        }
    }

    Ok(())
}
