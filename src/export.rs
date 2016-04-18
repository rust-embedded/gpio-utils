// Copyright (C) 2016, The gpio-utils Authors

use std::path;
use std::os::unix::fs as unix_fs;
use std::fs;
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
pub fn export(pin_config: &PinConfig, symlink_root: Option<&str>) -> Result<(), sysfs_gpio::Error> {
    let pin = pin_config.get_pin();
    try!(pin.export());

    // if there is a symlink root provided, create symlink
    if let Some(symroot) = symlink_root {
        // create root directory if not exists
        try!(fs::create_dir_all(symroot));

        for name in &pin_config.names {
            let mut dst = path::PathBuf::from(symroot);
            dst.push(name);
            try!(match unix_fs::symlink(format!("/sys/class/gpio{}", pin_config.num), dst) {
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
