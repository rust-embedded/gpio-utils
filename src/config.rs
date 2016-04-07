// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

use toml;
use rustc_serialize::{Decodable};
use std::io;
use std::path::Path;
use std::collections::HashMap;
use sysfs_gpio::Direction;

#[derive(RustcDecodable, Debug)]
pub struct PinConfig {
    num: u64,
    direction: Option<String>,
    aliases: Option<Vec<String>>,
    export: Option<bool>,
    active_low: Option<bool>,
}

#[derive(RustcDecodable, Debug)]
pub struct GpioConfig {
    pins: HashMap<String, PinConfig>,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParseError,
    NoConfigFound,
}

fn to_direction(dirstr: &str) -> Option<Direction> {
    match dirstr {
        "in" => Some(Direction::In),
        "out" => Some(Direction::Out),
        "high" => Some(Direction::High),
        "low" => Some(Direction::Low),
        _ => None
    }
}

impl GpioConfig {

    /// Load a GPIO Config from the system
    ///
    /// This function will load the GPIO configuration from standard system
    /// locations as well as from the additional configs passed in via the
    /// `configs` parameter.  Each parameter is expected to be a path to a
    /// config file in disk.
    ///
    /// Under the covers, this function will attempt to discover configuration
    /// files in the following standard locations in order:
    ///
    /// - `/etc/gpio.toml`
    /// - `/etc/gpio.d/*.toml`
    /// - `configs` (parameter)
    ///
    /// Each config file found in these locations will be loaded and then they
    /// will be pulled together to form a unified configuration via the
    /// `combine` method.
    pub fn load(configs: Vec<String>) -> Result<GpioConfig, Error> {
        Err(Error::NoConfigFound)
    }

    /// Load a GPIO configuration for the provided toml string
    pub fn from_str(config: &str) -> Result<GpioConfig, Error> {
        let root = toml::Parser::new(config).parse().unwrap();
        let mut d = toml::Decoder::new(toml::Value::Table(root));
        Ok(Decodable::decode(&mut d).unwrap())
    }

    /// Load a GPIO config from the specified path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<GpioConfig, Error> {
        Err(Error::NoConfigFound)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let configstr = r#"
# Basic Form
[pins.reset_button]
num = 73           # required
direction = "in"   # default: in
active_low = true  # default: false
export = true      # default: true

[pins.status_led]
num = 37
aliases = ["A27", "green_led"]
direction = "out"

# Compact Form
[pins]
error_led = { num = 11, direction = "in", export = false}
"#;
        let config = GpioConfig::from_str(configstr).unwrap();
        let status_led = config.pins.get("status_led").unwrap();
        assert_eq!(status_led.num, 37);
        assert_eq!(status_led.aliases,
                   Some(vec!(String::from("A27"),
                             String::from("green_led"))));
        assert_eq!(status_led.direction, Some(String::from("out")));
        assert_eq!(status_led.active_low, None);
        assert_eq!(status_led.export, None);
    }
}
