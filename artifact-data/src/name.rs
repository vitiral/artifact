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
//! #SPC-data-name
//!
//! This is the name module, the module for representing artifact names
//! and their global cache.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::result;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use regex::Regex;

use dev_prelude::*;

// EXPORTED TYPES AND FUNCTIONS

#[macro_export]
/// Macro to get a name with no error checking.
macro_rules! name {
    ($raw:expr) => (
        match Name::from_str(&$raw) {
            Ok(n) => n,
            Err(_) => panic!("invalid name!({})", $raw),
        }
    );
}

#[macro_export]
/// Macro to get a subname with no error checking.
macro_rules! subname {
    ($raw:expr) => (
        match SubName::from_str(&$raw) {
            Ok(n) => n,
            Err(_) => panic!("invalid subname!({})", $raw),
        }
    );
}

#[derive(Debug, Fail)]
pub enum NameError {
    #[fail(display = "{}", msg)] InvalidName { msg: String },

    #[fail(display = "{}", msg)] InvalidCollapsed { msg: String },

    #[fail(display = "{}", msg)] InvalidSubName { msg: String },
}

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
/// The atomically reference counted name, the primary one used by
/// this module.
///
/// # Examples
/// ```rust
/// #[macro_use] extern crate artifact_data;
/// use artifact_data::{Name, SubName, Type};
/// use std::str::FromStr;
///
/// # fn main() {
/// let name = name!("REQ-example");    // macro instantiation
/// let name2 = name.clone();           // cloning is cheap.
/// assert_eq!(name, Name::from_str("REQ-example").unwrap());
/// assert_eq!(name.ty, Type::REQ);
///
/// // case is ignored for equality/hashing
/// assert_eq!(name!("SPC-key"), name!("sPc-KeY"));
///
/// // Helper to get the full name
/// assert_eq!(name!("REQ-foo").full(Some(&subname!(".sub"))), "REQ-foo.sub");
/// # }
/// ```
pub struct Name {
    inner: Arc<InternalName>,
}

/// type of an `Artifact`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    REQ,
    SPC,
    TST,
}

#[derive(Clone)]
/// A subname, i.e. `ART-foo.subname`
///
/// # Examples
/// ```rust
/// #[macro_use] extern crate artifact_data;
/// use artifact_data::{Name, SubName, Type};
/// use std::str::FromStr;
///
/// # fn main() {
/// let sub = subname!(".sub_name"); // macro instantiation
/// let sub2 = sub.clone();          // cloning is NOT cheap.
///
/// // case is ignored for equality/hashing
/// assert_eq!(sub, SubName::from_str(".SuB_NaMe").unwrap());
///
/// // Helper to get the full name
/// assert_eq!(name!("REQ-foo").full(Some(&sub)), "REQ-foo.sub_name");
/// # }
pub struct SubName {
    pub raw: String,
    pub key: String,
}

/// Internal Name object, use Name instead.
#[derive(Clone)]
pub struct InternalName {
    /// The artifact type, determined from the name prefix
    pub ty: Type,
    /// Capitalized form
    pub key: Arc<String>,
    /// Raw "user" form
    pub raw: String,
}

// CONSTANTS

/// The location in the name where the type is split at
/// ```
/// REQ-foo
///    ^
/// ```
pub const TYPE_SPLIT_LOC: usize = 3;

macro_rules! NAME_VALID_CHARS {
    () => { "A-Z0-9_" };
}

/// base definition of a valid name. Some pieces may ignore case.
pub const NAME_VALID_STR: &str = concat!(
    r"(?:REQ|SPC|TST)-(?:[",
    NAME_VALID_CHARS!(),
    r"]+-)*(?:[",
    NAME_VALID_CHARS!(),
    r"]+)",
);

lazy_static!{
    /// Valid name regular expression
    static ref NAME_VALID_RE: Regex = Regex::new(
        &format!(r"(?i)^{}$", NAME_VALID_STR)).unwrap();

    /// Valid subname regex
    static ref VALID_SUB_NAME_RE: Regex = Regex::new(
        &format!(r"(?i)^\.[{}]+$", NAME_VALID_CHARS!())).unwrap();

    /// Parse subname from text regex
    static ref TEXT_SUB_NAME_RE: Regex = Regex::new(
        &format!(r"(?i)\[\[(\.[{}]+)\]\]", NAME_VALID_CHARS!())).unwrap();
}

// NAME METHODS
impl Name {
    /// Get the `Name`'s user-defined string representation.
    ///
    /// # Examples
    /// ```rust
    /// #[macro_use] extern crate artifact_data;
    /// use artifact_data::Name;
    /// use std::str::FromStr;
    ///
    /// # fn main() {
    /// assert_eq!(
    ///     name!("REQ-Example").as_str(),
    ///     "REQ-Example"
    /// );
    /// # }
    /// ```
    /// Get the raw str representation
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Get the `Name`'s "key" representation.
    ///
    /// # Examples
    /// ```rust
    /// #[macro_use] extern crate artifact_data;
    /// use artifact_data::Name;
    /// use std::str::FromStr;
    ///
    /// # fn main() {
    /// assert_eq!(
    ///     name!("REQ-Example").key_str(),
    ///     "REQ-EXAMPLE"
    /// );
    /// # }
    /// ```
    /// Get the raw str representation
    pub fn key_str(&self) -> &str {
        &self.key
    }

