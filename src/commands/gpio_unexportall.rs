// Copyright (C) 2016, The gpio-utils Authors

use options::GpioUnexportAllOptions;
use config::GpioConfig;
use std::process::exit;
use export;

pub fn main(config: &GpioConfig, opts: &GpioUnexportAllOptions) {
    let symlink_root = match opts.symlink_root {
        Some(ref slr) => &slr[..],
        None => config.get_symlink_root(),
    };

    for pin in config.get_pins().iter().filter(|p| p.export) {
        if let Err(e) = export::unexport(pin, Some(symlink_root)) {
            println!("Error occurred while exporting pin: {:?}", pin);
            println!("{}", e);
            exit(1);
        }
    }
}
