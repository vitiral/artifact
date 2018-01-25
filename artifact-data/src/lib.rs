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

#![allow(unknown_lints)]
#![allow(doc_markdown)]

extern crate failure;
extern crate itertools;
#[macro_use]
extern crate matches;
#[macro_use]
extern crate ordermap;
extern crate path_abs;
extern crate petgraph;
extern crate rayon;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate std_prelude;
extern crate toml;
extern crate walkdir;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;
extern crate time;

// MODULES

mod artifact;
mod dev_prelude;
#[macro_use]
mod name;
mod expand_names;
#[macro_use]
mod family;
mod graph;
mod implemented;
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

#[cfg(test)]
extern crate rand;

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
pub use name::{Name, NameError, SubName, Type, NAME_VALID_STR};
pub use settings::ProjectPaths;
pub use lint::{Categorized, Category, Level, Lint};
pub use project::{load_project, Project};
pub use graph::Completed;
