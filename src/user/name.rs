/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! module for defining logic for parsing and collapsing artifact names

use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

use dev_prefix::*;
use types::*;

// Public Trait Methods

impl FromStr for Name {
    type Err = Error;
    fn from_str(s: &str) -> Result<Name> {
        Name::from_string(s.to_string())
    }
}

impl Name {
    pub fn from_string(s: String) -> Result<Name> {
        let value = s.to_ascii_uppercase();
        if !NAME_VALID.is_match(&value) {
            return Err(ErrorKind::InvalidName(s.to_string()).into());
        }
        let value: Vec<String> = value.split('-').map(|s| s.to_string()).collect();
        let ty = _get_type(&value[0], &s)?;
        Ok(Name {
            raw: s,
            value: value,
            ty: ty,
        })
    }

    /// parse name from string and handle errors
    /// see: SPC-artifact-name.2
    /// see: SPC-artifact-partof-2
    pub fn parent(&self) -> Option<Name> {
        if self.value.len() <= 2 {
            return None;
        }
        let mut value = self.value.clone();
        value.pop().unwrap();
        Some(Name {
            raw: value.join("-"),
            value: value,
            ty: self.ty,
        })
    }

    /// return whether this is a root name (whether it has no parent)
    pub fn is_root(&self) -> bool {
        self.value.len() == 2
    }

    pub fn parent_rc(&self) -> Option<NameRc> {
        match self.parent() {
            Some(p) => Some(Arc::new(p)),
            None => None,
        }
    }

    /// return the artifact this artifact is automatically
    /// a partof (because of it's name)
    /// see: SPC-artifact-partof-1
    pub fn named_partofs(&self) -> Vec<Name> {
        match self.ty {
            Type::TST => vec![self._get_named_partof("SPC")],
            Type::SPC => vec![self._get_named_partof("REQ")],
            Type::REQ => vec![],
        }
    }

    /// CAN PANIC
    fn _get_named_partof(&self, ty: &str) -> Name {
        let s = ty.to_string() + self.raw.split_at(3).1;
        Name::from_str(&s).unwrap()
    }
}

#[test]
fn test_artname_parent() {
    let name = Name::from_str("REQ-foo-bar-b").unwrap();
    let parent = name.parent().unwrap();
    assert_eq!(parent, Name::from_str("REQ-foo-bar").unwrap());
    let parent = parent.parent().unwrap();
    assert_eq!(parent, Name::from_str("REQ-foo").unwrap());
}

impl Default for Name {
    fn default() -> Name {
        Name::from_str("REQ-default").unwrap()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value.join("-"))
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Name) -> bool {
        self.value == other.value
    }
}

impl Eq for Name {}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl LoadFromStr for NameRc {
    fn from_str(s: &str) -> Result<NameRc> {
        Ok(Arc::new(try!(Name::from_str(s))))
    }
}

impl LoadFromStr for Names {
    /// Parse a "names str" and convert into a Set of Names
    fn from_str(partof_str: &str) -> Result<Names> {
        let strs = try!(parse_names(&mut partof_str.chars(), false));
        let mut out = HashSet::new();
        for s in strs {
            out.insert(Arc::new(try!(Name::from_str(&s))));
        }
        Ok(out)
    }
}

// Private Methods

fn _get_type(value: &str, raw: &str) -> Result<Type> {
    match value {
        "REQ" => Ok(Type::REQ),
        "SPC" => Ok(Type::SPC),
        "TST" => Ok(Type::TST),
        _ => Err(
            ErrorKind::InvalidName(format!(
                "name must start with REQ-, SPC- or TST-: \
                 {}",
                raw
            )).into(),
        ),
    }
}

// Private: Expanding Names. Use `Name::from_str`

/// subfunction to parse names from a names-str recusively
fn parse_names<I>(raw: &mut I, in_brackets: bool) -> Result<Vec<String>>
where
    I: Iterator<Item = char>,
{
    // hello-[there, you-[are, great]]
    // hello-there, hello-you-are, hello-you-great
    let mut strout = String::new();
    let mut current = String::new();
    loop {
        // SPC-names.1: read one char at a time
        let c = match raw.next() {
            Some(c) => c,
            None => {
                if in_brackets {
                    // SPC-names.2: do validation
                    return Err(
                        ErrorKind::InvalidName("brackets are not closed".to_string()).into(),
                    );
                }
                break;
            }
        };
        match c {
            ' ' | '\n' | '\r' => {}
            // ignore whitespace
            '[' => {
                if current == "" {
                    // SPC-names.2: more validation
                    let msg = "cannot have '[' after characters ',' or ']'\
                               or at start of string"
                        .to_string();
                    return Err(ErrorKind::InvalidName(msg).into());
                }
                // SPC-names.3: recurse for brackets
                for p in try!(parse_names(raw, true)) {
                    strout.write_str(&current).unwrap();
                    strout.write_str(&p).unwrap();
                    strout.push(',');
                }
                current.clear();
            }
            ']' => break,
            ',' => {
                strout.write_str(&current).unwrap();
                strout.push(',');
                current.clear();
            }
            _ => current.push(c),
        }
    }
    strout.write_str(&current).unwrap();
    Ok(
        strout
            .split(',')
            .filter(|s| s != &"")
            .map(|s| s.to_string())
            .collect(),
    )
}

#[cfg(test)]
fn do_test_parse(user: &str, expected_collapsed: &[&str]) {
    let parsed = parse_names(&mut user.chars(), false).unwrap();
    assert_eq!(parsed, expected_collapsed);
}

#[test]
/// #TST-project-partof
fn test_parse_names() {
    do_test_parse("hi, ho", &["hi", "ho"]);
    do_test_parse("hi-[he, ho]", &["hi-he", "hi-ho"]);
    do_test_parse(
        "he-[ha-[ha, he], hi, ho], hi-[he, ho]",
        &["he-ha-ha", "he-ha-he", "he-hi", "he-ho", "hi-he", "hi-ho"],
    );
    assert!(parse_names(&mut "[]".chars(), false).is_err());
    assert!(parse_names(&mut "[hi]".chars(), false).is_err());
    assert!(parse_names(&mut "hi-[ho, [he]]".chars(), false).is_err());
    assert!(parse_names(&mut "hi-[ho, he".chars(), false).is_err());
}

#[test]
fn test_name() {
    // valid names
    for name in vec![
        "REQ-foo",
        "REQ-foo-2",
        "REQ-foo2",
        "REQ-foo2",
        "REQ-foo-bar-2_3",
        "SPC-foo",
        "TST-foo",
    ] {
        assert!(Name::from_str(name).is_ok());
    }
    for name in vec!["REQ-foo*", "REQ-foo\n", "REQ-foo-"] {
        assert!(Name::from_str(name).is_err())
    }
    // spaces are invalid
    assert!(Name::from_str("REQ-foo ").is_err());
}
