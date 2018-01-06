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
//! #SPC-data-family
//! This implements the types related to discovering the "family"
//! of any artifact.

use std::sync::mpsc::Sender;
use std::fmt;
use serde;
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};

use dev_prelude::*;
use name::{Name, Type, TYPE_SPLIT_LOC};
use path_abs::PathAbs;
use lint;

#[macro_export]
/// Macro to get a name with no error checking.
macro_rules! names {
    ($raw:expr) => (
        match Names::from_str(&$raw) {
            Ok(n) => n,
            Err(e) => panic!("invalid names!({}): {}", $raw, e),
        }
    );
}

/// Collection of Names, used in partof and parts for storing relationships
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Names(pub(crate) OrderSet<Name>);

impl fmt::Debug for Names {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for Names {
    type Target = OrderSet<Name>;

    fn deref(&self) -> &OrderSet<Name> {
        &self.0
    }
}

impl DerefMut for Names {
    fn deref_mut(&mut self) -> &mut OrderSet<Name> {
        &mut self.0
    }
}

impl From<OrderSet<Name>> for Names {
    fn from(names: OrderSet<Name>) -> Names {
        Names(names)
    }
}

impl From<HashSet<Name>> for Names {
    fn from(mut names: HashSet<Name>) -> Names {
        Names(names.drain().collect())
    }
}

impl FromStr for Names {
    type Err = Error;
    /// Parse a collapsed set of names to create them
    fn from_str(collapsed: &str) -> Result<Names> {
        let inner = ::expand_names::expand_names(collapsed)?;
        Ok(Names(inner))
    }
}

impl Name {
    /// #SPC-data-family.parent
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

    /// #SPC-data-family.auto_partof
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
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
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
    fn deserialize<D>(deserializer: D) -> result::Result<Names, D::Error>
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

    fn visit_seq<A>(self, mut seq: A) -> result::Result<Self::Value, A::Error>
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

/// Run lints on the names, making sure that:
/// - all children have parents.
pub fn lint_names(lints: &Sender<lint::Lint>, names: &OrderMap<Name, PathAbs>) {
    for (name, path) in names.iter() {
        if let Some(parent) = name.parent() {
            if !names.contains_key(&parent) {
                lints
                    .send(lint::Lint {
                        level: lint::Level::Error,
                        path: Some(path.to_path_buf()),
                        line: None,
                        category: lint::Category::AutoPartof,
                        msg: format!(
                            "Parent of {} ({}) must exist but does not",
                            name.as_str(),
                            parent.as_str()
                        ),
                    })
                    .expect("inserting lint in auto_partof");
            }
        }
    }
}

/// #SPC-data-family.auto
/// Given an ordermap of all names, return the partof attributes that will be added.
pub fn auto_partofs<T>(names: &OrderMap<Name, T>) -> OrderMap<Name, OrderSet<Name>> {
    let mut out: OrderMap<Name, OrderSet<Name>> = OrderMap::with_capacity(names.len());
    for name in names.keys() {
        let mut auto = OrderSet::new();
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
    out
}
