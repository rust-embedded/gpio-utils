// Copyright (C) 2016, The gpio-utils Authors

use options::GpioWriteOptions;
use config::GpioConfig;
use std::process::exit;
use sysfs_gpio::Direction;

pub fn main(config: &GpioConfig, opts: &GpioWriteOptions) {
    let pin_config = match config.get_pin(&opts.pin[..]) {
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
