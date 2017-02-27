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
//! module for defining logic for parsing and collapsing artifact names

use dev_prefix::*;
use core::types::*;

use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

fn _get_type(value: &str, raw: &str) -> Result<ArtType> {
    match value {
        "REQ" => Ok(ArtType::REQ),
        "SPC" => Ok(ArtType::SPC),
        "RSK" => Ok(ArtType::RSK),
        "TST" => Ok(ArtType::TST),
        _ => {
            Err(ErrorKind::InvalidName(format!("name must start with REQ-, RSK-, SPC- or TST-: \
                                                {}",
                                               raw))
                .into())
        }
    }
}

impl FromStr for ArtName {
    type Err = Error;
    /// #SPC-partof-load
    fn from_str(s: &str) -> Result<ArtName> {
        let value = s.to_ascii_uppercase().replace(' ', "");
        if !ART_VALID.is_match(&value) {
            return Err(ErrorKind::InvalidName(s.to_string()).into());
        }
        let value: Vec<String> = value.split('-').map(|s| s.to_string()).collect();
        let ty = _get_type(&value[0], s)?;
        Ok(ArtName {
            raw: s.to_string(),
            value: value,
            ty: ty,
        })
    }
}

impl ArtName {
    /// parse name from string and handle errors
    /// see: SPC-artifact-name.2

    /// see: SPC-artifact-partof-2
    pub fn parent(&self) -> Option<ArtName> {
        if self.value.len() <= 1 {
            return None;
        }
        let mut value = self.value.clone();
        value.pop().unwrap();
        Some(ArtName {
            raw: value.join("-"),
            value: value,
            ty: self.ty,
        })
    }

    /// return whether this artifact is the root type
    pub fn is_root(&self) -> bool {
        self.value.len() == 1
    }

    pub fn parent_rc(&self) -> Option<ArtNameRc> {
        match self.parent() {
            Some(p) => Some(Arc::new(p)),
            None => None,
        }
    }

    /// return the artifact this artifact is automatically
    /// a partof (because of it's name)
    /// see: SPC-artifact-partof-1
    pub fn named_partofs(&self) -> Vec<ArtName> {
        if self.value.len() <= 1 {
            return vec![];
        }
        let ty = self.ty;
        match ty {
            ArtType::TST => vec![self._get_named_partof("SPC")],
            ArtType::SPC => vec![self._get_named_partof("REQ")],
            ArtType::RSK | ArtType::REQ => vec![],
        }
    }

    /// CAN PANIC
    fn _get_named_partof(&self, ty: &str) -> ArtName {
        let s = ty.to_string() + self.raw.split_at(3).1;
        ArtName::from_str(&s).unwrap()
    }
}

#[test]
fn test_artname_parent() {
    let name = ArtName::from_str("REQ-foo-bar-b").unwrap();
    let parent = name.parent().unwrap();
    assert_eq!(parent, ArtName::from_str("REQ-foo-bar").unwrap());
    let parent = parent.parent().unwrap();
    assert_eq!(parent, ArtName::from_str("REQ-foo").unwrap());
    let parent = parent.parent().unwrap();
    let req = ArtName::from_str("REQ-2").unwrap().parent().unwrap();
    assert_eq!(parent, req);
    assert!(parent.parent().is_none());
}

impl Default for ArtName {
    fn default() -> ArtName {
        ArtName::from_str("REQ-default").unwrap()
    }
}

impl fmt::Display for ArtName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl fmt::Debug for ArtName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value.join("-"))
    }
}

impl Hash for ArtName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for ArtName {
    fn eq(&self, other: &ArtName) -> bool {
        self.value == other.value
    }
}

impl Eq for ArtName {}

impl Ord for ArtName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for ArtName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl LoadFromStr for ArtNameRc {
    fn from_str(s: &str) -> Result<ArtNameRc> {
        Ok(Arc::new(try!(ArtName::from_str(s))))
    }
}

impl LoadFromStr for ArtNames {
    /// Parse a "names str" and convert into a Set of ArtNames
    fn from_str(partof_str: &str) -> Result<ArtNames> {
        let strs = try!(parse_names(&mut partof_str.chars(), false));
        let mut out = HashSet::new();
        for s in strs {
            out.insert(Arc::new(try!(ArtName::from_str(&s))));
        }
        Ok(out)
    }
}


/// subfunction to parse names from a names-str recusively
pub fn parse_names<I>(raw: &mut I, in_brackets: bool) -> Result<Vec<String>>
    where I: Iterator<Item = char>
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
                    return Err(ErrorKind::InvalidName("brackets are not closed".to_string())
                        .into());
                }
                break;
            }
        };
        match c {
            ' ' | '\n' | '\r' => {} // ignore whitespace
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
    Ok(strout.split(',').filter(|s| s != &"").map(|s| s.to_string()).collect())
}

