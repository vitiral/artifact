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

use regex::Regex;

use dev_prelude::*;
use name::{Name, NameError, SubName};
use path_abs::PathAbs;

/// base definition of a valid name. Some pieces may ignore case.
pub const NAME_VALID_STR: &'static str = concat!(
    r"(?:REQ|SPC|TST)-(?:[",
    NAME_VALID_CHARS!(),
    r"]+-)*(?:[",
    NAME_VALID_CHARS!(),
    r"]+)",
);

lazy_static!{
    /// Name reference that can exist in source code
    static ref SRC_NAME_RE: Regex = Regex::new(
        &format!(r#"(?xi)
        \#(                 # start main section
        (?:SPC|TST)         # only SPC or TST
        -(?:[{0}]+-)*       # first section
        (?:[{0}]+)          # (optional) additional sections
        )                   # end main section
        (\.[{0}]+)?         # (optional) sub section
        "#, NAME_VALID_CHARS!())).unwrap();
}

/// Read from the stream, returning parsed location references
pub fn parse_locations<R: Read>(stream: R) -> ::std::io::Result<Vec<(u64, Name, Option<SubName>)>> {
    let mut out: Vec<(u64, Name, Option<SubName>)> = Vec::new();
    for (line_num, line_maybe) in BufReader::new(stream).lines().enumerate() {
        let line = line_maybe?;
        for captures in SRC_NAME_RE.captures_iter(&line) {
            // unwrap: group 1 always exists in regex
            let name_mat = captures.get(1).unwrap();
            // unwrap: pre-validated by regex
            let name = Name::from_str(name_mat.as_str()).unwrap();
            // subname is optional
            let subname = match captures.get(2) {
                Some(sub_mat) => Some(SubName::new_unchecked(sub_mat.as_str())),
                None => None,
            };
            out.push((line_num as u64, name, subname));
        }
    }
    Ok(out)
}

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
}

