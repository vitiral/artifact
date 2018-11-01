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
//! This subcrate is to provide a common testing framework/functions
//! for testing artifact.
//!
//! Related:
//! - #TST-unit
//! - #TST-fuzz
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

pub mod artifact;
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
