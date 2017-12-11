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
// the failure `Error` type
pub use std_prelude::*;

pub use std::result;
pub use failure::{Error, Fail};

pub type Result<V> = result::Result<V, Error>;

/// Inplace trim is annoyingly not in the stdlib
pub fn string_trim_right(s: &mut String) {
    let end = s.trim_right().len();
    s.truncate(end);
}

/// A simple implementation of "touch"
pub fn touch<P: AsRef<Path>>(path: P) -> ::std::io::Result<()> {
    match OpenOptions::new().create(true).write(true).open(path.as_ref()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[test]
fn sanity_trim_right() {
    let mut result = "  hello    ".into();
    string_trim_right(&mut result);
    assert_eq!(result, "  hello");
}
