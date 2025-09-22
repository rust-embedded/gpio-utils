// Copyright (c) 2018, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use nix::Error as NixError;
use std::io::Error as IoError;
use sysfs_gpio::Error as GpioError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Gpio(#[from] GpioError),
    #[error(transparent)]
    Nix(#[from] NixError),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error("{0}")]
    Msg(String),
}

impl From<String> for Error {
    fn from(msg: String) -> Error {
        Error::Msg(msg)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