struct NamePiece {
    raw: Vec<Vec<String>>,
    prefix: String,
    pieces: Option<Vec<NamePiece>>,
}

impl NamePiece {
    /// note: raw must be sorted
    fn from(prefix: String, raw: Vec<Vec<String>>) -> NamePiece {
        NamePiece {
            raw: raw,
            prefix: prefix,
            pieces: None,
        }
    }

    /// recursively process the NamePiece until all pieces are just their prefix
    /// this works because:
    /// - we know raw is sorted, so we know all single item prefixes will appear
    ///     one after the other
    /// - from there we just need to go down the tree until all of the lowest
    ///     level pieces have only a prefix
    fn process(&mut self) {
        let mut prefix = "".to_string();
        let mut pieces: Vec<NamePiece> = vec![];
        for part in &self.raw {
            if part.len() == 1 {
                // it is already it's own piece
                pieces.push(NamePiece::from(part[0].clone(), vec![]));
                prefix = "".to_string();
            } else if part[0] == prefix {
                // found (at least) two parts with the same prefix
                // store the part in raw without it's prefix
                let i = pieces.len() - 1; // wow, you can't do this inline...
                pieces[i].raw.push(part.split_first().unwrap().1.to_vec())
            } else {
                // we found a new prefix, create a new piece to store it
                prefix = part[0].clone();
                let raw = part.iter().skip(1).cloned().collect();
                let piece = NamePiece::from(prefix.clone(), vec![raw]);
                pieces.push(piece);
            }
        }
        // we don't need the raw data anymore, it's all been copied somewhere else
        if !self.raw.is_empty() {
            self.raw = vec![];
        }
        if !pieces.is_empty() {
            for p in &mut pieces {
                p.process();
            }
            self.pieces = Some(pieces);
        }
    }

    /// once we have processed all the name pieces, we can collapse them
    /// into a single string
    fn collapse(&self, w: &mut String, is_last: bool) {
        if self.prefix.is_empty() {
            // this is the "head" Piece, it has no filler
            // just write out the pieces
            if let Some(ref pieces) = self.pieces {
                let last_i = pieces.len() - 1;
                for (i, piece) in pieces.iter().enumerate() {
                    piece.collapse(w, last_i == i);
                }
            }
            return;
        }
        w.write_str(&self.prefix).unwrap();
        if let Some(ref pieces) = self.pieces {
            // there are some names after you, more to write
            let last_i = pieces.len() - 1;
            if last_i == 0 {
                // if you only have one piece, then you are foo-bar-baz-etc
                w.write_str("-").unwrap();
            } else {
                // else you are foo-[bar, bar-baz-etc] (unless you are the beginning)
                w.write_str("-[").unwrap();
            }
            for (i, piece) in pieces.iter().enumerate() {
                piece.collapse(w, last_i == i);
            }
            if last_i != 0 {
                w.write_str("]").unwrap();
            }
        }
        if !is_last {
            w.write_str(", ").unwrap();
        }
    }
}

/// take a list of names and collapse them into a single
/// string with format `REQ-foo-[bar, baz-boo], SPC-foo`
pub fn collapse_names(mut names: Vec<String>) -> String {
    names.sort();
    let names: Vec<Vec<String>> = names.iter()
        .map(|n| n.split('-').map(|s| s.to_string()).collect())
        .collect();
    let mut piece = NamePiece {
        raw: names,
        prefix: String::new(),
        pieces: None,
    };
    piece.process();

    let mut collapsed = String::new();
    let is_last = match piece.pieces {
        None => true,
        Some(ref pieces) => pieces.len() > 1,
    };
    piece.collapse(&mut collapsed, is_last);
    collapsed
}

#[cfg(test)]
fn do_test_parse_collapse(user: &str, expected_collapsed: &[&str]) {
    let parsed = parse_names(&mut user.chars(), false).unwrap();
    assert_eq!(parsed, expected_collapsed);
    assert_eq!(user, collapse_names(parsed));
}

#[test]
/// #TST-partof-load
fn test_parse_names() {
    do_test_parse_collapse("hi, ho", &["hi", "ho"]);
    do_test_parse_collapse("hi-[he, ho]", &["hi-he", "hi-ho"]);
    do_test_parse_collapse("he-[ha-[ha, he], hi, ho], hi-[he, ho]",
                           &["he-ha-ha", "he-ha-he", "he-hi", "he-ho", "hi-he", "hi-ho"]);
    assert!(parse_names(&mut "[]".chars(), false).is_err());
    assert!(parse_names(&mut "[hi]".chars(), false).is_err());
    assert!(parse_names(&mut "hi-[ho, [he]]".chars(), false).is_err());
    assert!(parse_names(&mut "hi-[ho, he".chars(), false).is_err());
}
