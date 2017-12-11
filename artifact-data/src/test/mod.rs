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
//! Artifact Data Test Harness
//!
//! This module defines the tests for artifact data and also exports
//! functions for other libraries to use to make testing artifact
//! easier.

mod dev_prelude;
mod name;
mod family;
mod raw;
mod raw_names;

pub use test::dev_prelude::assert_generic;
pub use test::family::arb_names;
pub use test::name::{arb_name, arb_name_string, names_raw};
pub use test::raw_names::arb_names_raw;
