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
use base64;
use siphasher::sip128::{Hash128, Hasher128, SipHasher};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

use dev_prelude::*;
use family;
use raw::ArtifactRaw;
use artifact::Artifact;
use name::Name;
use path_abs::PathFile;

/// The type used for unique hash ids
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashIm([u8; 16]);

/// #SPC-structs.artifact_op
/// Used for specifying operations to perform.
pub enum ArtifactOp {
    Create { artifact: ArtifactIm, hash: HashIm },
    Update(HashIm, ArtifactIm),
    Delete(HashIm),
}

#[derive(Debug)]
/// #SPC-structs.artifact_im
pub struct ArtifactIm {
    pub(crate) name: Name,
    pub(crate) file: PathFile,
    pub(crate) partof: OrderSet<Name>,
    pub(crate) done: Option<String>,
    pub(crate) text: String,
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
            file: file,
            partof: partof,
            done: raw.done,
            text: raw.text.map(|t| t.0).unwrap_or_else(String::new),
        }
    }
}

impl From<Artifact> for ArtifactIm {
    /// Get an `ArtifactIm` from an `Artifact`
    fn from(mut art: Artifact) -> ArtifactIm {
        family::strip_auto_partofs(&art.name, &mut art.partof);
        art.partof.sort();
        ArtifactIm {
            name: art.name,
            file: art.file,
            partof: art.partof,
            done: match art.impl_ {
                ::implemented::Impl::Done(d) => Some(d),
                _ => None,
            },
            text: art.text,
        }
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

impl Serialize for HashIm {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let b64 = base64::encode_config(
            &self.0,
            base64::URL_SAFE_NO_PAD,
        );
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
