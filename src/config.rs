// Copyright (C) 2016, Paul Osborne <osbpau@gmail.com>

use glob::glob;
use rustc_serialize::{Decodable, Decoder};
use std::collections::BTreeSet;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use sysfs_gpio;
use toml;

const DEFAULT_SYMLINK_ROOT: &'static str = "/var/run/gpio";

#[derive(Debug, PartialEq, Clone)]
pub struct Direction(pub sysfs_gpio::Direction);

impl From<sysfs_gpio::Direction> for Direction {
    fn from(e: sysfs_gpio::Direction) -> Self {
        Direction(e)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PinConfig {
    pub num: u64,
    pub direction: sysfs_gpio::Direction,
    pub names: BTreeSet<String>,
    pub export: bool,
    pub active_low: bool,
}

#[derive(Clone, Debug, Default)]
pub struct GpioConfig {
    pins: Vec<PinConfig>,
    symlink_root: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParserErrors(Vec<toml::ParserError>),
    DecodingError(toml::DecodeError),
    NoConfigFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref e) => e.fmt(f),
            Error::ParserErrors(ref errors) => {
                for e in errors {
                    try!(e.fmt(f));
                }
                Ok(())
            }
            Error::DecodingError(ref e) => e.fmt(f),
            Error::NoConfigFound => write!(f, "No Config Found"),
        }
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

impl Decodable for GpioConfig {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        // Get items under the [config] header if present
        let symlink_root: Option<String> = d.read_struct_field("config", 0, |cfg| {
                                                cfg.read_struct_field("symlink_root",
                                                                      0,
                                                                      Decodable::decode)
                                            })
                                            .ok();

        Ok(GpioConfig {
            pins: try!(d.read_struct_field("pins", 0, Decodable::decode)),
            symlink_root: symlink_root,
        })
    }
}

