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
//! The `ArtifactRaw` type and methods for loading it from files.

use dev_prelude::*;

use std::result;
use std::fmt;
use regex::Regex;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml;
use name::Name;
use family::Names;
use raw_names::NamesRaw;

// TYPES

impl Names {
    /// Names and NamesRaw are equivalent
    pub fn into_names_raw(self) -> NamesRaw {
        NamesRaw { inner: self.0 }
    }
}

impl NamesRaw {
    /// Names and NamesRaw are equivalent
    pub fn into_names(self) -> Names {
        Names(self.inner)
    }
}

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
// -- MARKDOWN

// READ MARKDOWN

lazy_static!{
    pub static ref NAME_LINE_RE: Regex = Regex::new(
        &format!(r"(?i)^#\s*({})\s*$", ::name::NAME_VALID_STR)).unwrap();

    pub static ref ATTRS_END_RE: Regex = Regex::new(r"^###+\s*$").unwrap();
}

/// #SPC-data-raw-markdown
/// Load raw artifacts from a markdown stream
pub fn from_markdown<R: Read>(stream: R) -> Result<BTreeMap<Name, ArtifactRaw>> {
    let mut out: BTreeMap<Name, ArtifactRaw> = BTreeMap::new();
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
    out: &mut BTreeMap<Name, ArtifactRaw>,
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

fn to_yaml<S: ::serde::Serialize>(value: &S) -> String {
    let mut s = serde_yaml::to_string(value).unwrap();
    s.drain(0..4); // remove the ---\n
    s
}

pub fn to_markdown(raw_artifacts: &BTreeMap<Name, ArtifactRaw>) -> String {
    let mut out = String::new();
    for (name, raw) in raw_artifacts {
        push_artifact_md(&mut out, name, raw);
    }
    // No newlines at end of file.
    string_trim_right(&mut out);
    out
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
