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


use std::path::MAIN_SEPARATOR;
use strfmt;

use dev_prefix::*;
use types::*;

lazy_static! {
    static ref MAIN_SEPARATOR_STR: String = MAIN_SEPARATOR.to_string();
}

/// perform the strfmt, converting the error
pub fn do_strfmt(s: &str, vars: &HashMap<String, String>, fpath: &Path) -> Result<String> {
    strfmt::strfmt(s, vars).chain_err(|| format!("ERROR at {}: {}", fpath.display(), s.to_string()))
}

/// Hacky: convert the path to a string... raising an error if it doesn't.
/// we don't yet support non-unicode (weird windows) paths.
pub fn get_path_str(path: &Path) -> Result<&str> {
    match path.to_str() {
        Some(p) => Ok(p),
        None => Err(ErrorKind::InvalidUnicode(format!("{}", path.display())).into()),
    }
}

/// in windows we need to convert raw path strings
/// to use the correct separator
pub fn convert_path_str(path: &str) -> String {
    path.replace("/", &MAIN_SEPARATOR_STR)
}

#[test]
#[cfg(windows)]
/// assert that convert works for windows paths
fn test_convert_windows() {
    let expected = "this\\is\\a\\windows\\path";
    assert_eq!(expected, convert_path_str("this/is/a/windows/path"));
}

#[test]
#[cfg(not(windows))]
/// assert that convert does nothing
fn test_convert_posix() {
    let expected = "this/is/a/unix/path";
    assert_eq!(expected, convert_path_str(expected));
}
