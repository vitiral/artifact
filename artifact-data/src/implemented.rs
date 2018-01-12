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
//! #SPC-read-impl
//!
//! This module defines the types and methods necessary for getting the
//! locations where artifacts are implemented in source code.
#![allow(dead_code)]

use regex::Regex;
use rayon::prelude::*;
use ordermap::Entry;
use std::sync::mpsc::{channel, Sender};

use dev_prelude::*;
use name::{Name, SubName};
use lint;
use path_abs::PathFile;

// EXPORTED TYPES

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
    pub secondary: OrderMap<SubName, CodeLoc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The location of an artifact reference in code.
pub struct CodeLoc {
    pub file: PathFile,
    pub line: u64,
}

impl CodeLoc {
    pub fn new(file: &PathFile, line: u64) -> CodeLoc {
        CodeLoc {
            file: file.clone(),
            line: line,
        }
    }
}

impl Impl {
    /// Return the `(count, value, secondary_count, secondary_value)`
    /// that this impl should contribute to the "implemented" statistics.
    ///
    /// "secondary" is used because the Done field actually does contribute to
    /// both spc AND tst for REQ and SPC types.
    ///
    /// `subnames` should contain the subnames that exist in that artifact's text
    pub(crate) fn to_statistics(&self, subnames: &OrderSet<SubName>) -> (usize, f64, usize, f64) {
        match *self {
            Impl::Done(_) => (1, 1.0, 1, 1.0),
            Impl::Code(ref impl_) => {
                let mut count = 1;
                let mut value = f64::from(impl_.primary.is_some() as u8);
                for sub in subnames.iter() {
                    count += 1;
                    // add 1 if the subname is implemented, else 0
                    value += f64::from(impl_.secondary.contains_key(sub) as u8);
                }
                (count, value, 0, 0.0)
            }
            Impl::NotImpl => {
                if !subnames.is_empty() {
                    // If subnames are defined not being implemented
                    // in code means that you get counts against you
                    (1 + subnames.len(), 0.0, 0, 0.0)
                } else {
                    (0, 0.0, 0, 0.0)
                }
            }
        }
    }

    /// Return whether this is the `Done` variant.
    pub fn is_done(&self) -> bool {
        match *self {
            Impl::Done(_) => true,
            _ => false,
        }
    }
}

// METHODS

lazy_static!{
    /// Name reference that can exist in source code
    static ref SRC_NAME_RE: Regex = Regex::new(
        &format!(r#"(?xi)
        \#(                 # start main section
        (?:REQ|SPC|TST)     # all types are supported
        -(?:[{0}]+-)*       # any number of first elements
        (?:[{0}]+)          # required end element
        )                   # end main section
        (\.[{0}]+)?         # (optional) sub section
        "#, NAME_VALID_CHARS!())).unwrap();
}

/// Parse the locations from a set of files in parallel
///
/// Any io errors are converted into lint errors instead.
pub(crate) fn load_locations(
    send_lints: &Sender<lint::Lint>,
    files: &OrderSet<PathFile>,
) -> Vec<(CodeLoc, Name, Option<SubName>)> {
    let (send, locations) = channel();
    let par: Vec<_> = files
        .iter()
        .map(|f| (send_lints.clone(), send.clone(), f.clone()))
        .collect();
    drop(send);

    par.into_par_iter()
        .map(|(lints, send, file)| {
            if let Err(err) = parse_file(&send, &file) {
                lint::io_error(&lints, file.as_path(), &err.to_string());
            }
        })
        // consume the iterator
        .count();

    locations.into_iter().collect()
}

/// internal helper to just open a path and parse it
fn parse_file(
    send: &Sender<(CodeLoc, Name, Option<SubName>)>,
    file: &PathFile,
) -> ::std::io::Result<()> {
    let f = File::open(file.as_path())?;
    parse_locations(send, file, f)
}

/// #SPC-read-impl.load
/// Read from the stream, returning parsed location references
pub(crate) fn parse_locations<R: Read>(
    send: &Sender<(CodeLoc, Name, Option<SubName>)>,
    file: &PathFile,
    stream: R,
) -> ::std::io::Result<()> {
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
            send.send((CodeLoc::new(file, line_num as u64), name, subname))
                .expect("failed to send during parse");
        }
    }
    Ok(())
}

/// #SPC-read-impl.join
/// Consume the parsed locations, returning the combined implementation objects.
///
/// This also lints against duplicates.
pub(crate) fn join_locations(
    send_lints: &Sender<lint::Lint>,
    mut locations: Vec<(CodeLoc, Name, Option<SubName>)>,
) -> OrderMap<Name, ImplCode> {
    // split into primary and secondary, simultaniously checking there are no duplicates.
    let mut primary_locs: OrderMap<Name, CodeLoc> = OrderMap::new();
    let mut secondary_locs: OrderMap<Name, OrderMap<SubName, CodeLoc>> = OrderMap::new();
    for (loc, name, sub) in locations.drain(0..) {
        if let Some(sub) = sub {
            insert_secondary(send_lints, &mut secondary_locs, &name, &sub, loc);
        } else if let Some(orig) = primary_locs.insert(name.clone(), loc.clone()) {
            duplicate_detected(send_lints, &orig.file, orig.line, name.as_str());
            duplicate_detected(send_lints, &loc.file, loc.line, name.as_str());
        }
    }

    // Now join them together
    let empty_hash = OrderMap::with_capacity(0);
    let mut out: OrderMap<Name, ImplCode> =
        OrderMap::from_iter(primary_locs.drain(..).map(|(name, loc)| {
            let code = ImplCode {
                primary: Some(loc),
                secondary: secondary_locs
                    .remove(&name)
                    .unwrap_or_else(|| empty_hash.clone()),
            };
            (name, code)
        }));

    for (name, secondary) in secondary_locs.drain(..) {
        // We removed secondary_locs while constructing from primary_locs,
        // so these names are always without a primary.
        // (i.e. no need to use Entry API)
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

/// internal helper for `join_locations`
fn insert_secondary(
    send_lints: &Sender<lint::Lint>,
    locs: &mut OrderMap<Name, OrderMap<SubName, CodeLoc>>,
    name: &Name,
    sub: &SubName,
    loc: CodeLoc,
) {
    match locs.entry(name.clone()) {
        Entry::Occupied(mut entry) => {
            let e = entry.get_mut();
            if let Some(orig) = e.insert(sub.clone(), loc.clone()) {
                duplicate_detected(
                    send_lints,
                    &orig.file,
                    orig.line,
                    &format!("{}{}", name.as_str(), sub.as_str()),
                );
                duplicate_detected(
                    send_lints,
                    &loc.file,
                    loc.line,
                    &format!("{}{}", name.as_str(), sub.as_str()),
                );
            }
        }
        Entry::Vacant(entry) => {
            entry.insert(ordermap!{sub.clone() => loc});
        }
    }
}

/// internal helper for `join_locations`
fn duplicate_detected(send_lints: &Sender<lint::Lint>, path: &PathFile, line: u64, msg: &str) {
    send_lints
        .send(lint::Lint {
            level: lint::Level::Error,
            category: lint::Category::ParseCodeImplementations,
            path: Some(path.to_path_buf()),
            line: Some(line),
            msg: format!("duplicate detected: {}", msg),
        })
        .expect("send failed in implemented.rs");
}
