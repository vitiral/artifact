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
#![allow(dead_code)]

//! The `ArtifactRaw` type and methods for loading it from files.

use dev_prelude::*;

use std::sync::mpsc::{channel, Sender};
use std::result;
use std::fmt;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml;
use serde_json;
use toml;

use name::Name;
use raw_names::NamesRaw;
use path_abs::PathFile;
use lint;

// TYPES

#[derive(Debug, Fail)]
pub enum LoadError {
    #[fail(display = "{}", msg)] MarkdownError { msg: String },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
/// The representation of an artifact when stripped down and encoded
pub struct ArtifactRaw {
    pub done: Option<String>,
    pub partof: Option<NamesRaw>,
    pub text: Option<TextRaw>,
}

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct TextRaw(pub(crate) String);

impl fmt::Debug for TextRaw {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for TextRaw {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl Serialize for TextRaw {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: error check for invalid markdown lines
        let trimmed = self.0.trim_right();
        if trimmed.contains('\n') {
            // TODO: the performance could be improved a lot here
            let mut out = trimmed.to_string();
            out.push('\n');
            serializer.serialize_str(&out)
        } else {
            serializer.serialize_str(trimmed)
        }
    }
}

impl<'de> Deserialize<'de> for TextRaw {
    fn deserialize<D>(deserializer: D) -> result::Result<TextRaw, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: error check for invalid markdown lines
        let mut s = String::deserialize(deserializer)?;
        string_trim_right(&mut s);
        if s.contains('\n') {
            s.push('\n');
        }
        Ok(TextRaw(s))
    }
}

// ------------------------------
// -- LOAD

/// Load artifacts from a set of files in parallel.
///
/// Any Errors are converted into lints.
pub(crate) fn load_artifacts_raw(
    send_lints: &Sender<lint::Lint>,
    files: &OrderSet<PathFile>,
) -> Vec<(PathFile, OrderMap<Name, ArtifactRaw>)> {
    let (send, artifacts) = channel();
    let par: Vec<_> = files
        .iter()
        .map(|f| (send_lints.clone(), send.clone(), f.clone()))
        .collect();

    par.into_par_iter()
        .map(|(lints, send, file)| load_file(&lints, &send, &file))
        // consume the iterator
        .count();

    drop(send);
    artifacts.into_iter().collect()
}

/// Join loaded raw artifacts into a single hashmap and lint against duplicates.
pub(crate) fn join_artifacts_raw(
    lints: &Sender<lint::Lint>,
    mut raw: Vec<(PathFile, OrderMap<Name, ArtifactRaw>)>,
) -> (OrderMap<Name, PathFile>, OrderMap<Name, ArtifactRaw>) {
    let mut files: OrderMap<Name, PathFile> = OrderMap::with_capacity(raw.len());
    let mut artifacts = OrderMap::with_capacity(raw.len());
    for (file, mut arts) in raw.drain(..) {
        for (name, art) in arts.drain(..) {
            if let Some(dup) = files.insert(name.clone(), file.clone()) {
                lints
                    .send(lint::Lint {
                        level: lint::Level::Error,
                        category: lint::Category::ParseArtifactFiles,
                        path: Some(dup.to_path_buf()),
                        line: None,
                        msg: format!(
                            "duplicate name detected: {} in {}",
                            name.as_str(),
                            dup.display()
                        ),
                    })
                    .expect("send dup artifact");
                lints
                    .send(lint::Lint {
                        level: lint::Level::Error,
                        category: lint::Category::ParseArtifactFiles,
                        path: Some(file.to_path_buf()),
                        line: None,
                        msg: format!(
                            "duplicate name detected: {} in {}",
                            name.as_str(),
                            file.display()
                        ),
                    })
                    .expect("send dup artifact");
            }

            artifacts.insert(name, art);
        }
    }

    (files, artifacts)
}

/// Load artifacts from a file.
///
/// Any Errors are converted into lints.
fn load_file(
    lints: &Sender<lint::Lint>,
    send: &Sender<(PathFile, OrderMap<Name, ArtifactRaw>)>,
    file: &PathFile,
) {
    let ty = match FileType::from_path(file.as_path()) {
        Some(t) => t,
        None => panic!("An invalid filetype reached this code: {}", file.display()),
    };

    let text = match file.read_string() {
        Ok(t) => t,
        Err(err) => {
            lint::io_error(lints, file.as_path(), &err.to_string());
            return;
        }
    };

    let r: ::std::result::Result<OrderMap<Name, ArtifactRaw>, String> = match ty {
        FileType::Toml => toml::from_str(&text).map_err(|e| e.to_string()),
        FileType::Md => from_markdown(text.as_bytes()).map_err(|e| e.to_string()),
        FileType::Json => serde_json::from_str(&text).map_err(|e| e.to_string()),
    };

    match r {
        Ok(raw) => send.send((file.clone(), raw)).expect("send raw artifact"),
        Err(err) => lint::io_error(lints, file.as_path(), &err.to_string()),
    }
}

// ------------------------------
// -- MARKDOWN

// READ MARKDOWN

lazy_static!{
    pub(crate) static ref NAME_LINE_RE: Regex = Regex::new(
        &format!(r"(?i)^#\s*({})\s*$", ::name::NAME_VALID_STR)).unwrap();

    pub(crate) static ref ATTRS_END_RE: Regex = Regex::new(r"^###+\s*$").unwrap();
}

/// #SPC-data-raw-markdown
/// Load raw artifacts from a markdown stream
pub(crate) fn from_markdown<R: Read>(stream: R) -> Result<OrderMap<Name, ArtifactRaw>> {
    let mut out: OrderMap<Name, ArtifactRaw> = OrderMap::new();
    let mut name: Option<Name> = None;
    let mut attrs: Option<String> = None;
    let mut other: Vec<String> = Vec::new();

    for line_maybe in BufReader::new(stream).lines() {
        let line = line_maybe?;
        if let Some(mat) = NAME_LINE_RE.captures(&line) {
            // We found a new name, `other` is clearly text (if it exists at all
            if let Some(n) = name.take() {
                // Put a new artifact.
                // Use `take()` for name and attrs so that they end up empty
                insert_from_parts(&mut out, &n, attrs.take(), &other)?;
            } else {
                // Ignore text above the first artifact
                attrs = None;
            }
            debug_assert!(name.is_none());
            debug_assert!(attrs.is_none());
            other.clear();
            name = Some(Name::from_str(mat.get(1).unwrap().as_str())?);
            continue;
        } else if ATTRS_END_RE.is_match(&line) {
            // the `other` lines we have been collecting are attrs!
            if name.is_some() && attrs.is_some() {
                let e = LoadError::MarkdownError {
                    msg: format!("`###+\\s+` exists twice under {}", name.unwrap().as_str()),
                };
                return Err(e.into());
            }
            attrs = Some(other.join("\n"));
            other.clear();
            continue;
        }
        // Note: the below should be in an`else` block but the borrow checker is bad at this...
        other.push(line)
    }
    if let Some(name) = name {
        insert_from_parts(&mut out, &name, attrs, &other)?;
    }
    Ok(out)
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
/// For deserializing markdown
struct AttrsRaw {
    pub done: Option<String>,
    pub partof: Option<NamesRaw>,
}

/// Inserts the artifact based on parts gotten from markdown.
fn insert_from_parts(
    out: &mut OrderMap<Name, ArtifactRaw>,
    name: &Name,
    attrs: Option<String>,
    other: &[String],
) -> Result<()> {
    let (done, partof) = match attrs {
        Some(s) => {
            let a: AttrsRaw = serde_yaml::from_str(&s)?;
            (a.done, a.partof)
        }
        None => (None, None),
    };

    let text = {
        let mut t = other.join("\n");
        string_trim_right(&mut t);
        if t.is_empty() {
            None
        } else {
            if t.contains('\n') {
                t.push('\n');
            }
            Some(TextRaw(t))
        }
    };

    let art = ArtifactRaw {
        done: done,
        partof: partof,
        text: text,
    };
    if out.insert(name.clone(), art).is_some() {
        let e = LoadError::MarkdownError {
            msg: format!("name exists twice: {}", name.as_str()),
        };
        Err(e.into())
    } else {
        Ok(())
    }
}

// WRITE MARKDOWN

/// Convert the artifacts to markdown
pub(crate) fn to_markdown(raw_artifacts: &OrderMap<Name, ArtifactRaw>) -> String {
    let mut out = String::new();
    for (name, raw) in raw_artifacts {
        push_artifact_md(&mut out, name, raw);
    }
    // No newlines at end of file.
    string_trim_right(&mut out);
    out
}

fn to_yaml<S: ::serde::Serialize>(value: &S) -> String {
    let mut s = serde_yaml::to_string(value).unwrap();
    s.drain(0..4); // remove the ---\n
    s
}

/// Push a single artifact onto the document
fn push_artifact_md(out: &mut String, name: &Name, raw: &ArtifactRaw) {
    write!(out, "# {}\n", name.as_str()).unwrap();

    // push attrs if they exist
    if raw.done.is_some() || raw.partof.is_some() {
        push_attrs(out, raw);
    }

    // push text if it exists
    if let Some(ref text) = raw.text {
        out.push_str(text);
    }

    // The end of an artifact is always EXACTLY two blank lines
    string_trim_right(out);
    out.push_str("\n\n\n")
}

fn push_attrs(out: &mut String, raw: &ArtifactRaw) {
    if let Some(ref done) = raw.done {
        write!(out, "{}\n\n", to_yaml(&hashmap!{"done" => done})).unwrap();
    }
    if let Some(ref partof) = raw.partof {
        // do `partof` special so it looks prettier
        write!(out, "partof:").unwrap();
        if partof.is_empty() {
            panic!("partof is not None but has no length");
        } else if partof.len() == 1 {
            let n = partof.iter().next().unwrap();
            write!(out, " {}", n.as_str()).unwrap();
        } else {
            write!(out, "\n").unwrap();
            let mut partof = partof.iter().cloned().collect::<Vec<_>>();
            partof.sort();
            for n in &partof {
                write!(out, "- {}\n", n.as_str()).unwrap();
            }
        }
    }
    string_trim_right(out);
    out.push_str("\n###\n");
}

// ------------------------------
// -- INTERNAL STUFF

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FileType {
    Toml,
    Md,
    Json,
}

impl FileType {
    pub(crate) fn from_path<P: AsRef<Path>>(path: P) -> Option<FileType> {
        match path.as_ref().extension() {
            Some(e) => {
                if e == OsStr::new("toml") {
                    Some(FileType::Toml)
                } else if e == OsStr::new("md") {
                    Some(FileType::Md)
                } else if e == OsStr::new("json") {
                    Some(FileType::Json)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[test]
fn sanity_filetype() {
    assert_eq!(
        FileType::from_path(Path::new("/foo/bar.toml")),
        Some(FileType::Toml)
    );
    assert_eq!(
        FileType::from_path(Path::new("this-is-it.md")),
        Some(FileType::Md)
    );
    assert_eq!(
        FileType::from_path(Path::new("/what.json")),
        Some(FileType::Json)
    );
    assert_eq!(FileType::from_path(Path::new("noext")), None);
    assert_eq!(FileType::from_path(Path::new("other.ext")), None);
}
