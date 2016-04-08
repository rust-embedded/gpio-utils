// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

use glob::glob;
use rustc_serialize::{Decodable};
use std::collections::{HashMap, BTreeSet};
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use sysfs_gpio::Direction;
use toml;

#[derive(RustcDecodable, Clone, Debug)]
pub struct PinConfig {
    num: u64,
    direction: Option<String>,
    aliases: Option<BTreeSet<String>>,
    export: Option<bool>,
    active_low: Option<bool>,
}

#[derive(RustcDecodable, Clone, Debug)]
pub struct GpioConfig {
    pins: HashMap<String, PinConfig>,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParserErrors(Vec<toml::ParserError>),
    DecodingError(toml::DecodeError),
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

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<Vec<toml::ParserError>> for Error {
    fn from(e: Vec<toml::ParserError>) -> Self {
        Error::ParserErrors(e)
    }
}

impl From<toml::DecodeError> for Error {
    fn from(e: toml::DecodeError) -> Self {
        Error::DecodingError(e)
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
    pub fn load(configs: &[&str]) -> Result<GpioConfig, Error> {
        let mut config_instances: Vec<GpioConfig> = Vec::new();

        // check /etc/gpio.toml
        if fs::metadata("/etc/gpio.toml").is_ok() {
            config_instances.push(try!(Self::from_file("/etc/gpio.toml")));
        }

        // /etc/gpio.d/*.toml
        for fragment in glob("/etc/gpio.d/*.toml").unwrap().filter_map(Result::ok) {
            config_instances.push(try!(Self::from_file(fragment)));
        }

        // additional from command-line
        for fragment in configs {
            config_instances.push(try!(Self::from_file(fragment)));
        }

        if config_instances.len() == 0 {
            Err(Error::NoConfigFound)
        } else {
            Ok(config_instances[1..].iter().fold(config_instances[0].clone(), |a, b| {
                a.merge(b)
            }))
        }
    }

    /// Load a GPIO configuration for the provided toml string
    pub fn from_str(config: &str) -> Result<GpioConfig, Error> {
        let mut parser = toml::Parser::new(config);
        let root = try!(parser.parse().ok_or(parser.errors));
        let mut d = toml::Decoder::new(toml::Value::Table(root));
        match Decodable::decode(&mut d) {
            Ok(cfg) => Ok(cfg),
            Err(e) => Err(Error::from(e)),
        }
    }

    /// Load a GPIO config from the specified path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<GpioConfig, Error> {
        let mut contents = String::new();
        let mut f = try!(File::open(path));
        try!(f.read_to_string(&mut contents));
        GpioConfig::from_str(&contents[..])
    }

    /// Merge this config with other yielding a new, merged version
    ///
    /// If in conflict, the other GPIO config takes priority.
    pub fn merge(&self, other: &GpioConfig) -> GpioConfig {
        // TODO: This needs to actually resolve conflicts rather than
        //   blindly writing over as it does now.
        let mut pins = HashMap::new();
        pins.extend(self.pins.clone());
        pins.extend(other.pins.clone());
        GpioConfig {
            pins: pins
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;
    use std::collections::BTreeSet;

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
        let mut aliases = BTreeSet::from_iter(vec!(String::from("A27"), String::from("green_led")));
        assert_eq!(status_led.num, 37);
        assert_eq!(status_led.aliases, Some(aliases));
        assert_eq!(status_led.direction, Some(String::from("out")));
        assert_eq!(status_led.active_low, None);
        assert_eq!(status_led.export, None);
    }

    #[test]
    fn test_parser_empty_toml() {
        let configstr = "";
        match GpioConfig::from_str(configstr) {
            Err(Error::DecodingError(_)) => {},
            _ => panic!("Expected a decoding error"),
        }
    }

    #[test]
    fn test_parser_missing_pinnum() {
        let configstr = r#"
[pins.reset_button]
export = true
"#;
        match GpioConfig::from_str(configstr) {
            Err(Error::DecodingError(_)) => {},
            _ => panic!("Expected a decoding error"),
        }
    }

    #[test]
    fn test_parse_error_bad_toml() {
        // basically, just garbage data
        let configstr = r#"
[] -*-..asdf=-=-@#$%^&*()
"#;
        match GpioConfig::from_str(configstr) {
            Err(Error::ParserErrors(e)) => {},
            _ => panic!("Did not receive parse error when expected"),
        }
    }
}
