/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */

//! artifact library
// need for clippy lints
#![allow(unknown_lints)]
#![allow(zero_ptr)]
#![recursion_limit = "1024"]

// # general crates
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate fern;
extern crate itertools;

// # core crates
extern crate regex;
extern crate strfmt;
extern crate time;
extern crate rustc_serialize;
extern crate difference;
extern crate unicode_segmentation;
extern crate unicode_width;

// # cmdline crates
extern crate clap;
extern crate ansi_term;
extern crate tabwriter;
extern crate tar;

// # server crates
#[cfg(feature="server")]
extern crate nickel;
#[cfg(feature="server")]
extern crate jsonrpc_core;
extern crate tempdir;

#[macro_use]
#[cfg(feature="server")]
extern crate diesel;
#[cfg(feature="server")]
#[macro_use]
extern crate diesel_codegen;
#[cfg(feature="server")]
extern crate dotenv;
#[cfg(features="server")]
extern crate chrono;

// serialization
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate toml;

// crates for test
#[cfg(test)]
extern crate fs_extra;

// "core" modules
pub mod dev_prefix;
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

// server modules
#[cfg(feature="server")]
pub mod api;

pub use types::*;

