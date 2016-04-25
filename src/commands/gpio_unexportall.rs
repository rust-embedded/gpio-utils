// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use options::GpioUnexportAllOptions;
use config::GpioConfig;
use std::process::exit;
use export;

pub fn main(config: &GpioConfig, opts: &GpioUnexportAllOptions) {
    let symlink_root = match opts.symlink_root {
        Some(slr) => slr,
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
