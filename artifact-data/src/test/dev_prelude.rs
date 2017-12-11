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
pub use dev_prelude::*;
pub use proptest::prelude::*;
pub use pretty_assertions::Comparison;
pub use itertools::Itertools;
use name::Name;
use serde::{Serialize, Deserialize};

pub type StrResult<T> = result::Result<T, String>;

// TODO: Given list of `(input, expected)`, assert `method(input) == expected
pub fn assert_generic<F, I, E>(method: F, values: &[(I, Option<E>)])
where
    F: Fn(&I) -> StrResult<E>,
    I: Debug,
    E: Debug + Clone + Eq,
{
    let errors = values
        .iter()
        .filter_map(|&(ref inp, ref expected)| {
            let result = method(inp);
            match (result, expected) {
                (Err(_), &None) => None, // error as expected
                (Err(e), &Some(_)) => Some(format!("Expected value but got error: {}", e)),
                (Ok(r), &None) => Some(format!("Expected error but got: {:?}", r)),
                (Ok(ref r), &Some(ref e)) => {
                    if r == e {
                        None // equal as expected
                    } else {
                        Some(format!(
                            "## ERROR input={:?} expected != result:\n{}",
                            inp,
                            Comparison::new(r, e),
                        ))
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        let errors = errors.join("\n");
        panic!("The method had unexpected results:\n{}", errors);
    }
}

pub fn from_toml_str<'a, T: Deserialize<'a>>(s: &'a str) -> StrResult<T> {
    ::toml::from_str(s)
        .map_err(|e| e.to_string())
}

pub fn to_toml_string<T: Serialize>(value: &T) -> String {
    ::toml::to_string(value).expect("failed ser")
}

pub fn from_json_str<'a, T: Deserialize<'a>>(s: &'a str) -> StrResult<T> {
    ::serde_json::from_str(s)
        .map_err(|e| e.to_string())
}

pub fn to_json_string<T: Serialize>(value: &T) -> String {
    ::serde_json::to_string(value).expect("failed ser")
}

pub fn from_markdown_str(s: &str) -> StrResult<BTreeMap<Name, ::raw::ArtifactRaw>> {
    ::raw::from_markdown(s.as_bytes())
        .map_err(|e| e.to_string())
}

/// Do a serialization/deserialization roundtrip assertion.
///
/// Return the resulting serialized string.
pub fn serde_roundtrip<T, De, Ser>(de: De, ser: Ser, value: &T) -> StrResult<String>
    where T: Debug+PartialEq,
          De: Fn(&str) -> StrResult<T>,
          Ser: Fn(&T) -> String
{
    let raw = ser(value);
    let result = match de(&raw) {
        Ok(n) => n,
        Err(e) => return Err(format!("Roundtrip failed: {}", e)),
    };

    if result != *value {
        return Err(format!(
            "roundtrip failed:\n{}",
            Comparison::new(&result, value)
        ));
    }
    Ok(raw)
}
