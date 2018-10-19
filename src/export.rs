// Copyright (c) 2016, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use config::PinConfig;
use error::*;
use nix::unistd::{chown, Gid, Uid};
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs as unix_fs;
use std::os::unix::fs::PermissionsExt;
use std::path;
use std::sync::Mutex;
use sysfs_gpio;
use users::{Groups, Users, UsersCache};

lazy_static! {
    static ref USERS_CACHE: Mutex<UsersCache> = Mutex::new(UsersCache::new());
}

/// Unexport the pin specified in the provided config
///
/// Unexporting a config (in this context) involves a few different
/// actions:
///
/// 1. For each GPIO name/alias, the corresponding symlink is remvoed from
///    `/var/run/gpio/<name>` (or an alternate configured `symlink_root`).
/// 2. The GPIO pin istself is unexported (vai /sys/class/gpio/unexport)
///
/// If the GPIO was already unexported, this function will continue
/// without an error as the desired end state is achieved.
pub fn unexport(pin_config: &PinConfig, symlink_root: Option<&str>) -> Result<()> {
    if let Some(symroot) = symlink_root {
        // create symlink for each name
        for name in &pin_config.names {
            let mut dst = path::PathBuf::from(symroot);
            dst.push(name);
            match fs::remove_file(dst) {
                Ok(_) => (),
                Err(ref e) if e.kind() == ErrorKind::NotFound => (),
                Err(e) => return Err(e.into()),
            };
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
        Err(e) => Err(e.into()),
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
pub fn export(pin_config: &PinConfig, symlink_root: Option<&str>) -> Result<()> {
    let pin = pin_config.get_pin();
    pin.export()?;

    let uid = if let Some(username) = pin_config.user.as_ref() {
        Some(
            USERS_CACHE
                .lock()
                .unwrap()
                .get_user_by_name(username)
                .map(|u| Uid::from_raw(u.uid()))
                .ok_or_else(|| format!("Unable to find user {:?}", username))?,
        )
    } else {
        None
    };

    let gid = if let Some(groupname) = pin_config.group.as_ref() {
        Some(
            USERS_CACHE
                .lock()
                .unwrap()
                .get_group_by_name(groupname)
                .map(|g| Gid::from_raw(g.gid()))
                .ok_or_else(|| format!("Unable to find group {:?}", groupname))?,
        )
    } else {
        None
    };

    // change user, group, mode for files in gpio directory
    if uid.is_some() || gid.is_some() || pin_config.mode.is_some() {
        for entry in fs::read_dir(format!("/sys/class/gpio/gpio{}", &pin_config.num))? {
            let e = entry?;
            let metadata = e.metadata()?;

            if metadata.is_file() {
                if uid.is_some() || gid.is_some() {
                    chown(e.path().as_path(), uid, gid)?;
                }

                if let Some(mode) = pin_config.mode {
                    let mut permissions = metadata.permissions();
                    permissions.set_mode(mode);
                    fs::set_permissions(e.path().as_path(), permissions)?;
                }
            }
        }
    }

    // if there is a symlink root provided, create symlink
    if let Some(symroot) = symlink_root {
        // create root directory if not exists
        fs::create_dir_all(symroot)?;

        // set the pin direction
        pin_config
            .get_pin()
            .set_direction(pin_config.direction.clone())?;

        // set active low directio
        pin_config
            .get_pin()
            .set_active_low(pin_config.active_low.clone())?;

        // create symlink for each name
        for name in &pin_config.names {
            let mut dst = path::PathBuf::from(symroot);
            dst.push(name);
            match unix_fs::symlink(format!("/sys/class/gpio/gpio{}", pin_config.num), dst) {
                Err(ref e) if e.kind() == ErrorKind::AlreadyExists => (),
                Err(e) => return Err(e.into()),
                _ => (),
            };
        }
    }

    Ok(())
}
