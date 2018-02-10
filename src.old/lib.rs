/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! artifact library
// need for clippy lints
#![allow(unknown_lints)]
#![allow(zero_ptr)]
#![recursion_limit = "1024"]

// # general crates

#[macro_use]
extern crate error_chain;
extern crate fern;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;

// # core crates

extern crate difference;
extern crate regex;
extern crate strfmt;
extern crate time;
extern crate unicode_segmentation;
extern crate unicode_width;

// # cmdline crates

extern crate ansi_term;
extern crate clap;
#[macro_use]
extern crate self_update;
extern crate tabwriter;
extern crate tar;

// # server crates

extern crate ctrlc;
extern crate jsonrpc_core;
extern crate nickel;
extern crate tempdir;

// # serialization

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;
extern crate uuid;

// crates for test

#[cfg(test)]
extern crate fs_extra;

// "core" modules
pub mod dev_prefix;
#[macro_use]
pub mod types;
pub mod user;
pub mod logging;
pub mod export;
pub mod utils;
pub mod security;

// command line modules
pub mod ui;
pub mod cmd;

#[cfg(test)]
pub mod test_data;
pub mod api;

pub use types::*;
