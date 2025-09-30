// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use crate::config::GpioConfig;
use crate::options::GpioWriteOptions;
use std::process::exit;
use sysfs_gpio::Direction;

pub fn main(config: &GpioConfig, opts: &GpioWriteOptions) {
    let pin_config = match config.get_pin(opts.pin) {
        Some(pin) => pin,
        None => {
            println!("Unable to find config entry for pin '{}'", opts.pin);
            exit(1);
        }
    };

    let pin = pin_config.get_pin();
    pin.set_direction(Direction::Out).unwrap_or_else(|e| {
        println!("Error setting GPIO direction: {:?}", e);
        exit(1)
    });
    pin.set_value(opts.value).unwrap_or_else(|e| {
        println!("There was an error writing to the gpio: {:?}", e);
        exit(1);
    });
}
