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

#[allow(unused_imports)]
use ergo::*;
#[allow(unused_imports)]
use quicli::prelude::*;

// macro modules must come first
#[macro_use]
mod dev_prelude;

mod ls;

pub fn run() -> Result<i32> {
    use quicli::prelude::structopt::clap::*;
    let app = App::new("art")
        .author("vitiral")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Design documentation tool for everybody.")
        .subcommand(ls::Ls::clap());

    let matches = app.get_matches();

    match matches.subcommand() {
        ("ls", Some(args)) => ls::run(ls::Ls::from_clap(args.clone())),
        (sub, _) => unimplemented!("sub: {}", sub),
    }
}
