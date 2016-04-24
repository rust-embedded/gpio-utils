// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use options::GpioReadOptions;
use config::GpioConfig;
use std::process::exit;

pub fn main(config: &GpioConfig, opts: &GpioReadOptions) {
    let pin_config = match config.get_pin(&opts.pin[..]) {
        Some(pin) => pin,
        None => {
            println!("Unable to find config entry for pin '{}'", opts.pin);
            exit(1)
        }
    };

    let pin = pin_config.get_pin();
    match pin.get_value() {
        Ok(value) => println!("{}", value),
        Err(e) => println!("ERROR: {:?}", e),
    }
}