    /// Concatenate the name with a (possible) subname.
    ///
    /// # Examples
    /// ```rust
    /// #[macro_use] extern crate artifact_data;
    /// use artifact_data::{Name, SubName};
    /// use std::str::FromStr;
    ///
    /// # fn main() {
    /// assert_eq!(
    ///     name!("REQ-Example").full(Some(&subname!(".sub"))),
    ///     "REQ-Example.sub"
    /// );
    /// # }
    /// ```
    pub fn full(&self, subname: Option<&SubName>) -> String {
        let mut out: String = self.as_str().to_string();
        if let Some(s) = subname {
            out.push_str(s.as_str());
        }
        out
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> result::Result<Name, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Name::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner.raw)
    }
}

impl Deref for Name {
    type Target = InternalName;

    fn deref(&self) -> &InternalName {
        self.inner.as_ref()
    }
}

impl FromStr for Name {
    type Err = Error;
    #[cfg(feature = "cache")]
    /// Primary method to create a name.
    ///
    /// The name itself, as well as its key are actually stored in the cache
    fn from_str(raw: &str) -> Result<Name> {
        let mut cache = ::cache::NAME_CACHE.lock().expect("name cache poisioned");
        cache.get(raw)
    }

    #[cfg(not(feature = "cache"))]
    /// When fuzz testing, the cost of the cache is too great as we are
    /// creating random names too often.
    ///
    /// I COULD clear the cache for every fuzz test, but I don't want to
    /// diagnose memory running out if I forget.
    fn from_str(raw: &str) -> Result<Name> {
        Ok(Name {
            inner: Arc::new(InternalName::from_str(raw)?),
        })
    }
}

// PUBLIC METHODS

/// Parse subnames from the text field.
pub(crate) fn parse_subnames(text: &str) -> OrderSet<SubName> {
    TEXT_SUB_NAME_RE
        .captures_iter(text)
        .map(|cap| SubName::new_unchecked(cap.get(1).unwrap().as_str()))
        .collect()
}

// INTERNAL NAME METHODS

impl FromStr for InternalName {
    type Err = Error;

    /// Use `Name::from_str` instead. This should only be used in this module.
    fn from_str(raw: &str) -> Result<InternalName> {
        if !NAME_VALID_RE.is_match(raw) {
            let msg = format!("Name is invalid: {}", raw);
            return Err(NameError::InvalidName { msg: msg }.into());
        }
        let key = Arc::new(raw.to_ascii_uppercase());
        let ty = match &key[0..TYPE_SPLIT_LOC] {
            "REQ" => Type::REQ,
            "SPC" => Type::SPC,
            "TST" => Type::TST,
            _ => unreachable!(),
        };

        Ok(InternalName {
            ty: ty,
            key: key,
            raw: raw.into(),
        })
    }
}

impl Hash for InternalName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // name is a hash of its type and key
        self.ty.hash(state);
        self.key.hash(state);
    }
}

impl PartialEq for InternalName {
    fn eq(&self, other: &InternalName) -> bool {
        self.ty == other.ty && self.key == other.key
    }
}

impl Eq for InternalName {}

impl Ord for InternalName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for InternalName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// SUBNAME METHODS
//
impl Serialize for SubName {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> Deserialize<'de> for SubName {
    fn deserialize<D>(deserializer: D) -> result::Result<SubName, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SubName::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl SubName {
    /// Unchecked creation of subname
    pub(crate) fn new_unchecked(raw: &str) -> SubName {
        debug_assert!(VALID_SUB_NAME_RE.is_match(raw), "raw: {:?}", raw);
        SubName {
            raw: raw.to_string(),
            key: raw.to_ascii_uppercase(),
        }
    }

    /// Get the raw str representation
    ///
    /// # Examples
    /// ```rust
    /// #[macro_use] extern crate artifact_data;
    /// use artifact_data::SubName;
    /// use std::str::FromStr;
    ///
    /// # fn main() {
    /// assert_eq!(
    ///     subname!(".Example").as_str(),
    ///     ".Example"
    /// );
    /// # }
    /// ```
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Get the "key" representation of the subname.
    ///
    /// # Examples
    /// ```rust
    /// #[macro_use] extern crate artifact_data;
    /// use artifact_data::SubName;
    /// use std::str::FromStr;
    ///
    /// # fn main() {
    /// assert_eq!(
    ///     subname!(".example").key_str(),
    ///     ".EXAMPLE"
    /// );
    /// # }
    /// ```
    pub fn key_str(&self) -> &str {
        &self.key
    }
}

impl FromStr for SubName {
    type Err = Error;

    /// Primary method to create a subname.
    fn from_str(raw: &str) -> Result<SubName> {
        if !VALID_SUB_NAME_RE.is_match(raw) {
            Err(NameError::InvalidSubName {
                msg: format!("{} is not a valid subname", raw),
            }.into())
        } else {
            Ok(SubName::new_unchecked(raw))
        }
    }
}

impl fmt::Debug for SubName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.raw)
    }
}

impl Hash for SubName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for SubName {
    fn eq(&self, other: &SubName) -> bool {
        self.key == other.key
    }
}

impl Eq for SubName {}

impl Ord for SubName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for SubName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// TYPE METHODS

impl Type {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Type::REQ => "REQ",
            Type::SPC => "SPC",
            Type::TST => "TST",
        }
    }
}

// NAME CACHE METHODS

#[cfg(feature = "cache")]
impl ::cache::NameCache {
    /// Get the name from the cache, inserting it if it doesn't exist
    ///
    /// This is the only way that names are created.
    fn get(&mut self, raw: &str) -> Result<Name> {
        // FIXME: I would like to use Arc for raw+name, but
        // Borrow<str> is not implemented for Arc<String>
        if let Some(n) = self.names.get(raw) {
            return Ok(n.clone());
        }

        let name = Name {
            inner: Arc::new(InternalName::from_str(raw)?),
        };
        self.names.insert(raw.into(), name.clone());
        Ok(name)
    }
}
