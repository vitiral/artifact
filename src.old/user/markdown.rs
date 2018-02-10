/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Deserialize from markdown.
//!
//! The markdown format must be as follows:
//! ```
//! # ART-name
//! - partof:       # partof links
//!     - ART-1
//!     - ART-2
//!     - ART-3
//! - done: 2       # optional
//! ###
//! This is the text for the artifact.
//! It is regular markdown
//!
//! # ART-name2
//! This is another artifact, if `###` is not given then the text starts immediately.
//! ```

use std::collections::BTreeMap;
use serde_yaml;

use dev_prefix::*;
use types::*;
use user::types::{UserArtifact, UserPartof};

lazy_static! {
    static ref NAME_HEADER_RE: Regex = Regex::new(
        &format!(r"(?i)^#\s*({})\s*$", NAME_VALID_STR)
    ).unwrap();
    static ref DATA_END_RE: Regex = Regex::new(r"^###+\s*$").unwrap();
}

#[derive(Debug, Eq, PartialEq)]
struct MdArtifact<'a> {
    name: &'a str,
    attrs: &'a str,
    text: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MdAttrs {
    partof: Option<UserPartof>,
    done: Option<String>,
}

impl<'a> MdArtifact<'a> {
    fn new(
        s: &'a str,
        name: &'a str,
        name_end: usize,
        attrs_end: usize,
        mut text_start: usize,
        current: usize,
    ) -> MdArtifact<'a> {
        if text_start > current {
            text_start = current;
        }
        let (attrs, text) = if attrs_end == usize::max_value() {
            // no attrs, so everything from name-end is just text
            ("", &s[name_end + 1..current])
        } else {
            (&s[name_end + 1..attrs_end], &s[text_start..current])
        };
        MdArtifact {
            name: name,
            attrs: attrs,
            text: text.trim_right_matches('\n'),
        }
    }
}

// ------------------------
// Read

/// parse markdown into raw artifact data
pub fn from_markdown(s: &str) -> Result<BTreeMap<String, UserArtifact>> {
    let mut out: BTreeMap<String, UserArtifact> = BTreeMap::new();
    let mut raw = parse_raw(s)?;
    for md in raw.drain(0..) {
        // sanitize attrs to a map and deseriailze
        let (done, partof) = if md.attrs.is_empty() {
            (None, None)
        } else {
            let attrs = match serde_yaml::from_str(md.attrs)? {
                // serde_yaml::Value::Sequence(s) => sanitize_attrs(s)?,
                serde_yaml::Value::Mapping(m) => m,
                _ => bail!(ErrorKind::Load("attrs must be in map form".into())),
            };
            let attrs: MdAttrs = serde_yaml::from_value(serde_yaml::Value::Mapping(attrs))?;
            (attrs.done, attrs.partof)
        };
        let text = if md.text.is_empty() {
            None
        } else {
            Some(md.text.into())
        };
        let ua = UserArtifact {
            done: done,
            partof: partof,
            text: text,
        };
        out.insert(md.name.into(), ua);
    }
    Ok(out)
}

/// Split the text into "tokens" containing name, attrs and text
fn parse_raw(s: &str) -> Result<Vec<MdArtifact>> {
    let mut out = Vec::new();
    let mut current: usize = 0;
    let mut name_end = usize::max_value();
    let mut current_name: &str = "";
    let mut attrs_end = usize::max_value();
    let mut text_start = usize::max_value();

    for line in s.split('\n') {
        if let Some(cap) = NAME_HEADER_RE.captures(line) {
            if name_end != usize::max_value() {
                // We found a new name, first store the last one
                out.push(MdArtifact::new(
                    s,
                    current_name,
                    name_end,
                    attrs_end,
                    text_start,
                    current,
                ));
            }
            name_end = current + line.len();
            current_name = cap.get(1).unwrap().as_str();
            attrs_end = usize::max_value();
            text_start = usize::max_value();
        } else if DATA_END_RE.is_match(line) {
            if attrs_end != usize::max_value() {
                // FIXME: raise an error, double attrs without new name
            }
            attrs_end = current;
            text_start = current + line.len() + 1;
        }

        current += line.len() + 1; // +1 == the '\n' character
    }

    if name_end != usize::max_value() {
        // Append the final value
        out.push(MdArtifact::new(
            s,
            current_name,
            name_end,
            attrs_end,
            text_start,
            current - 1, // -1 because there was no '\n' character in last one
        ));
    }

    Ok(out)
}

