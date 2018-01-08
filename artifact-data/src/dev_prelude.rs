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
pub(crate) use std_prelude::*;
// TODO: move these to std_prelude
pub(crate) use std::ffi::OsStr;
use std::cmp::Ord;
use std::hash::Hash;

pub(crate) use ordermap::{OrderMap, OrderSet};

pub(crate) use std::result;
pub(crate) use failure::Error;

pub(crate) type Result<V> = result::Result<V, Error>;

/// Inplace trim is annoyingly not in the stdlib
pub(crate) fn string_trim_right(s: &mut String) {
    let end = s.trim_right().len();
    s.truncate(end);
}

#[allow(dead_code)]
/// A simple implementation of "touch"
pub(crate) fn touch<P: AsRef<Path>>(path: P) -> ::std::io::Result<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(path.as_ref())?;
    Ok(())
}

#[test]
fn sanity_trim_right() {
    let mut result = "  hello    ".into();
    string_trim_right(&mut result);
    assert_eq!(result, "  hello");
}

pub(crate) fn sort_ordermap<K: Ord + Hash, V>(m: &mut OrderMap<K, V>) {
    let mut ordered: Vec<_> = m.drain(..).collect();
    ordered.sort_by(|left, right| left.0.cmp(&right.0));
    m.extend(ordered.drain(..));
}

pub(crate) fn sort_orderset<K: Ord + Hash>(m: &mut OrderSet<K>) {
    let mut ordered: Vec<_> = m.drain(..).collect();
    ordered.sort_by(|left, right| left.cmp(right));
    m.extend(ordered.drain(..));
}
