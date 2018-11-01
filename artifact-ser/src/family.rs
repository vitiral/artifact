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
//! #SPC-read-family
//! This implements the types related to discovering the "family"
//! of any artifact.

use std::fmt;
use ergo_std::serde;
use ergo_std::serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use ergo_std::serde::ser::{Serialize, Serializer};

use dev_prelude::*;
use name::{Name, NameError, Type, TYPE_SPLIT_LOC};

#[macro_export]
/// Macro to get a name with no error checking.
macro_rules! names {
    ($raw: expr) => {
        match Names::from_str(&$raw) {
            Ok(n) => n,
            Err(e) => panic!("invalid names!({}): {}", $raw, e),
        }
    };
}

/// Collection of Names, used in partof and parts for storing relationships
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Names(pub(crate) IndexSet<Name>);

impl fmt::Debug for Names {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for Names {
    type Target = IndexSet<Name>;

    fn deref(&self) -> &IndexSet<Name> {
        &self.0
    }
}

impl DerefMut for Names {
    fn deref_mut(&mut self) -> &mut IndexSet<Name> {
        &mut self.0
    }
}

impl From<IndexSet<Name>> for Names {
    fn from(names: IndexSet<Name>) -> Names {
        Names(names)
    }
}

impl From<HashSet<Name>> for Names {
    fn from(mut names: HashSet<Name>) -> Names {
        Names(names.drain().collect())
    }
}

impl FromStr for Names {
    type Err = NameError;
    /// Parse a collapsed set of names to create them
    fn from_str(collapsed: &str) -> Result<Names, NameError> {
        let inner = ::expand_names::expand_names(collapsed)?;
        Ok(Names(inner))
    }
}

impl Name {
    /// #SPC-read-family.parent
    /// The parent of the name. This must exist if not None for all
    /// artifats.
    pub fn parent(&self) -> Option<Name> {
        let loc = self.raw.rfind('-').expect("name.parent:rfind");
        if loc == TYPE_SPLIT_LOC {
            None
        } else {
            Some(Name::from_str(&self.raw[0..loc]).expect("name.parent:from_str"))
        }
    }

    /// #SPC-read-family.auto_partof
    /// The artifact that this COULD be automatically linked to.
    ///
    /// - REQ is not autolinked to anything
    /// - SPC is autolinked to the REQ with the same name
    /// - TST is autolinked to the SPC with the same name
    pub fn auto_partof(&self) -> Option<Name> {
        let ty = match self.ty {
            Type::REQ => return None,
            Type::SPC => Type::REQ,
            Type::TST => Type::SPC,
        };
        let mut out = String::with_capacity(self.raw.len());
        out.push_str(ty.as_str());
        out.push_str(&self.raw[TYPE_SPLIT_LOC..self.raw.len()]);
        Some(Name::from_str(&out).unwrap())
    }
}

impl Serialize for Names {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // always sort the names first
        let mut names: Vec<_> = self.0.iter().collect();
        names.sort();
        names.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Names {
    fn deserialize<D>(deserializer: D) -> Result<Names, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(NamesVisitor)
    }
}

struct NamesVisitor;

impl<'de> Visitor<'de> for NamesVisitor {
    type Value = Names;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a list of names")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut out = Names::default();
        while let Some(s) = seq.next_element::<String>()? {
            let n = Name::from_str(&s).map_err(serde::de::Error::custom)?;
            out.insert(n);
        }
        Ok(out)
    }
}

/// #SPC-read-family.auto
/// Given an indexmap of all names, return the partof attributes that will be added.
pub fn auto_partofs<T>(names: &IndexMap<Name, T>) -> IndexMap<Name, IndexSet<Name>> {
    let mut out: IndexMap<Name, IndexSet<Name>> = IndexMap::with_capacity(names.len());
    for name in names.keys() {
        let mut auto = IndexSet::new();
        if let Some(parent) = name.parent() {
            if names.contains_key(&parent) {
                auto.insert(parent);
            }
        }
        if let Some(partof) = name.auto_partof() {
            if names.contains_key(&partof) {
                auto.insert(partof);
            }
        }
        out.insert(name.clone(), auto);
    }
    debug_assert_eq!(names.len(), out.len());
    out
}
