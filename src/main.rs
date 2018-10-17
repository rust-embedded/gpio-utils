// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.
extern crate gpio_utils;
extern crate clap;
extern crate env_logger;
extern crate log;

use clap::{Arg, App, SubCommand, AppSettings};
use gpio_utils::options::*;
use gpio_utils::commands::*;
use gpio_utils::config::{self, GpioConfig};
use std::process::exit;

fn main() {
    env_logger::init();

    let matches = App::new("GPIO Utils")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Read, Write, and Configure GPIOs")
        .setting(AppSettings::SubcommandRequired)

        // Global options
        .arg(Arg::with_name("config")
             .help("additional configuration to use")
             .takes_value(true)
             .short("c")
             .long("config")
             .multiple(true)
             .required(false))

        // gpio read
        .subcommand(SubCommand::with_name("read")
                    .about("Read the value of a GPIO Input")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true)))

        // gpio poll
        .subcommand(SubCommand::with_name("poll")
                    .about("Wait for an event to happen on a GPIO Input")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true))
                    .arg(Arg::with_name("timeout")
                         .help("Timeout (in ms) for the poll operation (-1 to wait forever, default)")
                         .takes_value(true)
                         .short("t")
                         .long("timeout")
                         .required(false))
                    .arg(Arg::with_name("edge")
                         .help("The edge to poll on")
                         .takes_value(true)
                         .short("e")
                         .long("edge")
                         .required(false)))

        // gpio write
        .subcommand(SubCommand::with_name("write")
                    .about("Write the value of a GPIO Output")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true))
                    .arg(Arg::with_name("value")
                         .help("Value to write to pin (0|1)")
                         .index(2)
                         .required(true)))

        // gpio export
        .subcommand(SubCommand::with_name("export")
                    .about("Export a given GPIO")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true))
                    .arg(Arg::with_name("symlink-root")
                         .help("root directory for export symlinks")
                         .takes_value(true)
                         .short("r")
                         .long("symlink-root")
                         .required(false)))

        // gpio export-all
        .subcommand(SubCommand::with_name("export-all")
                    .about("Export all configured GPIOs")
                    .arg(Arg::with_name("symlink-root")
                         .help("root directory for export symlinks")
                         .takes_value(true)
                         .short("r")
                         .long("symlink-root")
                         .required(false)))

        // gpio unexport
        .subcommand(SubCommand::with_name("unexport")
                    .about("Unexport a given GPIO")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true))
                    .arg(Arg::with_name("symlink-root")
                         .help("root directory for export symlinks")
                         .takes_value(true)
                         .short("r")
                         .long("symlink-root")
                         .required(false)))

        // gpio unexport-all
        .subcommand(SubCommand::with_name("unexport-all")
                    .about("Unexport all configured, exported GPIOs")
                    .arg(Arg::with_name("symlink-root")
                         .help("root directory for export symlinks")
                         .takes_value(true)
                         .short("r")
                         .long("symlink-root")
                         .required(false)))

        // gpio status
        .subcommand(SubCommand::with_name("status")
                    .about("Output status of a GPIO or all GPIOs if no pin is specified")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(false)))

        .get_matches();

    // process global options
    let gpio_options = GpioOptions {
        configs: matches.values_of_lossy("config").unwrap_or_default(),
    };

    // parse the config
    let cfg = match GpioConfig::load(&gpio_options.configs[..]) {
        Ok(cfg) => cfg,
        Err(config::Error::NoConfigFound) => Default::default(),
        Err(e) => {
            println!("Error parsing config.  Details follow...");
            println!("{}", e);
            std::process::exit(1);
        }
    };

    match matches.subcommand() {
        ("read", Some(m)) => {
            let read_options = GpioReadOptions {
                gpio_opts: gpio_options,
                pin: m.value_of("pin").unwrap(),
            };
            gpio_read::main(&cfg, &read_options);
        }
        ("poll", Some(m)) => {
            let timeout = m.value_of("timeout").map(|timeout| {
                timeout.parse::<isize>().unwrap_or_else(|_| {
                    println!("Unable to parse timeout value {:?} as integer", timeout);
                    exit(1);
                })
            });
            let poll_options = GpioPollOptions {
                gpio_opts: gpio_options,
                edge: m.value_of("edge").unwrap_or("both"),
                timeout: timeout,
                pin: m.value_of("pin").unwrap(),
            };
            gpio_poll::main(&cfg, &poll_options);
        }
        ("write", Some(m)) => {
            let write_options = GpioWriteOptions {
                gpio_opts: gpio_options,
                pin: m.value_of("pin").unwrap(),
                value: match m.value_of("value").unwrap().parse::<u8>() {
                    Ok(value) => value,
                    Err(_) => {
                        println!("Provided value {:?} is not valid",
                                 m.value_of("value").unwrap());
                        exit(1);
                    }
                },
            };
            gpio_write::main(&cfg, &write_options);
        }
        ("export", Some(m)) => {
            let export_options = GpioExportOptions {
                gpio_opts: gpio_options,
                pin: m.value_of("pin").unwrap(),
                symlink_root: match m.value_of("symlink-root") {
                    Some(slr) => Some(slr),
                    None => None,
                },
            };
            gpio_export::main(&cfg, &export_options);
        }
        ("export-all", Some(m)) => {
            let exportall_options = GpioExportAllOptions {
                gpio_opts: gpio_options,
                symlink_root: match m.value_of("symlink-root") {
                    Some(slr) => Some(slr),
                    None => None,
                },
            };
            gpio_exportall::main(&cfg, &exportall_options);
        }
        ("unexport", Some(m)) => {
            let unexport_options = GpioUnexportOptions {
                gpio_opts: gpio_options,
                pin: m.value_of("pin").unwrap(),
                symlink_root: m.value_of("symlink-root"),
            };
            gpio_unexport::main(&cfg, &unexport_options);
        }
        ("unexport-all", Some(m)) => {
            let unexportall_options = GpioUnexportAllOptions {
                gpio_opts: gpio_options,
                symlink_root: m.value_of("symlink-root"),
            };
            gpio_unexportall::main(&cfg, &unexportall_options);
        }
        ("status", Some(m)) => {
            let status_options = GpioStatusOptions {
                gpio_opts: gpio_options,
                pin: m.value_of("pin"),
            };
            gpio_status::main(&cfg, &status_options);
        }
        _ => {}
    }
}
