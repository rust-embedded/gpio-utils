// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

extern crate gpio_utils;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;


use clap::{Arg, App, SubCommand, AppSettings};
use gpio_utils::options::*;
use gpio_utils::commands::*;

fn main() {
    env_logger::init().unwrap();

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
                         .required(true)))

        // gpio write
        .subcommand(SubCommand::with_name("write")
                    .about("Write the value of a GPIO Output")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true)))

        // gpio export
        .subcommand(SubCommand::with_name("export")
                    .about("Export a given GPIO")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true)))

        // gpio export-all
        .subcommand(SubCommand::with_name("export-all")
                    .about("Export all configured GPIOs"))

        // gpio unexport
        .subcommand(SubCommand::with_name("unexport")
                    .about("Unexport a given GPIO")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true)))

        // gpio unexport-all
        .subcommand(SubCommand::with_name("unexport-all")
                    .about("Unexport all configured, exported GPIOs")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(true)))

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
        configs: matches.values_of_lossy("config").unwrap_or(Vec::new()),
    };

    match matches.subcommand() {
        ("read", Some(m)) => {
            let read_options = GpioReadOptions {
                gpio_opts: gpio_options,
                pin: String::from(m.value_of("pin").unwrap()),
            };
            gpio_read::main(&read_options);
        },
        ("poll", Some(_)) => {},
        ("write", Some(_)) => {},
        ("export", Some(_)) => {},
        ("export-all", Some(_)) => {
            let exportall_options = GpioExportAllOptions {
                gpio_opts: gpio_options,
            };
            gpio_exportall::main(&exportall_options);
        },
        ("unexport", Some(_)) => {},
        ("unexport-all", Some(_)) => {},
        ("status", Some(_)) => {},
        _ => {}
    }
}
