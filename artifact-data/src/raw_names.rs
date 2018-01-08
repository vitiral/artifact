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
//! Define the serialization rules for raw names

use dev_prelude::*;
use std::fmt;
use serde;
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};
use name::Name;

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
pub struct NamesRaw {
    pub(crate) inner: OrderSet<Name>,
}

impl fmt::Debug for NamesRaw {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Deref for NamesRaw {
    type Target = OrderSet<Name>;

    fn deref(&self) -> &OrderSet<Name> {
        &self.inner
    }
}

impl DerefMut for NamesRaw {
    fn deref_mut(&mut self) -> &mut OrderSet<Name> {
        &mut self.inner
    }
}

impl From<OrderSet<Name>> for NamesRaw {
    fn from(names: OrderSet<Name>) -> NamesRaw {
        NamesRaw { inner: names }
    }
}

impl From<HashSet<Name>> for NamesRaw {
    fn from(mut names: HashSet<Name>) -> NamesRaw {
        NamesRaw {
            inner: names.drain().collect(),
        }
    }
}

impl FromStr for NamesRaw {
    type Err = Error;
    /// Parse a collapsed set of names to create them
    fn from_str(collapsed: &str) -> Result<NamesRaw> {
        let inner = ::expand_names::expand_names(collapsed)?;
        Ok(NamesRaw { inner: inner })
    }
}

impl Serialize for NamesRaw {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.inner.is_empty() {
            panic!("attempted to serialize an empty names field");
        } else if self.inner.len() == 1 {
            // serialize just the string
            self.inner.iter().next().unwrap().serialize(serializer)
        } else {
            // serialize the sorted names
            let mut names: Vec<_> = self.inner.iter().collect();
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
