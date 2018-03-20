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
//! #TST-name
//!
//! This module defines all operations around testing artifact names

use ergo::json;

use test::dev_prelude::*;

// HELPERS and TRAITS

// this purposely doesn't use the definition from `name.rs`
const GEN_NAME_RE: &str = r#"(?x)
(REQ|SPC|TST)-              # the type followed by `-`
([a-zA-Z0-9_]{1,7}-){0,3}     # an optional number of `elem-` elements
[a-zA-Z0-9_]{1,7}             # required final element
"#;

// lazy_static!{
//     static ref GEN_NAME_PROP: Arc<prop::string::RegexGeneratorStrategy<String>> =
//         Arc::new(prop::string::string_regex(GEN_NAME_RE).unwrap());
// }

#[inline(always)]
pub fn arb_name_string() -> BoxedStrategy<String> {
    GEN_NAME_RE.prop_map(|s| s.to_string()).boxed()
}

#[inline(always)]
pub fn arb_name() -> BoxedStrategy<Name> {
    arb_name_string().prop_map(|n| name!(n)).boxed()
}

/// Return a vector of the `raw` names
pub fn names_raw(names: &[Name]) -> Vec<String> {
    names.iter().map(|n| n.raw.clone()).collect()
}

/// Assert that the name is valid
fn assert_names_valid(raw: &[&str]) {
    let errors = raw.iter()
        .map(|r| (*r, Name::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(name) => if raw == name.raw {
                None
            } else {
                panic!("raw was different: {} => {}", raw, name.raw);
            },
            Err(_) => Some(raw),
        })
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("The following names were not valid:\n{:#?}", errors);
    }
}

/// Assert that the name is valid
fn assert_names_invalid(raw: &[&str]) {
    let errors = raw.iter()
        .map(|r| (r, Name::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(_) => Some(raw),
            Err(_) => None,
        })
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!(
            "The following names were valid but shouldn't have been:\n{:#?}",
            errors
        );
    }
}

// SANITY TESTS

#[test]
/// #TST-name.sanity_valid
fn sanity_names_valid() {
    assert_names_valid(&[
        "REQ-a",
        "REQ-a-b",
        "REQ-foo",
        "REQ-foo_bar",
        "SPC-foo",
        "TST-foo",
        "TST-Foo",
        "TST-FoO",
        "tst-FOO",
        "tst-foo",
        "TST-bPRJM_07msqpQ",
        "TST-bPRJM07msqpQ-pRMBtV-HJmJOpEgFTI2p8zdEMpluTbnkepzdELxf5CntsW",
    ]);
    assert_eq!(name!("REQ-foo").ty, Type::REQ);
    assert_eq!(name!("SPC-foo").ty, Type::SPC);
    assert_eq!(name!("TST-foo").ty, Type::TST);
    assert_eq!(name!("tSt-foo").ty, Type::TST);
}

#[test]
/// #TST-name.sanity_invalid
fn sanity_names_invalid() {
    assert_names_invalid(&[
        "RSK-foo",
        "REQ",
        "REQ-",
        "REQ-a-",
        "REQ-a--",
        "REQ-a-b-",
        "REQ--a",
        "REQ-a--b",
        "REQ-a--b-",
        "REQ-a.b",
        "REQ-a_b.",
        "REQ",
        "SPC",
        "TST",
        "hello",
        "",
        "a",
    ]);
}

#[test]
fn sanity_subnames() {
    let subnames: &[(String, Option<SubName>)] = &[
        // Valid
        (
            ".foo".into(),
            Some(SubName(Arc::new(InternalSubName {
                raw: ".foo".into(),
                key: ".FOO".into(),
            }))),
        ),
        (
            ".foo_bar".into(),
            Some(SubName(Arc::new(InternalSubName {
                raw: ".foo_bar".into(),
                key: ".FOO_BAR".into(),
            }))),
        ),
        (".BAR".into(), Some(subname!(".bar"))), // only keys matter
        (".bar".into(), Some(subname!(".bAr"))), // only keys matter
        // Invalid
        ("foo".into(), None),             // no period
        (".foo-bar".into(), None),        // `-` not allowed
        ("REQ-foo".into(), None),         // full artifact not allowed
        ("REQ-foo.foo-bar".into(), None), // full+subname not allowed
    ];

    fn subname_valid(s: &String) -> StrResult<SubName> {
        SubName::from_str(s).map_err(|e| e.to_string())
    }

    assert_generic(subname_valid, subnames);
}

#[test]
fn sanity_parse_subnames() {
    let text = r#"
This is some text
subname: [[.subname]].
[[.a]]
[[.a_b]]
[[.not-valid]]
[[REQ-foo.not-subname]]
"#;
    let subnames = parse_subnames(text);
    let expected = indexset!{
        subname!(".subname"),
        subname!(".a"),
        subname!(".a_b"),
    };
    assert_eq!(subnames, expected);
}

#[test]
/// #TST-name.sanity_serde
fn sanity_serde_name() {
    let json = r#"["REQ-foo","REQ-FOO","REQ-bar","SPC-foo-bar","tst-foo-BAR"]"#;
    let expected = &[
        "REQ-foo",
        "REQ-FOO",
        "REQ-bar",
        "SPC-foo-bar",
        "tst-foo-BAR",
    ];
    assert_eq!(json, json::to_string(expected).unwrap());
    let names: Vec<Name> = json::from_str(&json).unwrap();
    let result = names_raw(&names);
    assert_eq!(expected, result.as_slice());
}

proptest! {
    #[test]
    /// #TST-name.sanity_auto_partof
    fn fuzz_name_key(ref name in arb_name()) {
        let repr = name.key_str();
        let from_repr = Name::from_str(&repr).unwrap();
        assert_eq!(from_repr, *name);
        assert_eq!(repr, from_repr.key_str())
    }

    #[test]
    fn fuzz_name_serde(ref name in arb_name()) {
        let json = format!("\"{}\"", name.as_str());
        let result_json = json::to_string(&name).unwrap();
        let result = json::from_str(&json).unwrap();
        assert_eq!(*name, result);
        assert_eq!(json, result_json);
    }
}
