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
use raw::{self, ArtifactRaw, TextRaw};
use raw_names::NamesRaw;

pub trait ArtifactImExt {

    /// Get an `ArtifactIm` from an `ArtifactRaw`.
    fn from_raw(name: Name, file: PathFile, raw: ArtifactRaw) -> ArtifactIm;

    fn into_raw(self) -> (PathArc, Name, ArtifactRaw);
}

impl ArtifactImExt for ArtifactIm {

    fn from_raw(name: Name, file: PathFile, raw: ArtifactRaw) -> ArtifactIm {
        let mut partof = raw.partof
            .map(|mut p| {
                strip_auto_partofs(&name, &mut p.0);
                p.drain(..).collect()
            })
            .unwrap_or_else(IndexSet::new);
        partof.sort();

        ArtifactIm {
            name: name,
            file: file.into(),
            partof: partof,
            done: raw.done,
            text: raw.text.map(|t| t.0).unwrap_or_else(String::new),
        }
    }

    fn into_raw(self) -> (PathArc, Name, ArtifactRaw) {
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
