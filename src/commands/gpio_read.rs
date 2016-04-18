// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

use options::{GpioReadOptions};
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
