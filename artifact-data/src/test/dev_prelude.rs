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
pub use ergo::rand::{self, Rng};
use regex_generate;
use unicode_segmentation::UnicodeSegmentation;

use ergo::serde::{Deserialize, Serialize};

pub type StrResult<T> = result::Result<T, String>;

/// Pattern for generating a random string
pub const RNG_LINE_PAT: &str = r#"(?x)
    [-.\ \\/\(\)\[\]!@\#$%^&*A-Za-z0-9]{1,32}
"#;

lazy_static!{
    pub static ref ARTIFACT_DATA_PATH: PathAbs = PathAbs::new(
            PathAbs::new(file!())
                .unwrap() // crate/src/test/dev_prelude.rs
                .parent()
                .unwrap() // crate/src/test
                .parent()
                .unwrap() // crate/src
                .parent()
                .unwrap() // crate/
            ).unwrap();
    pub static ref INTEROP_TESTS_PATH: PathAbs = PathAbs::new(
        ARTIFACT_DATA_PATH.join("interop_tests")).unwrap();
}

/// Given list of `(input, expected)`, assert `method(input) == expected
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
    ::ergo::toml::from_str(s).map_err(|e| e.to_string())
}

pub fn to_toml_string<T: Serialize>(value: &T) -> String {
    ::ergo::toml::to_string(value).expect("failed ser")
}

pub fn from_json_str<'a, T: Deserialize<'a>>(s: &'a str) -> StrResult<T> {
    ::ergo::json::from_str(s).map_err(|e| e.to_string())
}

pub fn to_json_string<T: Serialize>(value: &T) -> String {
    ::ergo::json::to_string(value).expect("failed ser")
}

pub fn from_markdown_str(s: &str) -> StrResult<IndexMap<Name, ::raw::ArtifactRaw>> {
    ::raw::from_markdown(s.as_bytes()).map_err(|e| e.to_string())
}

/// Do a serialization/deserialization roundtrip assertion.
///
/// Return the resulting serialized string.
pub fn serde_roundtrip<T, De, Ser>(name: &str, de: De, ser: Ser, value: &T) -> StrResult<String>
where
    T: Debug + PartialEq,
    De: Fn(&str) -> StrResult<T>,
    Ser: Fn(&T) -> String,
{
    let raw = ser(value);
    let result = match de(&raw) {
        Ok(n) => n,
        Err(e) => return Err(format!("Roundtrip failed: {}", e)),
    };

    if result != *value {
        println!(
            "{:#<30}\n## roundtrip failed in {}:\n{}",
            "#",
            name,
            Comparison::new(&result, value)
        );
        return Err("roundtrip failed".to_string());
    }
    Ok(raw)
}

// RANDOM GENERATION

/// Generate random lines of text, where each line is separated into unicode 'words'
pub fn random_lines<R: Rng + Clone>(rng: &mut R) -> Vec<Vec<String>> {
    let num_lines = rng.gen_range(0, 10);
    let mut r = rng.clone();
    let mut textgen = regex_generate::Generator::parse(RNG_LINE_PAT, rng).unwrap();
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut buffer = Vec::with_capacity(100);
    for _ in 0..num_lines {
        if r.next_f32() < 0.2 {
            // 20% chance of blank line
            out.push(vec!["".to_string()]);
            continue;
        }
        buffer.clear();
        textgen.generate(&mut buffer).unwrap();
        let line: Vec<String> = str::from_utf8(&buffer)
            .unwrap()
            .unicode_words()
            .map(|s| s.to_string())
            .collect();
        out.push(line)
    }
    out
}

/// Insert a word ing into a random place in lines
pub fn insert_word<R: Clone + Rng>(rng: &mut R, lines: &mut Vec<Vec<String>>, word: String) {
    // We need a line to edit
    if lines.is_empty() {
        lines.push(vec!["".to_string()]);
    }
    let edit_line = rng.gen_range(0, lines.len());
    let line = lines.get_mut(edit_line).unwrap();
    let insert_index = rng.gen_range(0, line.len() + 1);
    line.insert(insert_index, word);
}

/// Return the formatted full name string.
///
/// TODO: move this to name.rs?
pub fn name_ref_string(name: &Name, sub: &Option<SubName>) -> String {
    let sub_str = match *sub {
        Some(ref s) => s.raw.as_str(),
        None => "",
    };
    format!("{}{}", name.as_str(), sub_str)
}
