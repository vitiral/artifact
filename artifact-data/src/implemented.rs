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
//! #SPC-data-src
//!
//! This module defines the types and methods necessary for getting the
//! locations where artifacts are implemented in source code.

use dev_prelude::*;
use path_abs::PathAbs;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Encapsulates the implementation state of the artifact
pub enum Impl {
    /// The artifact is "defined as done"
    Done(String),
    /// The artifact is at least partially implemented in code.
    Code(ImplCode),
    /// The artifact is not implemented directly at all
    NotImpl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Encapsulates the implementation state of the artifact in code.
pub struct ImplCode {
    pub primary: Option<CodeLoc>,
    pub secondary: HashMap<SubName, CodeLoc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The location of an artifact reference in code.
pub struct CodeLoc {
    pub file: PathAbs,
    pub line: u64,
    pub col: u64,
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A subname, i.e. `ART-foo.subname`
pub struct SubName {
    raw: String,
    key: String,
}
