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
//! The artifact data crate defines the method of serializing
//! and deserializing raw artifact and processing them into
//! a full project.
//!
//! Note that almost all tests for artifact-data are in artifact-test

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(doc_markdown)]

extern crate base64;
extern crate ergo;
#[macro_use]
extern crate expect_macro;
extern crate failure;
#[macro_use]
extern crate matches;
extern crate petgraph;
extern crate rayon;
extern crate siphasher;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate failure_derive;
extern crate time;

#[macro_use]
extern crate artifact_lib;
extern crate artifact_ser;
#[macro_use]
extern crate log;

// MODULES

pub mod artifact;
mod dev_prelude;
pub mod graph;
pub mod implemented;
mod intermediate;
mod modify;
mod project;
pub mod raw;
#[macro_use]
pub mod raw_names;
mod settings;

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

// #[cfg(test)]
// extern crate rand;

#[cfg(test)]
extern crate regex_generate;

#[cfg(test)]
extern crate tempdir;

#[cfg(test)]
extern crate unicode_segmentation;

pub use modify::modify_project;
pub use project::read_project;
pub use settings::{ART_DIR, SETTINGS_FILE};
