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
//! # Serialized Types
//! These are types that have been serialized as STFU8 and are editable by hand.
//!
//! These objects can be safely created by deserializing directly from the corresponding
//! `json::to_string(v)`
//!
//! Used mostly in the frontend or in places where path deserialization is not needed.

use base64;
#[macro_use]
extern crate expect_macro;
extern crate ergo_std;

#[macro_use]
extern crate derive_error;

// TODO: move to path_abs
pub use std::str::FromStr;
pub use std::string::ToString;

use std::error;
use std::fmt;
use std::result;

#[macro_use]
pub mod name;
mod dev_prelude;
pub mod lint;
mod ser;
#[macro_use]
mod family;
mod expand_names;
pub mod markdown;
pub mod md_graph;

pub use crate::expand_names::expand_names;
pub use crate::family::{auto_partofs, Names};
pub use crate::lint::Categorized;
pub use crate::name::{parse_subnames, InternalSubName, Name, SubName, Type, NAME_VALID_STR};
pub use crate::ser::{
    ArtifactImSer, ArtifactOpSer, ArtifactSer, CodeLocSer, ImplCodeSer, ImplSer, ProjectInitialSer,
    ProjectResultSer, ProjectSer, SettingsSer, WebType,
};

use crate::dev_prelude::*;

// ------ SETTINGS ------

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsExport {
    #[serde(default)]
    /// User definable header to include in the exported markdown
    pub md_header: Option<String>,

    #[serde(default = "return_true")]
    /// Whether to include a table of contents
    pub md_toc: bool,

    #[serde(default)]
    /// Settings related to rendering the "family" (graph, list, etc)
    pub md_family: SettingsMdFamily,

    /// How to handle formatting dot
    #[serde(default)]
    pub md_dot: SettingsMdDot,
}

fn return_true() -> bool {
    true
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum SettingsMdFamily {
    List,
    Dot,
}

impl Default for SettingsMdFamily {
    fn default() -> Self {
        SettingsMdFamily::List
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SettingsMdDot {
    /// Do nothing special (leave as-is)
    Ignore,

    /// Remove the braces, will be handled by another processor
    RemoveBraces,

    /// Replace the outer braces
    ReplaceBraces { pre: String, post: String },
}

impl Default for SettingsMdDot {
    fn default() -> Self {
        SettingsMdDot::Ignore
    }
}

// ------ HASH ------

/// The type used for unique hash ids
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashIm(pub [u8; 16]);

impl fmt::Display for HashIm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", expect!(json::to_string(&self)))
    }
}

impl fmt::Debug for HashIm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Default for HashIm {
    fn default() -> HashIm {
        HashIm([0; 16])
    }
}

impl Serialize for HashIm {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let b64 = base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD);
        serializer.serialize_str(&b64)
    }
}

impl<'de> Deserialize<'de> for HashIm {
    fn deserialize<D>(deserializer: D) -> result::Result<HashIm, D::Error>
    where
        D: Deserializer<'de>,
    {
        let b64 = String::deserialize(deserializer)?;
        let mut hash = [0_u8; 16];
        base64::decode_config_slice(&b64, base64::URL_SAFE_NO_PAD, &mut hash)
            .map_err(serde::de::Error::custom)?;
        Ok(HashIm(hash))
    }
}

// ------ COMPLETED ------

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Copy, Serialize, Deserialize)]
pub struct Completed {
    /// The specification completion ratio.
    pub spc: f32,
    /// The tested completion ratio.
    pub tst: f32,
}

impl fmt::Display for Completed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "spc={:.2}, tst={:.2}", self.spc, self.tst)
    }
}

impl Completed {
    /// Used to determine the color.
    ///
    /// See SPC-cli-ls.color_spc
    pub fn spc_points(&self) -> u8 {
        if self.spc >= 1.0 {
            3
        } else if self.spc >= 0.7 {
            2
        } else if self.spc >= 0.4 {
            1
        } else {
            0
        }
    }

