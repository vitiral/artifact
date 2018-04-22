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
//! This subcrate is to provide a common testing framework/functions
//! for testing artifact.
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_macros)]

pub extern crate base64;
pub extern crate ergo;
#[macro_use]
pub extern crate expect_macro;
pub extern crate failure;
#[macro_use]
pub extern crate matches;
pub extern crate petgraph;
pub extern crate rayon;
pub extern crate siphasher;

#[macro_use]
pub extern crate failure_derive;
pub extern crate time;

#[macro_use]
pub extern crate artifact_data;
#[macro_use]
pub extern crate artifact_lib;
#[macro_use]
extern crate log;

#[macro_use]
extern crate proptest;

#[macro_use]
extern crate pretty_assertions;

// #[cfg(test)]
// extern crate rand;

extern crate regex_generate;
extern crate tempdir;
extern crate unicode_segmentation;

pub mod dev_prelude;
pub mod name;
pub mod family;
pub mod graph;
pub mod implemented;
pub mod raw;
#[macro_use]
pub mod raw_names;
pub mod framework;

pub use framework::{assert_stuff_data, run_generic_interop_test, run_generic_interop_tests,
                    ExpectStuff};
pub use artifact_data::*; // for macros
pub use proptest::*;
pub use dev_prelude::*;

// pub use dev_prelude::assert_generic;
// pub use family::arb_names;
// pub use name::{arb_name, arb_name_string, names_raw};
// pub use raw_names::arb_names_raw;
