# Linux GPIO Utils

[![Build Status](https://img.shields.io/github/actions/workflow/status/rust-embedded/gpio-utils/ci.yaml?branch=master&logo=github)](https://github.com/rust-embedded/gpio-utils/actions)
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

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.75.0 and up. It *might*
compile with older versions but that may change in any new patch release.

## Contributing

Contributions are very welcome.

## License


Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Code of Conduct

Contribution to this crate is organized under the terms of the [Rust Code of
Conduct][CoC], the maintainer of this crate, the [Embedded Linux Team][team], promises
to intervene to uphold that code of conduct.

[CoC]: CODE_OF_CONDUCT.md
[team]: https://github.com/rust-embedded/wg#the-embedded-linux-team
