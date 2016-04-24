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
pub struct GpioReadOptions<'a> {
    pub gpio_opts: GpioOptions,
    pub pin: &'a str,
}

#[derive(Debug)]
pub struct GpioWriteOptions<'a> {
    pub gpio_opts: GpioOptions,
    pub pin: &'a str,
    pub value: u8,
}

#[derive(Debug)]
pub struct GpioPollOptions<'a> {
    pub gpio_opts: GpioOptions,
    pub timeout: Option<isize>,
    pub edge: &'a str,
    pub pin: &'a str,
}

#[derive(Debug)]
pub struct GpioExportOptions<'a> {
    pub gpio_opts: GpioOptions,
    pub symlink_root: Option<&'a str>,
    pub pin: &'a str,
}

#[derive(Debug)]
pub struct GpioExportAllOptions<'a> {
    pub gpio_opts: GpioOptions,
    pub symlink_root: Option<&'a str>,
}

#[derive(Debug)]
pub struct GpioUnexportOptions<'a> {
    pub gpio_opts: GpioOptions,
    pub symlink_root: Option<&'a str>,
    pub pin: &'a str,
}

#[derive(Debug)]
pub struct GpioUnexportAllOptions<'a> {
    pub gpio_opts: GpioOptions,
    pub symlink_root: Option<&'a str>,
}

#[derive(Debug)]
pub struct GpioStatusOptions<'a> {
    pub gpio_opts: GpioOptions,
    pub pin: Option<&'a str>,
}
