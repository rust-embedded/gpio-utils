# Linux GPIO Utils

[![Build Status](https://travis-ci.org/rust-embedded/gpio-utils.svg?branch=master)](https://travis-ci.org/rust-embedded/gpio-utils)
[![Version](https://img.shields.io/crates/v/gpio-utils.svg)](https://crates.io/crates/gpio-utils)
[![License](https://img.shields.io/crates/l/rustc-serialize.svg)](https://github.com/rust-embedded/gpio-utils/blob/master/README.md#license)

GPIO Utils provides convenient access to GPIOs on a Linux system. The library
builds on top of the sysfs interface to GPIOs exposed by the kernel and provides
essential functionality required for most embedded systems.

## Install

To install the latest released version of gpio utils, ensure that you have
installed Rust and then run:

```sh
cargo install gpio-utils
```

## Features

- [x] Infrastructure for providing names for the GPIOs in one's system providing
      names that map to individual pins.  These names (in addition to GPIO numbers)
      may be used with other commands.
- [x] Ability to export/unexport GPIOs and expose symlinks using the GPIO "friendly"
      names simpler.
- [x] Ability to set input/output state on each pin
- [x] Ability to set active low state on each pin
- [x] Ability to get/set gpio values by pin number or name (including temporary
      export if necessary)
- [x] Ability to block awaiting pin state change (with timeout)
- [x] Ability to set exported GPIO permissions

## System Integration

GPIO Utils provides two main pieces that one may integrate into their final
system:

1. The `gpio` command.  This provides the core functionality for GPIO Utils and
   is useful in its own right.
2. The `gpio` init script/systemd service.  This can be integrated into a target
   system and will ensure that configured GPIOs get exported on system startup
   (The GPIO command searches for `/etc/gpio.toml` and `/etc/gpio.d/*.toml`
   configs)

The GPIO Utils library is built on top of the
[Rust sysfs-gpio](https://github.com/rust-embedded/rust-sysfs-gpio) library
which may be used independent of this project.

## GPIO Configuration File

GPIO Utils uses the [TOML](https://github.com/toml-lang/toml).  There is some
flexibility in the configuration, but the following examples shows the basics of
how you can configure your GPIOs.

```toml
#
# Example GPIO configuration (e.g. /etc/gpio.toml)
#
# The main configuration consists of zero or more pins, each of which may have
# the following keys:
#
# - `num`: Required.  The GPIO number.
# - `names`: Required.  One or more names for the GPIO
# - `direction`: Default: `"in"`.  Must be either "in" or "out"
# - `active_low`: Default: `false`.  If set to true, the polarity of the pin will
#    be reversed.
# - `export`: Default: `true`.  If true, this GPIO will be automatically
#    exported when `gpio export-all` is run (e.g. by an init script).
# - `user`: User that should own the exported GPIO
# - `group`: Group that should own the exported GPIO
# - `mode`: Mode for exported directory

[[pins]]
num = 73                 # required
names = ["reset_button"] # required (may have multiple)
direction = "in"         # default: in
active_low = false       # default: false (really means invert logic)
export = true            # default: true
user = "root"            # default: (OS Default - root)
group = "gpio"           # default: (OS Default - root)
mode = 0o664             # default: (OS Default - 0o644)

[[pins]]
num = 37
names = ["status_led", "A27", "green_led"]
direction = "out"

# ...
```

## Implementation Notes

Unlike several other existing solutions to this problem, this project is
implemented in Rust (a modern systems programming language operating at the same
level as C but with a type system providing greater productivity and
reliability) and seeks to operate with a minimum of overhead.

## Contributing

Contributions are very welcome.  See [CONTRIBUTING.md](CONTRINBUTING.md) for
additional information on how to report bugs, submit changes, test changes, get
support, etc.

## License

```
Copyright (c) 2018, The gpio-utils Authors.

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
http://www.apache.org/license/LICENSE-2.0> or the MIT license
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
option.  This file may not be copied, modified, or distributed
except according to those terms.
```
