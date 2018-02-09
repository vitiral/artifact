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
//! Module for defining logic for parsing collapsed artifact names and
//! recollapsing them for tests
//!
//! This feature was added before artifact 1.0 and was purposely preserved.
//! It is *definitely* a "poweruser" feature, but it can come in handy.
//!
//! However, the implementation is not ideal. In particular I would like
//! to use a legitamite parsing library instead of the hand crafted
//! regression parsing seen here. However, I do believe this works.
//!
//! Notes: this is tested in `test_family.rs`

use dev_prelude::*;
use name::{Name, NameError};

/// Expand a string of names into multiple names.
///
/// i.e. `"REQ-[foo, bar]"` -> `["REQ-foo", "REQ-bar"]`
pub(crate) fn expand_names(raw: &str) -> Result<OrderSet<Name>> {
    parse_collapsed(&mut raw.chars(), false)?
        .iter()
        .map(|n| Name::from_str(n))
        .collect()
}

// Private: Expanding Names. Use `Name::from_str`

/// subfunction to parse names from a names-str recusively
fn parse_collapsed<I>(raw: &mut I, in_brackets: bool) -> Result<Vec<String>>
where
    I: Iterator<Item = char>,
{
    // hello-[there, you-[are, great]]
    // hello-there, hello-you-are, hello-you-great
    let mut strout = String::new();
    let mut current = String::new();
    loop {
        let c = match raw.next() {
            Some(c) => c,
            None => {
                if in_brackets {
                    return Err(NameError::InvalidCollapsed {
                        msg: "brackets are not closed".into(),
                    }.into());
                }
                break;
            }
        };
        match c {
            ' ' | '\n' | '\r' => {}
            // ignore whitespace
            '[' => {
                if current == "" {
                    // SPC-names.2: more validation
                    let msg = "cannot have '[' after characters ',' or ']'\
                               or at start of string";
                    return Err(NameError::InvalidCollapsed { msg: msg.into() }.into());
                }
                for p in try!(parse_collapsed(raw, true)) {
                    strout.write_str(&current).unwrap();
                    strout.write_str(&p).unwrap();
                    strout.push(',');
                }
                current.clear();
            }
            ']' => if !in_brackets {
                let err = NameError::InvalidCollapsed {
                    msg: "`]` character wasn't opened".into(),
                };
                return Err(err.into());
            } else {
                break;
            },
            ',' => {
                strout.write_str(&current).unwrap();
                strout.push(',');
                current.clear();
            }
            _ => current.push(c),
        }
    }
    strout.write_str(&current).unwrap();
    Ok(strout
        .split(',')
        .filter(|s| s != &"")
        .map(|s| s.to_string())
        .collect())
}
