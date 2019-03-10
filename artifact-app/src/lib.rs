/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
//! The CLI main binary

#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate artifact_data;

#[macro_use]
extern crate artifact_lib;

#[allow(unused_imports)]
#[macro_use]
extern crate ergo;

#[allow(unused_imports)]
#[macro_use]
extern crate quicli;

#[macro_use]
extern crate expect_macro;
use termstyle;

use jrpc;

#[allow(unused_imports)]
use ergo::*;
#[allow(unused_imports)]
use quicli::prelude::*;

// macro modules must come first
#[macro_use]
mod dev_prelude;

mod check;
mod export;
mod fmt;
mod frontend;
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
        .subcommand(serve::Serve::clap())
        .subcommand(export::Export::clap());

    let matches = app.get_matches();

    match matches.subcommand() {
        ("init", Some(args)) => init::run(init::Init::from_clap(&args)),
        ("check", Some(args)) => check::run(check::Check::from_clap(&args)),
        ("fmt", Some(args)) => fmt::run(fmt::Fmt::from_clap(&args)),
        ("ls", Some(args)) => ls::run(ls::Ls::from_clap(&args)),
        ("serve", Some(args)) => serve::run(serve::Serve::from_clap(&args)),
        ("export", Some(args)) => export::run(export::Export::from_clap(&args)),
        ("", _) => {
            eprintln!(
                "Error: must specify a subcommand. Use `art help` for a list of subcommands."
            );
            Ok(1)
        }
        (sub, _) => {
            eprintln!(
                "Error: subcommand '{}' is not valid. Use `art help` for a list of subcommands.",
                sub
            );
            Ok(1)
        }
    }
}
