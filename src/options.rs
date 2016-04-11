// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

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
}

#[derive(Debug)]
pub struct GpioPollOptions {
    pub gpio_opts: GpioOptions,
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
    pub pin: String,
}

#[derive(Debug)]
pub struct GpioUnexportAllOptions {
    pub gpio_opts: GpioOptions,
}

#[derive(Debug)]
pub struct GpioStatusOptions {
    pub gpio_opts: GpioOptions,
    pub pin: Option<String>,
}
