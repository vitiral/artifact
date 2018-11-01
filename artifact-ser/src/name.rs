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
//! #SPC-name
//!
//! This is the name module, the module for representing artifact names
//! and subnames

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::result;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

use dev_prelude::*;

// use ergo_std::*;
// use ergo_config::*;
// use failure::*;

// EXPORTED TYPES AND FUNCTIONS

#[macro_export]
/// Macro to get a name with no error checking.
macro_rules! name {
    ($raw: expr) => {
        match Name::from_str(&$raw) {
            Ok(n) => n,
            Err(_) => panic!("invalid name!({})", $raw),
        }
    };
}

#[macro_export]
/// Macro to get a subname with no error checking.
macro_rules! subname {
    ($raw: expr) => {
        match SubName::from_str(&$raw) {
            Ok(n) => n,
            Err(_) => panic!("invalid subname!({})", $raw),
        }
    };
}

pub type NameResult<T> = Result<T, NameError>;

#[derive(Debug, Error)]
pub enum NameError {
    #[error(msg_embedded, no_from, non_std)]
    /// Name is not valid.
    InvalidName { msg: String },
    #[error(msg_embedded, no_from, non_std)]
    /// Collapsed form is not valid.
    InvalidCollapsed { msg: String },
    #[error(msg_embedded, no_from, non_std)]
    /// SubName is not valid.
    InvalidSubName { msg: String },
}


#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
/// The atomically reference counted name, the primary one used by
/// this module.
///
/// # Examples
/// ```rust
/// #[macro_use] extern crate artifact_lib;
/// use artifact_lib::{Name, SubName, Type};
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
pub struct Name(Arc<InternalName>);

/// #SPC-name.type
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
/// #[macro_use] extern crate artifact_lib;
/// use artifact_lib::{Name, SubName, Type};
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
pub struct SubName(pub Arc<InternalSubName>);

/// Internal SubName object, use SubName instead.
pub struct InternalSubName {
    pub raw: String,
    pub key: String,
}

/// Internal Name object, use Name instead.
#[derive(Clone)]
pub struct InternalName {
    /// The artifact type, determined from the name prefix
    pub ty: Type,
    /// Capitalized form
    pub key: String,
    /// Raw "user" form
    pub raw: String,
}

// CONSTANTS

/// The location in the name where the type is split at
/// ```text,norun
/// REQ-foo
///    ^
/// ```
pub const TYPE_SPLIT_LOC: usize = 3;

#[macro_export]
macro_rules! NAME_VALID_CHARS {
    () => {
        "A-Z0-9_"
    };
}

/// base definition of a valid name. Some pieces may ignore case.
pub const NAME_VALID_STR: &str = concat!(
    r"(?:REQ|SPC|TST)-(?:[",
    NAME_VALID_CHARS!(),
    r"]+-)*(?:[",
    NAME_VALID_CHARS!(),
    r"]+)",
);

pub const SUB_RE_KEY: &str = "sub";
pub const NAME_RE_KEY: &str = "name";
pub const NAME_SUB_RE_KEY: &str = "name_sub";

lazy_static!{
    /// Valid name regular expression
    static ref NAME_VALID_RE: Regex = Regex::new(
        &format!(r"(?i)^{}$", NAME_VALID_STR)).unwrap();

    /// Valid subname regex
    static ref VALID_SUB_NAME_RE: Regex = Regex::new(
        &format!(r"(?i)^\.(?:tst-)?[{}]+$", NAME_VALID_CHARS!())).unwrap();

    pub static ref TEXT_SUB_NAME_STR: String = format!(
        r"(?i)\[\[(?P<{}>\.(?:tst-)?[{}]+)\]\]",
        SUB_RE_KEY,
        NAME_VALID_CHARS!(),
    );

    pub static ref TEXT_REF_STR: String = format!(r#"(?xi)
        \[\[(?P<{1}>                # start main section
        (?:REQ|SPC|TST)             # all types are supported
        -(?:[{0}]+-)*               # any number of first element
        (?:[{0}]+)                  # required end element
        )                           # end main section
        (?P<{2}>\.(?:tst-)?[{0}]+)? # (optional) sub section
        \]\]                        # close text reference
        "#,
        NAME_VALID_CHARS!(),
        NAME_RE_KEY,
        NAME_SUB_RE_KEY,
    );

    /// Parse subname from text regex
    pub static ref TEXT_SUB_NAME_RE: Regex = Regex::new(&TEXT_SUB_NAME_STR).unwrap();

    /// Name reference that can exist in source code
    pub static ref TEXT_REF_RE: Regex = Regex::new(&TEXT_REF_STR).unwrap();
}

// #SPC-name.attrs
// NAME METHODS
impl Name {
    /// Get the `Name`'s user-defined string representation.
    ///
    /// # Examples
    /// ```rust
    /// #[macro_use] extern crate artifact_lib;
    /// use artifact_lib::Name;
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
    /// #[macro_use] extern crate artifact_lib;
    /// use artifact_lib::Name;
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
    /// #[macro_use] extern crate artifact_lib;
    /// use artifact_lib::{Name, SubName};
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

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
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
        write!(f, "{}", self.0.raw)
    }
}

impl Deref for Name {
    type Target = InternalName;

    fn deref(&self) -> &InternalName {
        self.0.as_ref()
    }
}

impl FromStr for Name {
    type Err = NameError;
    fn from_str(raw: &str) -> NameResult<Name> {
        Ok(Name(Arc::new(InternalName::from_str(raw)?)))
    }
}

// PUBLIC METHODS

/// Parse subnames from the text field.
pub fn parse_subnames(text: &str) -> IndexSet<SubName> {
    TEXT_SUB_NAME_RE
        .captures_iter(text)
        .map(|cap| SubName::new_unchecked(cap.name(SUB_RE_KEY).unwrap().as_str()))
        .collect()
}

// INTERNAL NAME METHODS

impl FromStr for InternalName {
    type Err = NameError;

    /// Use `Name::from_str` instead. This should only be used in this module.
    fn from_str(raw: &str) -> NameResult<InternalName> {
        if !NAME_VALID_RE.is_match(raw) {
            let msg = format!("Name is invalid: {}", raw);
            return Err(NameError::InvalidName { msg: msg }.into());
        }
        let key = raw.to_ascii_uppercase();
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

impl fmt::Display for SubName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Deref for SubName {
    type Target = InternalSubName;

    fn deref(&self) -> &InternalSubName {
        self.0.as_ref()
    }
}

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
    pub fn new_unchecked(raw: &str) -> SubName {
        debug_assert!(VALID_SUB_NAME_RE.is_match(raw), "raw: {:?}", raw);
        SubName(Arc::new(InternalSubName {
            raw: raw.to_string(),
            key: raw.to_ascii_uppercase(),
        }))
    }

    /// Get the raw str representation
    ///
    /// # Examples
    /// ```rust
    /// #[macro_use] extern crate artifact_lib;
    /// use artifact_lib::SubName;
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
    /// #[macro_use] extern crate artifact_lib;
    /// use artifact_lib::SubName;
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
    type Err = NameError;

    /// Primary method to create a subname.
    fn from_str(raw: &str) -> NameResult<SubName> {
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
        write!(f, "{:?}", self.0.raw)
    }
}

impl Hash for SubName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.key.hash(state);
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
        self.0.key.cmp(&other.key)
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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
