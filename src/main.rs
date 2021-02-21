#![allow(clippy::useless_format)]

#[macro_use]
extern crate log;
extern crate simplelog;

use anyhow::{bail, Result};
use simplelog::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
mod module;
mod path;
mod symlink;

use module::*;

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
