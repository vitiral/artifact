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
//! The artifact data crate defines the method of serializing
//! and deserializing raw artifact and processing them into
//! a full project.

#![allow(dead_code)]
#![allow(unknown_lints)]
#![allow(doc_markdown)]

extern crate base64;
extern crate ergo;
#[macro_use]
extern crate expect_macro;
extern crate failure;
#[macro_use]
extern crate matches;
#[macro_use]
extern crate ordermap;
extern crate petgraph;
extern crate rayon;
extern crate siphasher;

#[macro_use]
extern crate failure_derive;
extern crate time;

#[macro_use]
extern crate log;

// MODULES

mod artifact;
mod dev_prelude;
mod modify;
#[macro_use]
mod name;
mod expand_names;
#[macro_use]
mod family;
mod graph;
mod implemented;
mod intermediate;
mod lint;
mod project;
mod raw;
#[macro_use]
mod raw_names;
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

#[cfg(test)]
pub mod test;

pub use artifact::Artifact;
pub use implemented::{CodeLoc, Impl, ImplCode};
pub use intermediate::{ArtifactIm, HashIm};
pub use name::{Name, NameError, SubName, Type, NAME_VALID_STR};
pub use settings::ProjectPaths;
pub use lint::{Categorized, Category, Level, Lint};
pub use project::{read_project, Project};
pub use graph::Completed;
