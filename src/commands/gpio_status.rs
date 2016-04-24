// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use options::GpioStatusOptions;
use config::GpioConfig;
use config::PinConfig;
use sysfs_gpio::Direction;
use std::process::exit;

pub fn main(config: &GpioConfig, opts: &GpioStatusOptions) {
    match opts.pin {
        Some(ref pin_name) => {
            let pin_config = match config.get_pin(pin_name) {
                Some(pin) => pin,
                None => {
                    println!("Unable to find config entry for pin '{}'", pin_name);
                    exit(1)
                }
            };
            print_pin_header();
            print_pin_row(&pin_config, true);
        }
        None => {
            print_pin_header();
            for (pos, pin) in config.get_pins().iter().enumerate() {
                print_pin_row(pin, pos == (config.get_pins().len() - 1));
            }
        }
    }
}


fn print_pin_header() {
    println!("| {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} |",
             "Number",
             "Exported",
             "Direction",
             "Active Low",
             "Names",
             "Value");
    print_row_sep(false);
}

fn print_row_sep(is_last: bool) {
    let col_sep = if is_last {
        "-"
    } else {
        "+"
    };
    println!("{}{:->13}{:->13}{:->13}{:->13}{:->13}{:->13}",
             col_sep,
             col_sep,
             col_sep,
             col_sep,
             col_sep,
             col_sep,
             col_sep);
}

fn print_pin_row(pin_config: &PinConfig, is_last: bool) {
    let direction = match pin_config.direction {
        Direction::In => "In",
        Direction::Out => "Out",
        Direction::High => "High",
        Direction::Low => "Low",
    };

    let value = match pin_config.get_pin().get_value() {
        Ok(value) => value,
        Err(e) => {
            println!("ERROR: {:?}", e);
            exit(1);
        }
    };

    for (pos, name) in pin_config.names.iter().enumerate() {
        if pos == 0 {
            println!("| {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} |",
                     pin_config.num,
                     pin_config.export,
                     direction,
                     pin_config.active_low,
                     name,
                     value);
        } else {
            println!("| {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} |",
                     "",
                     "",
                     "",
                     "",
                     name,
                     "");

        }
    }
    print_row_sep(is_last);
}
