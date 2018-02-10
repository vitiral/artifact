/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! module for defining logic for sub-names

use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

use dev_prefix::*;
use types::*;

impl SubName {
    pub fn from_parts(name: NameRc, sub: String) -> SubName {
        SubName {
            name: name,
            value: sub.to_ascii_uppercase(),
            raw: sub,
        }
    }
}

impl FromStr for SubName {
    type Err = Error;
    fn from_str(s: &str) -> Result<SubName> {
        let split: Vec<_> = s.split('.').collect();
        if split.len() != 2 {
            bail!(ErrorKind::InvalidSubName(s.into()));
        }
        let name = Name::from_str(split[0])?;

        // FIXME: the second part should also be errorchecked
        Ok(SubName::from_parts(Arc::new(name), split[1].to_string()))
    }
}

impl fmt::Display for SubName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.name, self.raw)
    }
}

impl fmt::Debug for SubName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}.{}", self.name, self.value)
    }
}

impl Hash for SubName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.value.hash(state);
    }
}

impl PartialEq for SubName {
    fn eq(&self, other: &SubName) -> bool {
        self.value == other.value && self.name == other.name
    }
}

impl Eq for SubName {}

impl Ord for SubName {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.name.cmp(&other.name) {
            Ordering::Equal => self.value.cmp(&other.value),
            c => c,
        }
    }
}

impl PartialOrd for SubName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self.name.cmp(&other.name) {
            Ordering::Equal => self.value.cmp(&other.value),
            c => c,
        })
    }
}
