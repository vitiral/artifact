/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
#![allow(dead_code)]

//! The `ArtifactRaw` type and methods for loading it from files.

use dev_prelude::*;

use ergo::serde::{Deserialize, Deserializer, Serialize, Serializer};
use ergo::{json, toml, yaml};
use std::fmt;
use std::result;

use intermediate::ArtifactImExt;
use raw_names::NamesRaw;

// TYPES

#[derive(Debug, Fail)]
pub enum LoadError {
    #[fail(display = "{}", msg)]
    MarkdownError { msg: String },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
/// The representation of an artifact when stripped down and encoded
pub struct ArtifactRaw {
    pub done: Option<String>,
    pub partof: Option<NamesRaw>,
    pub text: Option<TextRaw>,
}

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct TextRaw(pub String);

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
        let mut trimmed = self.0.trim_right().to_string();
        clean_text(&mut trimmed);
        serializer.serialize_str(&trimmed)
    }
}

impl<'de> Deserialize<'de> for TextRaw {
    fn deserialize<D>(deserializer: D) -> result::Result<TextRaw, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut s = String::deserialize(deserializer)?;
        clean_text(&mut s);
        Ok(TextRaw(s))
    }
}

// ------------------------------
// -- LOAD

/// Join loaded raw artifacts into a single hashmap and lint against duplicates.
pub(crate) fn join_artifacts_raw(
    lints: &Sender<lint::Lint>,
    mut art_ims: Vec<ArtifactIm>,
) -> (IndexMap<Name, PathArc>, IndexMap<Name, ArtifactIm>) {
    let mut files: IndexMap<Name, PathArc> = IndexMap::with_capacity(art_ims.len());
    let mut artifacts = IndexMap::with_capacity(art_ims.len());
    for mut art in art_ims.drain(..) {
        if let Some(dup) = files.insert(art.name.clone(), art.file.clone()) {
            lints
                .send(lint::Lint {
                    level: lint::Level::Error,
                    category: lint::Category::ParseArtifactFiles,
                    path: Some(dup.to_string()),
                    line: None,
                    msg: format!("duplicate name detected: {} in {}", art.name, dup.display()),
                })
                .expect("send dup artifact");
            lints
                .send(lint::Lint {
                    level: lint::Level::Error,
                    category: lint::Category::ParseArtifactFiles,
                    path: Some(art.file.to_string()),
                    line: None,
                    msg: format!(
                        "duplicate name detected: {} in {}",
                        art.name,
                        art.file.display()
                    ),
                })
                .expect("send dup artifact");
        }
        art.clean();
        artifacts.insert(art.name.clone(), art);
    }

    (files, artifacts)
}

/// Load artifacts from a file.
///
/// Any Errors are converted into lints.
pub(crate) fn load_file(lints: &Sender<lint::Lint>, send: &Sender<ArtifactIm>, file: &PathFile) {
    let ty = match ArtFileType::from_path(file.as_path()) {
        Some(t) => t,
        None => panic!("An invalid filetype reached this code: {}", file.display()),
    };

    let text = match file.read_string() {
        Ok(t) => t,
        Err(err) => {
            ch!(lints <- lint::Lint::load_error(file.to_string(), &err.to_string()));
            return;
        }
    };

    let r: ::std::result::Result<IndexMap<Name, ArtifactRaw>, String> = match ty {
        ArtFileType::Toml => toml::from_str(&text).map_err(|e| e.to_string()),
        ArtFileType::Md => from_markdown(text.as_bytes()).map_err(|e| e.to_string()),
        ArtFileType::Json => json::from_str(&text).map_err(|e| e.to_string()),
    };

    let mut raw_artifacts = match r {
        Ok(raw) => raw,
        Err(err) => {
            ch!(lints <- lint::Lint::load_error(file.to_string(), &err.to_string()));
            return;
        }
    };
    for (name, raw) in raw_artifacts.drain(..) {
        let art = ArtifactIm::from_raw(name, file.clone(), raw);
        send.send(art).expect("send raw artifact");
    }
}