    /// Used to determine the color.
    ///
    /// See SPC-cli-ls.color_tst
    pub fn tst_points(&self) -> u8 {
        if self.tst >= 1.0 {
            2
        } else if self.tst >= 0.5 {
            1
        } else {
            0
        }
    }
}

// ------ MODIFY ERROR ------

#[derive(Debug)]
pub struct ModifyError {
    pub lints: lint::Categorized,
    pub kind: ModifyErrorKind,
}

impl error::Error for ModifyError {
    fn description(&self) -> &str {
        "error while modifying an artifact project"
    }
}

impl fmt::Display for ModifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModifyErrorKind: {:?}\n", self.kind)?;
        write!(f, "{}", self.lints)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModifyErrorKind {
    /// Project was corrupted by the user
    InvalidFromLoad,

    /// Some of the operations have invalid paths
    InvalidPaths,

    /// Some of the hash ids did not match
    HashMismatch,

    /// The project would have been corrupted by the modifications
    InvalidFromModify,

    /// Failure while creating, recovery required.
    CreateBackups,

    /// Failure while saving the project, recovery required.
    SaveProject,
}

impl ModifyErrorKind {
    pub fn from_str(s: &str) -> Option<ModifyErrorKind> {
        let out = match s {
            "InvalidFromLoad" => ModifyErrorKind::InvalidFromLoad,
            "InvalidPaths" => ModifyErrorKind::InvalidPaths,
            "HashMismatch" => ModifyErrorKind::HashMismatch,
            "InvalidFromModify" => ModifyErrorKind::InvalidFromModify,
            "CreateBackups" => ModifyErrorKind::CreateBackups,
            "SaveProject" => ModifyErrorKind::SaveProject,
            _ => return None,
        };
        Some(out)
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            ModifyErrorKind::InvalidFromLoad => "InvalidFromLoad",
            ModifyErrorKind::InvalidPaths => "InvalidPaths",
            ModifyErrorKind::HashMismatch => "HashMismatch",
            ModifyErrorKind::InvalidFromModify => "InvalidFromModify",
            ModifyErrorKind::CreateBackups => "CreateBackups",
            ModifyErrorKind::SaveProject => "SaveProject",
        }
    }
}

impl fmt::Display for ModifyErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ------ API OBJECTS ------

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
/// API modification method
pub enum Method {
    ReadProject,
    ModifyProject,
}

impl Method {
    pub fn from_str(s: &str) -> Option<Method> {
        let out = match s {
            "ReadProject" => Method::ReadProject,
            "ModifyProject" => Method::ModifyProject,
            _ => return None,
        };
        Some(out)
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            Method::ReadProject => "ReadProject",
            Method::ModifyProject => "ModifyProject",
        }
    }
}

/// Optional parameters for the `ReadProject` method.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct ParamsReadProject {
    /// Force the backend to reread/reload the artifacts.
    pub reload: bool,
}

// ------ HELPERS ------

/// Inplace trim is annoyingly not in the stdlib
pub fn string_trim_right(s: &mut String) {
    let end = s.trim_right().len();
    s.truncate(end);
}

/// "clean" the text so that it can be serialized/deserialized to/from any of the supported
/// formats.
pub fn clean_text(s: &mut String) {
    string_trim_right(s);
    if s.contains('\n') {
        s.push('\n');
    }
}

/// #SPC-read-family.deauto
/// Strip the automatic family from the `partof` set.
pub fn strip_auto_partofs(name: &Name, names: &mut IndexSet<Name>) {
    if let Some(p) = name.parent() {
        names.remove(&p);
    }
    if let Some(p) = name.auto_partof() {
        names.remove(&p);
    }
}

#[macro_export]
/// Perform a round-trip serialization into the type requested.
///
/// Particularily useful for creating `*Ser` types from their corresponding type.
macro_rules! round_ser {
    [$to:ty, $from:expr] => {
        json::from_str::<$to>(&expect!(json::to_string(&$from)))
    }
}
