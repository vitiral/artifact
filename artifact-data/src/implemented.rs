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

use std::fmt;
use indexmap::map::Entry;
use dev_prelude::*;

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
    file: &PathFile,
    send_locs: &Sender<(CodeLoc, Name, Option<SubName>)>,
) {
    if let Err(err) = parse_file(send_locs, file) {
        ch!(send_lints <- lint::Lint::load_error(file, &err.to_string()));
    }
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
pub fn parse_locations<R: Read>(
    send: &Sender<(CodeLoc, Name, Option<SubName>)>,
    file: &PathFile,
    stream: R,
) -> ::std::io::Result<()> {
    for (line_num, line_maybe) in BufReader::new(stream).lines().enumerate() {
        let line = line_maybe?;
        for captures in SRC_NAME_RE.captures_iter(&line) {
            let name_mat = expect!(captures.get(1), "group 1");
            let name = expect!(Name::from_str(name_mat.as_str()), "name pre-validated");
            // subname is optional
            let subname = match captures.get(2) {
                Some(sub_mat) => Some(SubName::new_unchecked(sub_mat.as_str())),
                None => None,
            };
            ch!(send <- (CodeLoc::new(file, line_num as u64), name, subname));
        }
    }
    Ok(())
}

/// #SPC-read-impl.join
/// Consume the parsed locations, returning the combined implementation objects.
///
/// This also lints against duplicates.
pub fn join_locations(
    send_lints: &Sender<lint::Lint>,
    mut locations: Vec<(CodeLoc, Name, Option<SubName>)>,
) -> IndexMap<Name, ImplCode> {
    // split into primary and secondary, simultaniously checking there are no duplicates.
    let mut primary_locs: IndexMap<Name, CodeLoc> = IndexMap::new();
    let mut secondary_locs: IndexMap<Name, IndexMap<SubName, CodeLoc>> = IndexMap::new();
    for (loc, name, sub) in locations.drain(0..) {
        if let Some(sub) = sub {
            insert_secondary(send_lints, &mut secondary_locs, &name, &sub, loc);
        } else if let Some(orig) = primary_locs.insert(name.clone(), loc.clone()) {
            duplicate_detected(send_lints, &orig.file, orig.line, name.as_str());
            duplicate_detected(send_lints, &loc.file, loc.line, name.as_str());
        }
    }

    // Now join them together
    let empty_hash = IndexMap::with_capacity(0);
    let mut out: IndexMap<Name, ImplCode> =
        IndexMap::from_iter(primary_locs.drain(..).map(|(name, loc)| {
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
    locs: &mut IndexMap<Name, IndexMap<SubName, CodeLoc>>,
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
                    &format!("{}{}", name, sub),
                );
                duplicate_detected(
                    send_lints,
                    &loc.file,
                    loc.line,
                    &format!("{}{}", name, sub),
                );
            }
        }
        Entry::Vacant(entry) => {
            entry.insert(indexmap!{sub.clone() => loc});
        }
    }
}

/// internal helper for `join_locations`
fn duplicate_detected(send_lints: &Sender<lint::Lint>, path: &PathFile, line: u64, msg: &str) {
    send_lints
        .send(lint::Lint {
            level: lint::Level::Error,
            category: lint::Category::ParseCodeImplementations,
            path: Some(path.clone().into()),
            line: Some(line),
            msg: format!("duplicate detected: {}", msg),
        })
        .expect("send failed in implemented.rs");
}
