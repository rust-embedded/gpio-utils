// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

use glob::glob;
use rustc_serialize::{Decodable, Decoder};
use std::collections::{BTreeSet, HashMap};
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use sysfs_gpio;
use toml;

#[derive(Debug, PartialEq, Clone)]
pub struct Direction(pub sysfs_gpio::Direction);

impl From<sysfs_gpio::Direction> for Direction {
    fn from(e: sysfs_gpio::Direction) -> Self {
        Direction(e)
    }
}

#[derive(RustcDecodable, Clone, Debug, PartialEq)]
struct RawPinConfig {
    pub num: u64,
    pub direction: Option<Direction>,
    pub names: BTreeSet<String>,
    pub export: Option<bool>,
    pub active_low: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PinConfig {
    pub num: u64,
    pub direction: sysfs_gpio::Direction,
    pub names: BTreeSet<String>,
    pub export: bool,
    pub active_low: bool,
}

impl Into<PinConfig> for RawPinConfig {
    fn into(self) -> PinConfig {
        let default_direction = Direction(sysfs_gpio::Direction::In);
        PinConfig {
            num: self.num,
            direction: self.direction.unwrap_or(default_direction).0,
            names: self.names,
            export: self.export.unwrap_or(true),
            active_low: self.active_low.unwrap_or(false),
        }
    }
}

impl Decodable for Direction {
    fn decode<D: Decoder>(d: &mut D) -> Result<Direction, D::Error> {
        match &try!(d.read_str())[..] {
            "in" => Ok(Direction(sysfs_gpio::Direction::In)),
            "out" => Ok(Direction(sysfs_gpio::Direction::Out)),
            "high" => Ok(Direction(sysfs_gpio::Direction::High)),
            "low" => Ok(Direction(sysfs_gpio::Direction::Low)),
            _ => Err(d.error("Expected one of: {in, out, high, low}")),
        }
    }
}

#[derive(RustcDecodable, Clone, Debug)]
struct RawGpioConfig {
    pub pins: Vec<RawPinConfig>,
}

#[derive(Clone, Debug)]
pub struct GpioConfig {
    pub pins: Vec<PinConfig>,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParserErrors(Vec<toml::ParserError>),
    DecodingError(toml::DecodeError),
    NoConfigFound,
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

impl Into<GpioConfig> for RawGpioConfig {
    fn into(self) -> GpioConfig {
        GpioConfig {
            pins: self.pins.into_iter().map(|p| p.into()).collect(),
        }
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
    pub fn load(configs: &[String]) -> Result<GpioConfig, Error> {
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
            let mut cfg = config_instances.remove(0);
            for higher_priority_cfg in config_instances {
                cfg.update(higher_priority_cfg);
            }
            Ok(cfg)
        }
    }

    /// Load a GPIO configuration for the provided toml string
    pub fn from_str(config: &str) -> Result<GpioConfig, Error> {
        let mut parser = toml::Parser::new(config);
        let root = try!(parser.parse().ok_or(parser.errors));
        let mut d = toml::Decoder::new(toml::Value::Table(root));
        let rawcfg: RawGpioConfig = try!(match Decodable::decode(&mut d) {
            Ok(rawcfg) =>  Ok(rawcfg),
            Err(e) => Err(Error::from(e)),
        });

        Ok(rawcfg.into())
    }

    /// Load a GPIO config from the specified path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<GpioConfig, Error> {
        let mut contents = String::new();
        let mut f = try!(File::open(path));
        try!(f.read_to_string(&mut contents));
        GpioConfig::from_str(&contents[..])
    }

