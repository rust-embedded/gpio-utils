// Copyright (C) 2016, The gpio-utils Authors

use std::path;
use std::os::unix::fs as unix_fs;
use std::fs;
use std::io::ErrorKind;
use config::PinConfig;
use sysfs_gpio;

/// Unexport the pin specified in the provided config
///
/// Unexporting a config (in this context) involves a few different
/// actions:
///
/// 1. For each GPIO name/alias, the corresponding symlink is remvoed from
///    `/var/run/gpio/<name>` (or an alternate configured symlink_root).
/// 2. The GPIO pin istself is unexported (vai /sys/class/gpio/unexport)
///
/// If the GPIO was already unexported, this function will continue
/// without an error as the desired end state is achieved.
pub fn unexport(pin_config: &PinConfig,
                symlink_root: Option<&str>)
                -> Result<(), sysfs_gpio::Error> {
    if let Some(symroot) = symlink_root {
        // create symlink for each name
        for name in &pin_config.names {
            let mut dst = path::PathBuf::from(symroot);
            dst.push(name);
            try!(match fs::remove_file(dst) {
                Ok(_) => Ok(()),
                Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
            });
        }
    }

    // unexport the pin itself.  On many boards, it turns out, some pins are
    // exported by the kernel itself but we might still be assigning names.  In
    // those cases we will get an error here.  We handle that rather than
    // exposing the error up the chain. (EINVAL)
    let pin = pin_config.get_pin();
    match pin.unexport() {
        Ok(_) => Ok(()),
        Err(sysfs_gpio::Error::Io(ref e)) if e.kind() == ErrorKind::InvalidInput => Ok(()),
        Err(e) => Err(e)
    }
}

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

        // set the pin direction
        try!(pin_config.get_pin().set_direction(pin_config.direction.clone()));

        // create symlink for each name
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
