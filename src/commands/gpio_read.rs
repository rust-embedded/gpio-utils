// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

use options::{GpioReadOptions};
use config::GpioConfig;
use std::process::exit;

pub fn main(opts: &GpioReadOptions) {
    let config = match GpioConfig::load(&opts.gpio_opts.configs[..]) {
        Ok(config) => config,
        Err(e) => {
            println!("Error: {:?}", e);
            exit(1);
        }
    };

    println!("config: {:?}", config);
}
