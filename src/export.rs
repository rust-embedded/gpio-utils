// Copyright (C) 2016, The gpio-utils Authors

use std::path;
use std::os::unix::fs;
use std::io::ErrorKind;
use config::PinConfig;
use sysfs_gpio;

/// Export the pin specified in the provided config
///
/// Exporting a pin (in this context) involves, a few different
/// actions:
///
/// 1. The GPIO pin itself is exported (via /sys/class/gpio/export)
/// 2. For each GPIO name/alias, a symlink is created from
///     `/var/run/gpio/<name>` -> `/sys/class/gpio<num>`.
///
/// If the GPIO is already exported, this function will continue
/// without an error as the desired end state is achieved.
pub fn export(pin: &PinConfig, symlink_root: Option<&str>) -> Result<(), sysfs_gpio::Error> {
    let sysfs_gpio_pin = sysfs_gpio::Pin::new(pin.num);
    try!(sysfs_gpio_pin.export());

    // if there is a symlink root provided, create symlink
    if let Some(symroot) = symlink_root {
        for name in &pin.names {
            let mut dst = path::PathBuf::from(symroot);
            dst.push(name);
            try!(match fs::symlink(format!("/sys/class/gpio{}", pin.num), dst) {
                Ok(_) => Ok(()),
                Err(e) => {
                    match e.kind() {
                        ErrorKind::AlreadyExists => Ok(()),
                        _ => Err(e),
                    }
                }
            });
        }
    }

    Ok(())
}
