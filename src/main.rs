extern crate clap;

use clap::{Arg, App, SubCommand};

fn main() {
    let gpio_cmd_matches = App::new("GPIO Utils")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Read, Write, and Configure GPIOs")

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
                    .about("Wait for an event to happen on an GPIO Input")
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
                    .about("Output status of all configured GPIOs")
                    .arg(Arg::with_name("pin")
                         .help("The pin name (or number)")
                         .index(1)
                         .required(false)))

        .get_matches();

    match gpio_cmd_matches.subcommand() {
        ("read", Some(m)) => {},
        ("poll", Some(m)) => {},
        ("write", Some(m)) => {},
        ("export", Some(m)) => {},
        ("export-all", Some(m)) => {},
        ("unexport", Some(m)) => {},
        ("unexport-all", Some(m)) => {},
        ("status", Some(m)) => {},
        _ => {}
    }
}
