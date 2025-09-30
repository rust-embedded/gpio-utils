// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use glob::glob;
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;
use sysfs_gpio;
use toml;

const DEFAULT_SYMLINK_ROOT: &str = "/var/run/gpio";

#[derive(Debug, PartialEq, Clone)]
pub struct Direction(pub sysfs_gpio::Direction);

#[derive(Deserialize, Debug)]
#[serde(remote = "sysfs_gpio::Direction")]
pub enum DirectionDef {
    #[serde(rename = "in")]
    In,
    #[serde(rename = "out")]
    Out,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "low")]
    Low,
}

impl From<sysfs_gpio::Direction> for Direction {
    fn from(e: sysfs_gpio::Direction) -> Self {
        Direction(e)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PinConfig {
    pub num: u64,
    #[serde(default = "default_direction")]
    #[serde(with = "DirectionDef")]
    pub direction: sysfs_gpio::Direction,
    #[serde(default)]
    pub names: BTreeSet<String>,
    #[serde(default = "bool_true")]
    pub export: bool,
    #[serde(default)]
    pub active_low: bool,
    pub user: Option<String>,
    pub group: Option<String>,
    pub mode: Option<u32>,
}

fn default_direction() -> sysfs_gpio::Direction {
    sysfs_gpio::Direction::In
}

fn bool_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct GpioConfig {
    pub pins: Vec<PinConfig>,
    #[serde(default)]
    pub config: SysConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SysConfig {
    pub symlink_root: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParserErrors(toml::de::Error),
    NoConfigFound,
    DuplicateNames(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref e) => e.fmt(f),
            Error::ParserErrors(ref e) => e.fmt(f),
            Error::NoConfigFound => write!(f, "No Config Found"),
            Error::DuplicateNames(ref e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl PinConfig {
    /// Get the `sysfs_gpio::Pin` to go along with this config`
    pub fn get_pin(&self) -> sysfs_gpio::Pin {
        sysfs_gpio::Pin::new(self.num)
    }
}

impl FromStr for GpioConfig {
    type Err = Error;
    /// Load a GPIO configuration for the provided toml string
    fn from_str(config: &str) -> Result<Self, Error> {
        let cfg = toml::from_str(config);
        match cfg {
            Ok(cfg) => {
                let val_config: GpioConfig = toml::from_str(config).unwrap();
                val_config.validate()?;
                Ok(cfg)
            }
            Err(e) => Err(Error::ParserErrors(e)),
        }
    }
}

impl GpioConfig {
    /// Validate invariants on the config that cannot easily be done earlier
    ///
    /// Currently, this just checks that there are no duplicated names between
    /// different pins in the config
    fn validate(&self) -> Result<(), Error> {
        let mut all_names: HashMap<&str, &PinConfig> = HashMap::new();
        for pin in &self.pins {
            for name in &pin.names {
                if let Some(other_pin) = all_names.get(&name[..]) {
                    return Err(Error::DuplicateNames(format!(
                        "Pins {} and {} share duplicate \
                         name '{}'",
                        pin.num, other_pin.num, name
                    )));
                }
                all_names.insert(&name[..], pin);
            }
        }

        Ok(())
    }

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
            config_instances.push(Self::from_file("/etc/gpio.toml")?);
        }
        // /etc/gpio.d/*.toml
        for fragment in glob("/etc/gpio.d/*.toml").unwrap().filter_map(Result::ok) {
            config_instances.push(Self::from_file(fragment)?);
        }

        // additional from command-line
        for fragment in configs {
            config_instances.push(Self::from_file(fragment)?);
        }

        if config_instances.is_empty() {
            Err(Error::NoConfigFound)
        } else {
            let mut cfg = config_instances.remove(0);
            for higher_priority_cfg in config_instances {
                cfg.update(higher_priority_cfg)?;
            }
            Ok(cfg)
        }
    }

    /// Load a GPIO config from the specified path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<GpioConfig, Error> {
        let mut contents = String::new();
        let mut f = File::open(path)?;
        f.read_to_string(&mut contents)?;
        let config = GpioConfig::from_str(&contents[..])?;
        config.validate()?;

        Ok(config)
    }

    /// Get the pin with the provided name if present in this configuration
    pub fn get_pin(&self, name: &str) -> Option<&PinConfig> {
        // first, try to find pin by name
        if let Some(pin) = self.pins.iter().find(|p| p.names.contains(name)) {
            return Some(pin);
        }

        // Try to parse the name as a 64-bit integer and match against that
        match name.parse::<u64>() {
            Ok(pin_num) => self.pins.iter().find(|p| p.num == pin_num),
            Err(_) => None,
        }
    }

    /// Get a reference to all the pins in this config
    pub fn get_pins(&self) -> &[PinConfig] {
        &self.pins[..]
    }

    /// Get the symlink root specified in the config (or the default)
    pub fn get_symlink_root(&self) -> &str {
        match self.config.symlink_root {
            Some(ref root) => root,
            None => DEFAULT_SYMLINK_ROOT,
        }
    }

    /// Merge other into self (takes ownership of other)
    ///
    /// If in conflict, the other GPIO config takes priority.
    pub fn update(&mut self, other: GpioConfig) -> Result<(), Error> {
        if let Some(symlink_root) = other.config.symlink_root {
            self.config.symlink_root = Some(symlink_root);
        }
        for other_pin in other.pins {
            // determine the case we are dealing with
            let existing = match self.pins.iter_mut().find(|p| p.num == other_pin.num) {
                Some(pin) => {
                    pin.names.extend(other_pin.names.clone());
                    pin.direction = other_pin.direction;
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

        // validate the resulting structure
        self.validate()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeSet;
    use std::iter::FromIterator;
    use std::str::FromStr;
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

    const DUPLICATED_NAMES_CFG: &'static str = r#"
[[pins]]
num = 25
names = ["foo", "bar"]

[[pins]]
num = 26
names = ["baz", "foo"]  # foo is repeated!
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
        let names = BTreeSet::from_iter(vec![
            String::from("status_led"),
            String::from("A27"),
            String::from("green_led"),
        ]);

        assert_eq!(config.get_symlink_root(), "/var/run/gpio");

        let reset_button = config.pins.get(0).unwrap();
        assert_eq!(reset_button.num, 73);
        assert_eq!(
            reset_button.names,
            BTreeSet::from_iter(vec![String::from("reset_button")])
        );
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
    fn test_get_pin_by_number() {
        let config = GpioConfig::from_str(BASIC_CFG).unwrap();
        let status_led = config.get_pin("37").unwrap();
        assert_eq!(status_led.num, 37);
    }

    #[test]
    fn test_get_pin_by_number_not_found() {
        let config = GpioConfig::from_str(BASIC_CFG).unwrap();
        assert_eq!(config.get_pin("64"), None);
    }

    #[test]
    fn test_parser_compact() {
        let config = GpioConfig::from_str(COMPACT_CFG).unwrap();
        let status_led = config.pins.get(1).unwrap();
        let names = BTreeSet::from_iter(vec![
            String::from("status_led"),
            String::from("A27"),
            String::from("green_led"),
        ]);
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
            Err(Error::ParserErrors(_)) => {}
            _ => panic!("Expected a parsing error"),
        }
    }

    #[test]
    fn test_parser_missing_pinnum() {
        match GpioConfig::from_str(MISSING_PINNUM_CFG) {
            Err(Error::ParserErrors(_)) => {}
            _ => panic!("Expected a parsing error"),
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
    fn test_error_on_duplicated_names() {
        match GpioConfig::from_str(DUPLICATED_NAMES_CFG) {
            Err(Error::DuplicateNames(_)) => (),
            r => panic!("Expected DuplicateNames Error, got {:?}", r),
        }
    }

    #[test]
    fn test_merge_configs() {
        let mut config = GpioConfig::from_str(BASIC_CFG).unwrap();
        let cfg2 = GpioConfig::from_str(PARTIALLY_OVERLAPS_BASIC_CFG).unwrap();

        // perform the merge
        config.update(cfg2).unwrap();

        assert_eq!(config.get_symlink_root(), "/foo/bar/baz");

        let reset_button = config.pins.get(0).unwrap();
        assert_eq!(reset_button.num, 73);
        assert_eq!(
            reset_button.names,
            BTreeSet::from_iter(vec![String::from("reset_button"), String::from("new_name")])
        );
        assert_eq!(reset_button.direction, D::In);
        assert_eq!(reset_button.active_low, false);
        assert_eq!(reset_button.export, true);

        let status_led = config.pins.get(1).unwrap();
        let names = BTreeSet::from_iter(vec![
            String::from("status_led"),
            String::from("A27"),
            String::from("green_led"),
        ]);
        assert_eq!(status_led.names, names);
        assert_eq!(status_led.direction, D::In);
        assert_eq!(status_led.active_low, false);
        assert_eq!(status_led.export, true);

        let wildcard = config.pins.get(2).unwrap();
        assert_eq!(wildcard.num, 88);
        assert_eq!(
            wildcard.names,
            BTreeSet::from_iter(vec![String::from("wildcard")])
        );
    }
}
