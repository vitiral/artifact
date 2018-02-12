/* artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018  Garrett Berg <@vitiral, vitiral@gmail.com>
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
//! The CLI main binary

#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate artifact_data;

#[allow(unused_imports)]
#[macro_use]
extern crate ergo;

#[macro_use]
extern crate ordermap;

#[allow(unused_imports)]
#[macro_use]
extern crate quicli;

#[macro_use]
extern crate expect_macro;
extern crate termstyle;

extern crate nickel;
extern crate jsonrpc_core;

// #[macro_use]
// #[cfg(test)]
// extern crate pretty_assertions;

#[allow(unused_imports)]
use ergo::*;
#[allow(unused_imports)]
use quicli::prelude::*;

// macro modules must come first
#[macro_use]
mod dev_prelude;

mod check;
mod fmt;
mod init;
mod ls;
mod serve;

/// #SPC-cli
pub fn run() -> Result<i32> {
    use quicli::prelude::structopt::clap::*;
    let app = App::new("art")
        .author("vitiral")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Design documentation tool for everybody.")
        .subcommand(init::Init::clap())
        .subcommand(check::Check::clap())
        .subcommand(fmt::Fmt::clap())
        .subcommand(ls::Ls::clap())
        .subcommand(serve::Serve::clap());

    let matches = app.get_matches();

    match matches.subcommand() {
        ("ls", Some(args)) => ls::run(ls::Ls::from_clap(&args)),
        ("init", Some(args)) => init::run(init::Init::from_clap(&args)),
        ("check", Some(args)) => check::run(check::Check::from_clap(&args)),
        ("fmt", Some(args)) => fmt::run(fmt::Fmt::from_clap(&args)),
        ("serve", Some(args)) => serve::run(serve::Serve::from_clap(&args)),
        (sub, _) => unimplemented!("sub: {}", sub),
    }
}
