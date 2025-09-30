// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use clap::{Parser, Subcommand};
use gpio_utils::commands::*;
use gpio_utils::config::{self, GpioConfig};
use gpio_utils::options::*;
use std::process;

#[derive(Debug, Parser)]
#[command(
    name = "GPIO Utils",
    version,
    about = "Read, Write, and Configure GPIOs"
)]
struct Cli {
    /// additional configuration to use
    #[arg(short, long = "config", value_name = "FILE")]
    configs: Vec<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Read the value of a GPIO Input
    Read {
        /// The pin name (or number)
        pin: String,
    },
    /// Wait for an event to happen on a GPIO Input
    Poll {
        /// The pin name (or number)
        pin: String,
        /// Timeout (in ms) for the poll operation (-1 to wait forever, default)
        #[arg(short, long)]
        timeout: Option<isize>,
        /// The edge to poll on
        #[arg(short, long)]
        edge: Option<String>,
    },
    /// Write the value of a GPIO Output
    Write {
        /// The pin name (or number)
        pin: String,
        /// Value to write to pin (0|1)
        value: u8,
    },
    /// Export a given GPIO
    Export {
        /// The pin name (or number)
        pin: String,
        /// root directory for export symlinks
        #[arg(short = 'r', long)]
        symlink_root: Option<String>,
    },
    /// Export all configured GPIOs
    ExportAll {
        /// Export all configured GPIOs
        #[arg(short = 'r', long)]
        symlink_root: Option<String>,
    },
    /// Export all configured GPIOs
    Unexport {
        /// The pin name (or number)
        pin: String,
        /// root directory for export symlinks
        #[arg(short = 'r', long)]
        symlink_root: Option<String>,
    },
    /// Unexport all configured, exported GPIOs
    UnexportAll {
        /// root directory for export symlinks
        #[arg(short = 'r', long)]
        symlink_root: Option<String>,
    },
    /// Output status of a GPIO or all GPIOs if no pin is specified
    Status {
        /// The pin name (or number)
        pin: Option<String>,
    },
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let gpio_opts = GpioOptions {
        configs: cli.configs.clone(),
    };

    // parse the config
    let cfg = match GpioConfig::load(&gpio_opts.configs[..]) {
        Ok(cfg) => cfg,
        Err(config::Error::NoConfigFound) => Default::default(),
        Err(e) => {
            println!("Error parsing config.  Details follow...");
            println!("{}", e);
            process::exit(1);
        }
    };

    // TODO: Why are we passing the gpio_options and the config parsed from it to `gpio_read::main`
    // and the other handlers?
    match cli.command {
        Commands::Read { pin } => {
            let options = GpioReadOptions {
                gpio_opts,
                pin: &pin,
            };
            gpio_read::main(&cfg, &options);
        }
        Commands::Poll { pin, timeout, edge } => {
            let options = GpioPollOptions {
                gpio_opts,
                timeout,
                edge: &edge.unwrap_or_else(|| String::from("both")),
                pin: &pin,
            };
            gpio_poll::main(&cfg, &options);
        }
        Commands::Write { pin, value } => {
            let options = GpioWriteOptions {
                gpio_opts,
                pin: &pin,
                value,
            };
            gpio_write::main(&cfg, &options);
        }
        Commands::Export { pin, symlink_root } => {
            let options = GpioExportOptions {
                gpio_opts,
                pin: &pin,
                symlink_root: symlink_root.as_deref(),
            };
            gpio_export::main(&cfg, &options);
        }
        Commands::ExportAll { symlink_root } => {
            let options = GpioExportAllOptions {
                gpio_opts,
                symlink_root: symlink_root.as_deref(),
            };
            gpio_exportall::main(&cfg, &options);
        }
        Commands::Unexport { pin, symlink_root } => {
            let options = GpioUnexportOptions {
                gpio_opts,
                pin: &pin,
                symlink_root: symlink_root.as_deref(),
            };
            gpio_unexport::main(&cfg, &options);
        }
        Commands::UnexportAll { symlink_root } => {
            let options = GpioUnexportAllOptions {
                gpio_opts,
                symlink_root: symlink_root.as_deref(),
            };
            gpio_unexportall::main(&cfg, &options);
        }
        Commands::Status { pin } => {
            let options = GpioStatusOptions {
                gpio_opts,
                pin: pin.as_deref(),
            };
            gpio_status::main(&cfg, &options);
        }
    }
}
