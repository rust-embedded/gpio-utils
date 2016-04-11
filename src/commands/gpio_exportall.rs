// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

use options::GpioExportAllOptions;
use config::GpioConfig;
use std::process::exit;
use export;

pub fn main(config: &GpioConfig, opts: &GpioExportAllOptions) {
    for pin in config.get_pins() {
        let symlink_root = opts.symlink_root
                               .clone()
                               .unwrap_or(String::from(config.get_symlink_root()));
        match export::export(pin, Some(&symlink_root[..])) {
            Ok(_) => {}
            Err(e) => {
                println!("Error occurred while exporting pin: {:?}", pin);
                println!("{}", e);
                exit(1);
            }
        }
    }
}
