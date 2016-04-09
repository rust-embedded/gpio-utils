// Copyright (C) 2016, The gpio-utils Authors

use config::PinConfig;
use sysfs_gpio;

#[derive(Debug)]
pub enum ExportError {
    GpioError(sysfs_gpio::Error),
}

/// Export the pin specified in the provided config
///
/// Exporting a pin (in this context) involves, a few different
/// actions:
///
/// 1. The GPIO pin itself is exported (via /sys/class/gpoi/export)
/// 2. For each GPIO name/alias, a symlink is created from
///     `/var/run/gpio/<name>` -> `/sys/class/gpio<num>`.
///
/// If the GPIO is already exported, this function will continue
/// without an error as the desired end state is achieved.
pub fn export(pin: &PinConfig) -> Result<(), ExportError> {
    Ok(())
}
