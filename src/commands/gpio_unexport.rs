// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use crate::config::GpioConfig;
use crate::export;
use crate::options::GpioUnexportOptions;
use std::process::exit;

pub fn main(config: &GpioConfig, opts: &GpioUnexportOptions) {
    let pin_config = config.get_pin(opts.pin).unwrap_or_else(|| {
        println!("Unable to find config entry for pin '{}'", opts.pin);
        exit(1)
    });

    let symlink_root = match opts.symlink_root {
        Some(slr) => slr,
        None => config.get_symlink_root(),
    };

    if let Err(e) = export::unexport(pin_config, Some(symlink_root)) {
        println!("Error occurred while unexport pin {:?}", pin_config);
        println!("{}", e);
        exit(1);
    }
}
