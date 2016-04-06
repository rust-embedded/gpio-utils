extern crate clap;

use clap::{Arg, App, SubCommand};

fn main() {
    let gpio_cmd_matches = App::new("GPIO Utils")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Read, Write, and Configure GPIOs")
        .subcommand(SubCommand::with_name("read")
                    .about("Read the value of a GPIO Input"))
        .subcommand(SubCommand::with_name("poll")
                    .about("Wait for an event to happen on an GPIO Input"))
        .subcommand(SubCommand::with_name("write")
                    .about("Write the value of a GPIO Output"))
        .subcommand(SubCommand::with_name("export")
                    .about("Export a given GPIO"))
        .subcommand(SubCommand::with_name("export-all")
                    .about("Export all configured GPIOs"))
        .subcommand(SubCommand::with_name("unexport")
                    .about("Unexport a given GPIO"))
        .subcommand(SubCommand::with_name("unexport-all")
                    .about("Unexport all configured, exported GPIOs"))
        .subcommand(SubCommand::with_name("status")
                    .about("Output status of all configured GPIOs"))
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
