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

pub use ergo::*;
#[allow(unused_imports)]
pub use expect_macro::*;
// TODO: move these to std_prelude
pub use std::ffi::OsStr;
pub use std::cmp::Ord;
pub use std::cmp::PartialOrd;
pub use std::hash::{Hash, Hasher};
use std::io;
use std::fs;
pub use artifact_lib::*;
pub use artifact_lib;
pub use artifact_data;

pub use indexmap::{IndexMap, IndexSet};

pub use std::result;
pub use failure::Error;

pub type Result<V> = result::Result<V, Error>;

// FROM DATA.TEST
pub use proptest::prelude::*;
pub use pretty_assertions::Comparison;
pub use ergo::rand::{self, Rng};

pub use super::raw::ArtifactRawExt;

use regex_generate;
use unicode_segmentation::UnicodeSegmentation;

use ergo::serde::{Deserialize, Serialize};

pub type StrResult<T> = result::Result<T, String>;

/// Pattern for generating a random string
pub const RNG_LINE_PAT: &str = r#"(?x)
    [-.\ \\/\(\)\[\]!@\#$%^&*A-Za-z0-9]{1,32}
"#;

lazy_static!{
    pub static ref ARTIFACT_TEST_PATH: PathAbs = PathAbs::new(
            PathAbs::new(file!())
                .unwrap() // crate/src/dev_prelude.rs
                .parent()
                .unwrap() // crate/src
                .parent()
                .unwrap() // crate/
            ).unwrap();
    pub static ref INTEROP_TESTS_PATH: PathAbs = PathAbs::new(
        ARTIFACT_TEST_PATH.join("interop_tests")).unwrap();
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

pub fn from_markdown_str(s: &str) -> StrResult<IndexMap<Name, artifact_data::raw::ArtifactRaw>> {
    artifact_data::raw::from_markdown(s.as_bytes()).map_err(|e| e.to_string())
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

// FROM DATA

#[allow(dead_code)]
/// A simple implementation of "touch"
pub fn touch<P: AsRef<Path>>(path: P) -> ::std::io::Result<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(path.as_ref())?;
    Ok(())
}

/// Do a deep copy of a directory from one location to another.
pub fn deep_copy<P: AsRef<Path>>(send_err: Sender<io::Error>, from: PathDir, to: P) {
    let to = ch_try!(
        send_err,
        create_dir_maybe(to).map_err(|err| err.into()),
        return
    );

    let (send_file, recv_file) = ch::bounded(128);

    // First thread walks and creates directories, and sends files to copy
    take!(=send_err as errs, =to as to_walk);
    spawn(move || {
        walk_and_create_dirs(from, to_walk, errs, send_file);
    });

    // Threadpool copy files into directories that are pre-created.
    for _ in 0..num_cpus::get() {
        take!(=send_err, =recv_file, =to);
        spawn(move || {
            for (from, to_postfix) in recv_file {
                ch_try!(
                    send_err,
                    from.copy(to.join(to_postfix)).map_err(|err| err.into()),
                    continue
                );
            }
        });
    }
}

fn create_dir_maybe<P: AsRef<Path>>(path: P) -> path_abs::Result<PathDir> {
    let arc = PathArc::new(path);
    fs::create_dir(&arc).map_err(|err| path_abs::Error::new(err, "creating dir", arc.clone()))?;
    PathDir::new(arc)
}

/// Do a contents-first yeild and follow any symlinks -- we are doing an _actual_ copy
fn walk_and_create_dirs(
    from: PathDir,
    to: PathDir,
    send_err: Sender<io::Error>,
    send_file: Sender<(PathFile, PathBuf)>,
) {
    let mut it = from.walk().follow_links(true).into_iter();
    loop {
        let entry = match it.next() {
            Some(entry) => entry,
            None => break,
        };
        macro_rules! handle_err {
            ($entry:expr) => {
                match $entry {
                    Ok(e) => e,
                    Err(err) => {
                        ch!(send_err <- err.into());
                        continue;
                    }
                }
            };
        }
        let entry = handle_err!(entry);
        let to_postfix = expect!(
            entry.path().strip_prefix(&from),
            "{} does not have prefix {}",
            entry.path().display(),
            from.display()
        );
        match handle_err!(PathType::new(entry.path())) {
            PathType::Dir(_) => {
                // Create it immediately
                if let Err(err) = PathDir::create(to.join(to_postfix)) {
                    ch!(send_err <- err.into());
                    // We couldn't create the directory so it needs to be skipped.
                    it.skip_current_dir();
                }
            }
            PathType::File(from_file) => {
                ch!(send_file <- (from_file, to_postfix.to_path_buf()));
            }
        }
    }
}
