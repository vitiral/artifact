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
//! # Serialized Types
//! These are types that have been serialized as STFU8 and are editable by hand.
//!
//! These objects can be safely created by deserializing directly from the corresponding
//! `json::to_string(v)`
//!
//! Used mostly in the frontend or in places where path deserialization is not needed.

extern crate base64;
#[macro_use]
extern crate expect_macro;
#[macro_use]
extern crate ergo_std;
extern crate ergo_config;
#[macro_use]
extern crate derive_error;
// TODO: move to path_abs
pub use std::string::ToString;
pub use std::str::FromStr;

use std::fmt;
use std::result;
use std::error;

#[macro_use]
mod name;
mod dev_prelude;
pub mod lint;
mod ser;
#[macro_use]
mod family;
mod expand_names;

pub use name::{parse_subnames, InternalSubName, Name, SubName, Type, NAME_VALID_STR};
pub use ser::{ArtifactImSer, ArtifactSer, CodeLocSer, ImplCodeSer, ImplSer, ProjectPathsSer,
              ProjectSer, ProjectResultSer};
pub use family::{auto_partofs, Names};
pub use expand_names::expand_names;
pub use lint::Categorized;

use dev_prelude::*;

/// The type used for unique hash ids
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashIm(pub [u8; 16]);

impl fmt::Display for HashIm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", expect!(json::to_string(&self)))
    }
}

impl fmt::Debug for HashIm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
/// #SPC-read-structures.completed
pub struct Completed {
    /// The specification completion ratio.
    pub spc: f32,
    /// The tested completion ratio.
    pub tst: f32,
}

impl fmt::Display for Completed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        json::from_str::<$to>(&json::to_string(&$from).unwrap())
    }
}