impl Decodable for PinConfig {
    fn decode<D: Decoder>(d: &mut D) -> Result<PinConfig, D::Error> {
        Ok(PinConfig {
            num: try!(d.read_struct_field("num", 0, Decodable::decode)),
            direction: d.read_struct_field("direction", 0, |dir_d| {
                            match &try!(dir_d.read_str())[..] {
                                "in" => Ok(sysfs_gpio::Direction::In),
                                "out" => Ok(sysfs_gpio::Direction::Out),
                                "high" => Ok(sysfs_gpio::Direction::High),
                                "low" => Ok(sysfs_gpio::Direction::Low),
                                _ => Err(dir_d.error("Expected one of: {in, out, high, low}")),
                            }
                        })
                        .unwrap_or(sysfs_gpio::Direction::In), // default: In
            names: d.read_struct_field("names", 0, Decodable::decode).unwrap_or(BTreeSet::new()),
            export: d.read_struct_field("export", 0, Decodable::decode).unwrap_or(true),
            active_low: d.read_struct_field("active_low", 0, Decodable::decode).unwrap_or(false),
        })
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
        match Decodable::decode(&mut toml::Decoder::new(toml::Value::Table(root))) {
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

    /// Get the pin with the provided name if present in this configuration
    pub fn get_pin(&self, name: &str) -> Option<&PinConfig> {
        self.pins.iter().find(|p| p.names.contains(name))
    }

    /// Get a reference to all the pins in this config
    pub fn get_pins(&self) -> &[PinConfig] {
        &self.pins[..]
    }

    /// Get the symlink root specified in the config (or the default)
    pub fn get_symlink_root(&self) -> &str {
        match self.symlink_root {
            Some(ref root) => &root,
            None => DEFAULT_SYMLINK_ROOT,
        }
    }

    /// Merge other into self (takes ownership of other)
    ///
    /// If in conflict, the other GPIO config takes priority.
    pub fn update(&mut self, other: GpioConfig) {
        if let Some(symlink_root) = other.symlink_root {
            self.symlink_root = Some(symlink_root);
        }

        for other_pin in other.pins {
            // determine the case we are dealing with
            let existing = match self.pins.iter_mut().find(|p| p.num == other_pin.num) {
                Some(pin) => {
                    pin.names.extend(other_pin.names.clone());
                    pin.direction = other_pin.direction.clone(); // TODO impl copy
                    pin.export = other_pin.export;
                    pin.active_low = other_pin.active_low;
                    true
                }
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

    const BASIC_CFG: &'static str = r#"
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

    const COMPACT_CFG: &'static str = r#"
pins = [
   { num = 73, names = ["reset_button"], direction = "in", active_low = true, export = true},
   { num = 37, names = ["status_led", "A27", "green_led"], direction = "out"},
]

[config]
symlink_root = "/tmp/gpio"
"#;

    const MISSING_PINNUM_CFG: &'static str = r#"
[[pins]]
export = true
"#;

    const PARTIALLY_OVERLAPS_BASIC_CFG: &'static str = r#"
[config]
symlink_root = "/foo/bar/baz"

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
        let names = BTreeSet::from_iter(vec![String::from("status_led"),
                                             String::from("A27"),
                                             String::from("green_led")]);

        assert_eq!(config.get_symlink_root(), "/var/run/gpio");

        let reset_button = config.pins.get(0).unwrap();
        assert_eq!(reset_button.num, 73);
        assert_eq!(reset_button.names,
                   BTreeSet::from_iter(vec![String::from("reset_button")]));
        assert_eq!(reset_button.direction, D::In);
        assert_eq!(reset_button.active_low, true);
        assert_eq!(reset_button.export, true);

        assert_eq!(status_led.names, names);
        assert_eq!(status_led.direction, D::Out);
        assert_eq!(status_led.active_low, false);
        assert_eq!(status_led.export, true);
    }

    #[test]
    fn test_get_pin_present() {
        let config = GpioConfig::from_str(BASIC_CFG).unwrap();
        let status_led = config.get_pin("status_led").unwrap();
        assert_eq!(status_led.num, 37);
    }

    #[test]
    fn test_get_pin_not_present() {
        let config = GpioConfig::from_str(BASIC_CFG).unwrap();
        assert_eq!(config.get_pin("missing"), None);
    }

    #[test]
    fn test_parser_compact() {
        let config = GpioConfig::from_str(COMPACT_CFG).unwrap();
        let status_led = config.pins.get(1).unwrap();
        let names = BTreeSet::from_iter(vec![String::from("status_led"),
                                             String::from("A27"),
                                             String::from("green_led")]);
        assert_eq!(status_led.names, names);
        assert_eq!(status_led.direction, D::Out);
        assert_eq!(status_led.active_low, false);
        assert_eq!(status_led.export, true);
        assert_eq!(config.get_symlink_root(), "/tmp/gpio")
    }

    #[test]
    fn test_parser_empty_toml() {
        let configstr = "";
        match GpioConfig::from_str(configstr) {
            Ok(pins) => assert_eq!(pins.pins, vec![]),
            _ => panic!("Expected a decoding error"),
        }
    }

    #[test]
    fn test_parser_missing_pinnum() {
        match GpioConfig::from_str(MISSING_PINNUM_CFG) {
            Err(Error::DecodingError(_)) => {}
            _ => panic!("Expected a decoding error"),
        }
    }

    #[test]
    fn test_parse_error_bad_toml() {
        // basically, just garbage data
        let configstr = r"[] -*-..asdf=-=-@#$%^&*()";
        match GpioConfig::from_str(configstr) {
            Err(Error::ParserErrors(_)) => {}
            _ => panic!("Did not receive parse error when expected"),
        }
    }

    #[test]
    fn test_merge_configs() {
        let mut config = GpioConfig::from_str(BASIC_CFG).unwrap();
        let cfg2 = GpioConfig::from_str(PARTIALLY_OVERLAPS_BASIC_CFG).unwrap();

        // perform the merge
        config.update(cfg2);

        assert_eq!(config.get_symlink_root(), "/foo/bar/baz");

        let reset_button = config.pins.get(0).unwrap();
        assert_eq!(reset_button.num, 73);
        assert_eq!(reset_button.names,
                   BTreeSet::from_iter(vec![String::from("reset_button"),
                                            String::from("new_name")]));
        assert_eq!(reset_button.direction, D::In);
        assert_eq!(reset_button.active_low, false);
        assert_eq!(reset_button.export, true);

        let status_led = config.pins.get(1).unwrap();
        let names = BTreeSet::from_iter(vec![String::from("status_led"),
                                             String::from("A27"),
                                             String::from("green_led")]);
        assert_eq!(status_led.names, names);
        assert_eq!(status_led.direction, D::In);
        assert_eq!(status_led.active_low, false);
        assert_eq!(status_led.export, true);

        let wildcard = config.pins.get(2).unwrap();
        assert_eq!(wildcard.num, 88);
        assert_eq!(wildcard.names,
                   BTreeSet::from_iter(vec![String::from("wildcard")]));
    }
}