    /// Merge other into self (takes ownership of other)
    ///
    /// If in conflict, the other GPIO config takes priority.
    pub fn update(&mut self, other: GpioConfig) {
        for other_pin in other.pins {
            // determine the case we are dealing with
            let existing = match self.pins.iter_mut().find(|p| p.num == other_pin.num) {
                Some(pin) => {
                    pin.names.extend(other_pin.names.clone());
                    pin.direction = other_pin.direction.clone(); // TODO impl copy
                    pin.export = other_pin.export;
                    pin.active_low = other_pin.active_low;
                    true
                },
                None => false,
            };

            if !existing {
                self.pins.push(other_pin);
            }

        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;
    use std::collections::BTreeSet;
    use sysfs_gpio::Direction as D;

    static BASIC_CFG: &'static str = r#"
# Basic Form
[[pins]]
num = 73
names = ["reset_button"]
direction = "in"   # default: in
active_low = true  # default: false
export = true      # default: true

[[pins]]
num = 37
names = ["status_led", "A27", "green_led"]
direction = "out"
"#;

    static COMPACT_CFG: &'static str = r#"
pins = [
   { num = 73, names = ["reset_button"], direction = "in", active_low = true, export = true},
   { num = 37, names = ["status_led", "A27", "green_led"], direction = "out"},
]
"#;

    static MISSING_PINNUM_CFG: &'static str = r#"
[[pins]]
export = true
"#;

    static PARTIALLY_OVERLAPS_BASIC_CFG: &'static str = r#"
# Add a new alias to pin 73
[[pins]]
num = 73
names = ["new_name"]


# Change pin 37 to be an input (not output)
[[pins]]
num = 37
direction = "in"

# New pin 88
[[pins]]
num = 88
names = ["wildcard"]
"#;

    #[test]
    fn test_parse_basic() {
        let config = GpioConfig::from_str(BASIC_CFG).unwrap();
        let status_led = config.pins.get(1).unwrap();
        let names = BTreeSet::from_iter(
            vec!(String::from("status_led"),
                 String::from("A27"),
                 String::from("green_led")));

        let reset_button = config.pins.get(0).unwrap();
        assert_eq!(reset_button.num, 73);
        assert_eq!(reset_button.names,
                   BTreeSet::from_iter(vec!(String::from("reset_button"))));
        assert_eq!(reset_button.direction, D::In);
        assert_eq!(reset_button.active_low, true);
        assert_eq!(reset_button.export, true);

        assert_eq!(status_led.names, names);
        assert_eq!(status_led.direction, D::Out);
        assert_eq!(status_led.active_low, false);
        assert_eq!(status_led.export, true);
    }

    #[test]
    fn test_parser_compact() {
        let config = GpioConfig::from_str(COMPACT_CFG).unwrap();
        let status_led = config.pins.get(1).unwrap();
        let names = BTreeSet::from_iter(
            vec!(String::from("status_led"),
                 String::from("A27"),
                 String::from("green_led")));
        assert_eq!(status_led.names, names);
        assert_eq!(status_led.direction, D::Out);
        assert_eq!(status_led.active_low, false);
        assert_eq!(status_led.export, true);
    }

    #[test]
    fn test_parser_empty_toml() {
        let configstr = "";
        match GpioConfig::from_str(configstr) {
            Ok(pins) => { assert_eq!(pins.pins, vec!()) },
            _ => panic!("Expected a decoding error"),
        }
    }

    #[test]
    fn test_parser_missing_pinnum() {
        match GpioConfig::from_str(MISSING_PINNUM_CFG) {
            Err(Error::DecodingError(_)) => {},
            _ => panic!("Expected a decoding error"),
        }
    }

    #[test]
    fn test_parse_error_bad_toml() {
        // basically, just garbage data
        let configstr = r"[] -*-..asdf=-=-@#$%^&*()";
        match GpioConfig::from_str(configstr) {
            Err(Error::ParserErrors(e)) => {},
            _ => panic!("Did not receive parse error when expected"),
        }
    }

    #[test]
    fn test_merge_configs() {
        let mut config = GpioConfig::from_str(BASIC_CFG).unwrap();
        let cfg2 = GpioConfig::from_str(PARTIALLY_OVERLAPS_BASIC_CFG).unwrap();

        // perform the merge
        config.update(cfg2);

        let reset_button = config.pins.get(0).unwrap();
        assert_eq!(reset_button.num, 73);
        assert_eq!(reset_button.names, BTreeSet::from_iter(
            vec!(String::from("reset_button"),
                 String::from("new_name"))));
        assert_eq!(reset_button.direction, D::In);
        assert_eq!(reset_button.active_low, false);
        assert_eq!(reset_button.export, true);

        let status_led = config.pins.get(1).unwrap();
        let names = BTreeSet::from_iter(
            vec!(String::from("status_led"),
                 String::from("A27"),
                 String::from("green_led")));
        assert_eq!(status_led.names, names);
        assert_eq!(status_led.direction, D::In);
        assert_eq!(status_led.active_low, false);
        assert_eq!(status_led.export, true);

        let wildcard = config.pins.get(2).unwrap();
        assert_eq!(wildcard.num, 88);
        assert_eq!(wildcard.names, BTreeSet::from_iter(vec!(String::from("wildcard"))));
    }
}
