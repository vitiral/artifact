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
use base64;
use ergo::serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use siphasher::sip128::{Hasher128, SipHasher};
use std::fmt;

use crate::dev_prelude::*;
use crate::raw::{self, ArtifactRaw, TextRaw};
use crate::raw_names::NamesRaw;

pub trait ArtifactImExt {
    /// Get an `ArtifactIm` from an `ArtifactRaw`.
    fn from_raw(name: Name, file: PathFile, raw: ArtifactRaw) -> ArtifactIm;

    fn into_raw(self) -> (PathArc, Name, ArtifactRaw);
}

impl ArtifactImExt for ArtifactIm {
    fn from_raw(name: Name, file: PathFile, raw: ArtifactRaw) -> ArtifactIm {
        let mut partof = raw
            .partof
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
