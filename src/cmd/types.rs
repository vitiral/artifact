/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

// stdlib
pub use std::process::exit;

// crates
pub use ansi_term::Style;
pub use ansi_term::Colour::{Blue, Green, Red, Yellow};
pub use clap::{App, AppSettings as AS, Arg, ArgMatches, Result as ClapResult, SubCommand};

// module types
pub use ui;
pub use ui::{FmtArtifact, FmtSettings, PercentSearch, SearchSettings};

#[cfg(not(windows))]
pub const SUBCMD_SETTINGS: [AS; 3] = [AS::DeriveDisplayOrder, AS::ColorAuto, AS::ColoredHelp];

#[cfg(windows)]
pub const SUBCMD_SETTINGS: [AS; 1] = [AS::DeriveDisplayOrder];

lazy_static!{
    pub static ref APP_SETTINGS: Vec<AS> = {
        let mut v = vec![
            AS::ArgRequiredElseHelp,
            AS::SubcommandRequiredElseHelp,
            AS::VersionlessSubcommands,
        ];
        v.extend_from_slice(&SUBCMD_SETTINGS);
        v
    };
}

#[cfg(windows)]
pub const COLOR_IF_POSSIBLE: bool = false;

#[cfg(not(windows))]
pub const COLOR_IF_POSSIBLE: bool = true;
