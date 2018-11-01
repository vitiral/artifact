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
//! Define the serialization rules for raw names

use dev_prelude::*;
use std::fmt;
use ergo::serde;
use ergo::serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use ergo::serde::ser::{Serialize, Serializer};

#[macro_export]
/// Macro to get 'raw' names with no error checking
macro_rules! names_raw {
    ($raw:expr) => (
        match NamesRaw::from_str(&$raw) {
            Ok(n) => n,
            Err(e) => panic!("invalid names!({}): {}", $raw, e),
        }
    );
}

/// Collection of `NamesRaw`.
///
/// This mostly exists to provide custom
/// serialization/deserializtion for a better text user interface.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct NamesRaw(pub(crate) IndexSet<Name>);

impl fmt::Debug for NamesRaw {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for NamesRaw {
    type Target = IndexSet<Name>;

    fn deref(&self) -> &IndexSet<Name> {
        &self.0
    }
}

impl DerefMut for NamesRaw {
    fn deref_mut(&mut self) -> &mut IndexSet<Name> {
        &mut self.0
    }
}

impl From<IndexSet<Name>> for NamesRaw {
    fn from(names: IndexSet<Name>) -> NamesRaw {
        NamesRaw(names)
    }
}

impl From<HashSet<Name>> for NamesRaw {
    fn from(mut names: HashSet<Name>) -> NamesRaw {
        NamesRaw(names.drain().collect())
    }
}

impl FromStr for NamesRaw {
    type Err = Error;
    /// Parse a collapsed set of names to create them
    fn from_str(collapsed: &str) -> Result<NamesRaw> {
        Ok(NamesRaw(expand_names(collapsed)?))
    }
}

impl Serialize for NamesRaw {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0.is_empty() {
            panic!("attempted to serialize an empty names field");
        } else if self.0.len() == 1 {
            // serialize just the string
            self.0.iter().next().unwrap().serialize(serializer)
        } else {
            // serialize the sorted names
            let mut names: Vec<_> = self.0.iter().collect();
            names.sort();
            names.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for NamesRaw {
    fn deserialize<D>(deserializer: D) -> result::Result<NamesRaw, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NamesRawVisitor)
    }
}

struct NamesRawVisitor;

impl<'de> Visitor<'de> for NamesRawVisitor {
    type Value = NamesRaw;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an artifact or list of artifacts that can be in collapsed form")
    }

    fn visit_str<E>(self, v: &str) -> result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        NamesRaw::from_str(v).map_err(serde::de::Error::custom)
    }

    fn visit_seq<A>(self, mut seq: A) -> result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut out = NamesRaw::default();
        // Note: `::<String>` is necessary
        while let Some(s) = seq.next_element::<String>()? {
            let mut elem = NamesRaw::from_str(&s).map_err(serde::de::Error::custom)?;
            out.extend(elem.drain(..));
        }
        Ok(out)
    }
}
