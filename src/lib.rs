/*  rst: the requirements tracking tool made for developers
    Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the Lesser GNU General Public License as published 
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the Lesser GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/
#![allow(unknown_lints)]  // need for clippy lints
#![cfg_attr(feature = "serde_derive", feature(proc_macro))]
// # logger config
extern crate fern;

// # general crates
extern crate itertools;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

// # core crates
extern crate regex;
extern crate strfmt;
extern crate time;
extern crate rustc_serialize;
extern crate toml;

// # ui-cmdline crates
extern crate clap;
extern crate ansi_term;

// serde hybrid requirements
#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

#[cfg(feature = "serde_derive")]
include!("serde_types.in.rs");

#[cfg(feature = "serde_codegen")]
include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

// modules
pub mod core;
pub mod ui;
pub mod cmd;


pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn init_logger(quiet: bool, verbosity: u8, stderr: bool) -> Result<(), fern::InitError> {
    let level = if quiet {log::LogLevelFilter::Off } else {
        match verbosity {
            0 => log::LogLevelFilter::Warn,
            1 => log::LogLevelFilter::Info,
            2 => log::LogLevelFilter::Debug,
            3 => log::LogLevelFilter::Trace,
            _ => unreachable!(),
        }
    };
    let output = if stderr {
        fern::OutputConfig::stderr()
    } else {
        fern::OutputConfig::stdout()
    };

    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("{}: {}", level, msg)
        }),
        output: vec![output],
        level: level,
    };
    fern::init_global_logger(logger_config, log::LogLevelFilter::Trace)
}
