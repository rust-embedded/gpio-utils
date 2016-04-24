// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

#[derive(Debug)]
pub struct GpioOptions {
    pub configs: Vec<String>,
}

#[derive(Debug)]
pub struct GpioReadOptions {
    pub gpio_opts: GpioOptions,
    pub pin: String,
}

#[derive(Debug)]
pub struct GpioWriteOptions {
    pub gpio_opts: GpioOptions,
    pub pin: String,
    pub value: u8,
}

#[derive(Debug)]
pub struct GpioPollOptions {
    pub gpio_opts: GpioOptions,
    pub timeout: Option<isize>,
    pub edge: String,
    pub pin: String,
}

#[derive(Debug)]
pub struct GpioExportOptions {
    pub gpio_opts: GpioOptions,
    pub symlink_root: Option<String>,
    pub pin: String,
}

#[derive(Debug)]
pub struct GpioExportAllOptions {
    pub gpio_opts: GpioOptions,
    pub symlink_root: Option<String>,
}

#[derive(Debug)]
pub struct GpioUnexportOptions {
    pub gpio_opts: GpioOptions,
    pub symlink_root: Option<String>,
    pub pin: String,
}

#[derive(Debug)]
pub struct GpioUnexportAllOptions {
    pub gpio_opts: GpioOptions,
    pub symlink_root: Option<String>,
}

#[derive(Debug)]
pub struct GpioStatusOptions {
    pub gpio_opts: GpioOptions,
    pub pin: Option<String>,
}
