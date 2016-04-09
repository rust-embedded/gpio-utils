// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

use options::{GpioExportAllOptions};
use config::GpioConfig;
use std::process::exit;

pub fn main(opts: &GpioExportAllOptions) {
    let config = match GpioConfig::load(&opts.gpio_opts.configs[..]) {
        Ok(config) => config,
        Err(e) => {
            println!("Error: {:?}", e);
            exit(1);
        }
    };

    for pin in config.pins {
        println!("{:?}", pin);
    }
}
