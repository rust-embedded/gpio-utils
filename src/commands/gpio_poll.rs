// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use options::GpioPollOptions;
use config::GpioConfig;
use sysfs_gpio::Edge;
use std::process::exit;

pub fn main(config: &GpioConfig, opts: &GpioPollOptions) {
    let timeout = opts.timeout.unwrap_or(-1);
    let pin_config = match config.get_pin(&opts.pin[..]) {
        Some(pin) => pin,
        None => {
            println!("Unable to find config entry for pin '{}'", opts.pin);
            exit(1)
        }
    };
    let pin = pin_config.get_pin();
    let edge = match &opts.edge[..] {
        "rising" => Edge::RisingEdge,
        "falling" => Edge::FallingEdge,
        "both" => Edge::BothEdges,
        other => {
            println!("Unexpected edge value: {}", other);
            exit(1);
        }
    };

    // set the pin direction
    pin.set_edge(edge).unwrap_or_else(|e| {
        println!("Error setting edge on pin: {:?}", e);
        exit(1);
    });

    let mut poller = pin.get_poller().unwrap_or_else(|e| {
        println!("Error creating pin poller: {:?}", e);
        exit(1);
    });
    match poller.poll(timeout) {
        Ok(Some(value)) => {
            println!("{}", value);
            exit(0);
        }
        Ok(None) => {
            println!("TIMEOUT");
            exit(2)
        }
        Err(e) => {
            println!("Error on Poll: {:?}", e);
            exit(1);
        }
    }
}
