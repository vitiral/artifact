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
#![allow(dead_code)]

use regex::Regex;
use std::collections::hash_map::Entry;

use dev_prelude::*;
use name::{Name, NameError, SubName};
use lint;
use path_abs::PathAbs;

lazy_static!{
    /// Name reference that can exist in source code
    static ref SRC_NAME_RE: Regex = Regex::new(
        &format!(r#"(?xi)
        \#(                 # start main section
        (?:REQ|SPC|TST)     # all types are supported
        -(?:[{0}]+-)*       # first section
        (?:[{0}]+)          # (optional) additional sections
        )                   # end main section
        (\.[{0}]+)?         # (optional) sub section
        "#, NAME_VALID_CHARS!())).unwrap();
}

/// #SPC-data-src.load
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

/// #SPC-data-src.join
/// Consume the parsed locations, returning the combined implementation objects.
///
/// This also lints against duplicates.
pub fn join_locations(
    send_lints: &::std::sync::mpsc::Sender<lint::Lint>,
    locations: &[(PathAbs, Vec<(u64, Name, Option<SubName>)>)],
) -> HashMap<Name, ImplCode> {
    let mut primary_locs: HashMap<Name, CodeLoc> = HashMap::new();
    let mut secondary_locs: HashMap<Name, HashMap<SubName, CodeLoc>> = HashMap::new();
    // internal helper method for constructing lints
    let duplicate_detected = |path, line, msg| {
        send_lints.send(lint::Lint {
            category: lint::Category::ParseCodeImplementations,
            path: Some(path),
            line: Some(line),
            msg: lint::Msg::Error(format!("duplicate detected: {}", msg)),
        });
    };
    // get all locs and secondary_locs, simultaniously checking there are no duplicates
    for &(ref file, ref locs) in locations.iter() {
        for &(line, ref name, ref sub) in locs {
            let loc = CodeLoc::new(&file, line);
            if let &Some(ref s) = sub {
                match secondary_locs.entry(name.clone()) {
                    Entry::Occupied(mut entry) => {
                        let e = entry.get_mut();
                        if let Some(orig) = e.insert(s.clone(), loc) {
                            duplicate_detected(
                                orig.file.clone(),
                                orig.line,
                                format!("{}{}", name.as_str(), s.as_str()),
                            );
                            duplicate_detected(
                                file.clone(),
                                line,
                                format!("{}{}", name.as_str(), s.as_str()),
                            );
                        }
                    }
                    Entry::Vacant(mut entry) => {
                        entry.insert(hashmap!{s.clone() => loc});
                    }
                }
            } else {
                if let Some(orig) = primary_locs.insert(name.clone(), loc) {
                    duplicate_detected(orig.file.clone(), orig.line, name.as_str().to_string());
                    duplicate_detected(file.clone(), line, name.as_str().to_string());
                }
            }
        }
    }

    let empty_hash = HashMap::with_capacity(0);
    let mut out: HashMap<Name, ImplCode> =
        HashMap::from_iter(primary_locs.drain().map(|(name, loc)| {
            let code = ImplCode {
                primary: Some(loc),
                secondary: secondary_locs
                    .remove(&name)
                    .unwrap_or_else(|| empty_hash.clone()),
            };

            (name, code)
        }));

    for (name, secondary) in secondary_locs.drain() {
        // We removed secondary_locs while constructing from primary_locs,
        // so these names are always without a primary.
        // (i.e. no need to Entry API)
        out.insert(
            name,
            ImplCode {
                primary: None,
                secondary: secondary,
            },
        );
    }
    out
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

impl CodeLoc {
    pub fn new(file: &PathAbs, line: u64) -> CodeLoc {
        CodeLoc {
            file: file.clone(),
            line: line,
        }
    }
}
