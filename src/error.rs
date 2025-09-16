// Copyright (c) 2018, The gpio-utils Authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use error_chain::error_chain;
use nix::Error as NixError;
use std::io::Error as IoError;
use sysfs_gpio::Error as GpioError;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Gpio(GpioError);
        Nix(NixError);
        Io(IoError);
    }
}
