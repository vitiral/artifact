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
//! #SPC-structs.artifact_im
use std::fmt;
use base64;
use siphasher::sip128::{Hasher128, SipHasher};
use ergo::serde::{self, Deserialize, Deserializer, Serialize, Serializer};

use dev_prelude::*;
use family;
use raw::{self, ArtifactRaw, TextRaw};
use raw_names::NamesRaw;
use artifact::Artifact;
use name::Name;

/// The type used for unique hash ids
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashIm(pub(crate) [u8; 16]);

#[derive(Debug, Serialize, Deserialize)]
/// #SPC-structs.artifact_im
pub struct ArtifactIm {
    pub name: Name,
    pub file: PathArc,
    pub partof: OrderSet<Name>,
    pub done: Option<String>,
    pub text: String,
}

// IMPL ArtifactIm

impl ArtifactIm {
    /// Create the intermediate hash.
    ///
    /// This is the primary key used to ensure consistency when modifying artifats via an API.
    pub fn hash_im(&self) -> HashIm {
        let mut hasher = SipHasher::new();
        self.hash(&mut hasher);
        HashIm(hasher.finish128().as_bytes())
    }

    /// Process the `ArtifactIm`.
    ///
    /// This is required whenever serializing/deserializing the ArtifactIm.
    pub fn clean(&mut self) {
        family::strip_auto_partofs(&self.name, &mut self.partof);
        self.partof.sort();
        raw::clean_text(&mut self.text);
    }

    /// Get an `ArtifactIm` from an `ArtifactRaw`.
    pub(crate) fn from_raw(name: Name, file: PathFile, raw: ArtifactRaw) -> ArtifactIm {
        let mut partof = raw.partof
            .map(|mut p| {
                family::strip_auto_partofs(&name, &mut p.0);
                p.drain(..).collect()
            })
            .unwrap_or_else(OrderSet::new);
        partof.sort();

        ArtifactIm {
            name: name,
            file: file.into(),
            partof: partof,
            done: raw.done,
            text: raw.text.map(|t| t.0).unwrap_or_else(String::new),
        }
    }

    pub(crate) fn into_raw(self) -> (PathArc, Name, ArtifactRaw) {
        let partof = if self.partof.is_empty() {
            None
        } else {
            Some(NamesRaw::from(self.partof))
        };

        let text = if self.text.is_empty() {
            None
        } else {
            Some(TextRaw(self.text))
        };

        let raw = ArtifactRaw {
            done: self.done,
            partof: partof,
            text: text,
        };
        (self.file, self.name, raw)
    }
}

impl From<Artifact> for ArtifactIm {
    /// Get an `ArtifactIm` from an `Artifact`
    fn from(art: Artifact) -> ArtifactIm {
        let mut out = ArtifactIm {
            name: art.name,
            file: art.file.into(),
            partof: art.partof,
            done: match art.impl_ {
                ::implemented::Impl::Done(d) => Some(d),
                _ => None,
            },
            text: art.text,
        };
        out.clean();
        out
    }
}

impl Hash for ArtifactIm {
    /// Normal hash **except** we use `name.as_str().hash()` instead of
    /// `name.hash()` to record whether the raw name itself changed.
    ///
    /// Note: normally name is hashed by its type and key.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.as_str().hash(state);
        self.file.hash(state);
        // note: guaranteed that it is always stripped and sorted
        for p in self.partof.iter() {
            p.hash(state);
        }
        self.done.hash(state);
        self.text.hash(state);
    }
}

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