// ------------------------------
// -- MARKDOWN

// READ MARKDOWN

lazy_static! {
    pub static ref NAME_LINE_RE: Regex =
        Regex::new(&format!(r"(?i)^#\s*({})\s*$", NAME_VALID_STR)).unwrap();
    pub static ref ATTRS_END_RE: Regex = Regex::new(r"^###+\s*$").unwrap();
}

/// #SPC-read-raw-markdown
/// Load raw artifacts from a markdown stream
pub fn from_markdown<R: Read>(stream: R) -> Result<IndexMap<Name, ArtifactRaw>> {
    let mut out: IndexMap<Name, ArtifactRaw> = IndexMap::new();
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
            name = Some(Name::from_str(expect!(mat.get(1)).as_str())?);
            continue;
        } else if ATTRS_END_RE.is_match(&line) {
            // the `other` lines we have been collecting are attrs!
            if name.is_some() && attrs.is_some() {
                let e = LoadError::MarkdownError {
                    msg: format!("`###+\\s+` exists twice under {}", expect!(name)),
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
    out: &mut IndexMap<Name, ArtifactRaw>,
    name: &Name,
    attrs: Option<String>,
    other: &[String],
) -> Result<()> {
    let (done, partof) = match attrs {
        Some(s) => {
            let a: AttrsRaw = yaml::from_str(&s)?;
            (a.done, a.partof)
        }
        None => (None, None),
    };

    let text = {
        let mut t = other.join("\n");
        clean_text(&mut t);
        if t.is_empty() {
            None
        } else {
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
            msg: format!("name exists twice: {}", name),
        };
        Err(e.into())
    } else {
        Ok(())
    }
}

// WRITE MARKDOWN

/// Convert the artifacts to markdown
pub fn to_markdown(raw_artifacts: &IndexMap<Name, ArtifactRaw>) -> String {
    let mut out = String::new();
    for (name, raw) in raw_artifacts {
        push_artifact_md(&mut out, name, raw);
    }
    // No newlines at end of file.
    string_trim_right(&mut out);
    out
}

fn to_yaml<S: Serialize>(value: &S) -> String {
    let mut s = expect!(yaml::to_string(value));
    s.drain(0..4); // remove the ---\n
    s
}

/// Push a single artifact onto the document
fn push_artifact_md(out: &mut String, name: &Name, raw: &ArtifactRaw) {
    expect!(write!(out, "# {}\n", name));

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
        expect!(write!(out, "{}\n\n", to_yaml(&hashmap! {"done" => done})));
    }
    if let Some(ref partof) = raw.partof {
        // do `partof` special so it looks prettier
        expect!(write!(out, "partof:"));
        if partof.is_empty() {
            panic!("partof is not None but has no length");
        } else if partof.len() == 1 {
            let n = expect!(partof.iter().next());
            expect!(write!(out, " {}", n));
        } else {
            expect!(write!(out, "\n"));
            let mut partof = partof.iter().cloned().collect::<Vec<_>>();
            partof.sort();
            for n in &partof {
                expect!(write!(out, "- {}\n", n));
            }
        }
    }
    string_trim_right(out);
    out.push_str("\n###\n");
}

// ------------------------------
// -- INTERNAL STUFF

#[derive(Debug, PartialEq, Eq)]
pub enum ArtFileType {
    Toml,
    Md,
    Json,
}

impl ArtFileType {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<ArtFileType> {
        match path.as_ref().extension() {
            Some(e) => {
                if e == OsStr::new("toml") {
                    Some(ArtFileType::Toml)
                } else if e == OsStr::new("md") {
                    Some(ArtFileType::Md)
                } else if e == OsStr::new("json") {
                    Some(ArtFileType::Json)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
