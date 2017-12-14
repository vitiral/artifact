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

#![allow(unused_imports)]

extern crate failure;
#[macro_use(izip)]
extern crate itertools;
#[macro_use]
extern crate matches;
extern crate std_prelude;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
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

// MODULES

mod dev_prelude;
#[cfg(feature = "cache")]
pub mod cache;
#[macro_use]
mod name;
mod expand_names;
#[macro_use]
mod family;
mod implemented;
mod lint;
mod path_abs;
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
pub mod test;


pub use name::{InternalName, Name, NameError, Type, NAME_VALID_STR};
pub use raw::ArtifactRaw;
pub use raw_names::NamesRaw;