// ------------------------
// Write

fn to_yaml<S: ::serde::Serialize>(value: &S) -> String {
    let mut s = serde_yaml::to_string(value).unwrap();
    s.drain(0..4); // remove the ---\n
    s
}

/// Given a sorted map of `name: UserArtifact` convert to markdown
pub fn to_markdown(artifacts: &BTreeMap<String, UserArtifact>) -> Result<String> {
    let mut out = String::new();
    let mut first = true;
    for (name, raw) in artifacts {
        if !first {
            write!(out, "\n")?;
        }
        first = false;

        write!(out, "# {}\n", name)?;
        if raw.done.is_some() || raw.partof.is_some() {
            // convert from map -> vec of maps
            let mut attrs_vec = Vec::new();
            if let Some(ref done) = raw.done {
                attrs_vec.push(to_yaml(&btreemap!{"done" => done}));
            }
            if let Some(ref partof) = raw.partof {
                let mut partof_str = String::from("partof:");
                match *partof {
                    UserPartof::Single(ref s) => write!(partof_str, " {}", s)?,
                    UserPartof::Multi(ref v) => {
                        write!(partof_str, "\n")?;
                        for p in v {
                            write!(partof_str, "- {}\n", p)?;
                        }
                        partof_str.pop(); // pop last newline
                    }
                };
                attrs_vec.push(partof_str);
            }
            // keep whitespace between them
            if !attrs_vec.is_empty() {
                let mut first = true;
                for a in &attrs_vec {
                    if !first {
                        write!(out, "\n")?;
                    }
                    write!(out, "{}\n", a)?;
                    first = false;
                }
                write!(out, "###\n")?;
            }
        }
        if let Some(ref text) = raw.text {
            write!(out, "{}\n", text.trim_right_matches('\n'))?;
        }
    }
    Ok(out)
}


// ------------------------
// Tests

#[test]
fn test_parse_raw() {
    let md = r#"

$ REQ-foo
foo attrs
foo more attrs
$$$
This is some text, yay

more text
$SPC-short
some short text
$ SPC-empty
$ TST-attrs
some attrs for tests
but no text
$$$
$ TST-final
This is the final item and just
has text
"#;
    let req_foo = MdArtifact {
        name: "REQ-foo",
        attrs: "foo attrs\nfoo more attrs\n",
        text: "This is some text, yay\n\nmore text",
    };
    let spc_short = MdArtifact {
        name: "SPC-short",
        attrs: "",
        text: "some short text",
    };
    let spc_empty = MdArtifact {
        name: "SPC-empty",
        attrs: "",
        text: "",
    };
    let tst_attrs = MdArtifact {
        name: "TST-attrs",
        attrs: "some attrs for tests\nbut no text\n",
        text: "",
    };
    let tst_final = MdArtifact {
        name: "TST-final",
        attrs: "",
        text: "This is the final item and just\nhas text",
    };

    let md = md.replace('$', "#");
    let result = parse_raw(&md).unwrap();
    assert_eq!(req_foo, result[0]);
    assert_eq!(spc_short, result[1]);
    assert_eq!(spc_empty, result[2]);
    assert_eq!(tst_attrs, result[3]);
    assert_eq!(tst_final, result[4]);
}

#[test]
fn test_from_markdown() {
    let example = r#"
# REQ-foo
partof:
- REQ-1
- REQ-2
###
Some text for foo

# SPC-foo
This is some text

# REQ-done
partof: REQ-foo

done: this is done
###
"#;
    let result = from_markdown(example).unwrap();
    let req_foo = UserArtifact {
        done: None,
        partof: Some(UserPartof::Multi(vec!["REQ-1".into(), "REQ-2".into()])),
        text: Some("Some text for foo".into()),
    };
    let spc_foo = UserArtifact {
        done: None,
        partof: None,
        text: Some("This is some text".into()),
    };

    assert_eq!(result["REQ-foo"], req_foo);
    assert_eq!(result["SPC-foo"], spc_foo);
}

#[test]
fn test_roundtrip() {
    let example = r#"# REQ-done
done: this is done

partof: REQ-foo
###

# REQ-foo
partof:
- REQ-1
- REQ-2
###
Some text for foo

# SPC-foo
This is some text
"#;
    let result = to_markdown(&from_markdown(example).unwrap()).unwrap();
    assert_eq!(example, result);
}
