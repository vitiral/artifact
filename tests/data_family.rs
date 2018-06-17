//! Unit/Fuzz Tests:
//! - #TST-unit.family
//! - #TST-fuzz.family
//!
//! This also inclused auto partofs as well as collapsing/expanding partof.

extern crate artifact_test;
use artifact_test::family::*;
use artifact_test::name::arb_name;
use artifact_test::*;

#[test]
fn sanity_parent() {
    fn parent(name: &Name) -> StrResult<Name> {
        match name.parent() {
            Some(n) => Ok(n),
            None => Err("no parent".into()),
        }
    }

    assert_generic(
        parent,
        &[
            // no parents
            (name!("REQ-foo"), None),
            (name!("TST-a"), None),
            (name!("TST-23kjskljef32"), None),
            // has parents
            (name!("REQ-a-b"), Some(name!("REQ-a"))),
            (name!("REQ-A-B"), Some(name!("REQ-A"))),
            (name!("REQ-aasdf-bbSdf-DES"), Some(name!("REQ-aasdf-bbSdf"))),
        ],
    );
}

#[test]
fn sanity_auto_partof() {
    fn auto_partof(name: &Name) -> StrResult<Name> {
        match name.auto_partof() {
            Some(n) => Ok(n),
            None => Err("no auto partof".into()),
        }
    }
    assert_generic(
        auto_partof,
        &[
            (name!("REQ-foo"), None),
            (name!("REQ-a-b"), None),
            (name!("REQ-A-B"), None),
            (
                name!("spc-aasdf-bbSdf-DES"),
                Some(name!("REQ-aasdf-bbSdf-DES")),
            ),
            (name!("TSt-a"), Some(name!("SPC-a"))),
            (name!("TST-23kjskljef32"), Some(name!("SPC-23kjskljef32"))),
        ],
    );
}

#[test]
fn sanity_collapse_name() {
    let values = &[
        ("REQ-foo", None, indexset!["REQ-foo"]),
        ("REQ-[bar, foo]", None, indexset!["REQ-foo", "REQ-bar"]),
        (
            "REQ-[zay, bar-[baz, bom], foo]",
            Some("REQ-[bar-[baz, bom], foo, zay]"),
            indexset!["REQ-foo", "REQ-bar-baz", "REQ-bar-bom", "REQ-zay"],
        ),
        (
            "SPC-[foo, foo-bob, bar], REQ-baz, SPC-foo-baz",
            Some("REQ-baz, SPC-[bar, foo, foo-[baz, bob]]"),
            indexset![
                "REQ-baz",
                "SPC-bar",
                "SPC-foo",
                "SPC-foo-baz",
                "SPC-foo-bob",
            ],
        ),
    ];
    assert_collapsed_valid(values);
}

#[test]
fn sanity_collapse_name_invalid() {
    let values = &[
        "REQ",                       // invalid name
        "REQ-foo-[bar-, baz]",       // extra `-`
        "REQ-[foo, [bar]]",          // `[` can't appear by itself
        "SPC-foo[bar",               // no closing brace
        "SPC-foo]",                  // no opening brace
        "SPC-[foo]]",                // no opening brace
        "SPC-foo-[bar, [baz, bom]]", // brackets not after `-`
    ];
    assert_collapsed_invalid(values);
}

#[test]
fn sanity_auto_partofs() {
    let req_foo = name!("REQ-foo");
    let req_foo_bar = name!("REQ-foo-bar");
    let spc_foo = name!("SPC-foo");
    let tst_foo = name!("TST-foo");
    let tst_foo_bar = name!("TST-foo-bar");

    let spc_a_b = name!("SPC-a-b");
    let tst_a_b = name!("TST-a-b");

    let file = PathAbs::mock("/fake");

    let names = indexmap!{
        req_foo.clone() => file.clone(),
        req_foo_bar.clone() => file.clone(),
        spc_foo.clone() => file.clone(),
        tst_foo.clone() => file.clone(),
        tst_foo_bar.clone() => file.clone(),
        spc_a_b.clone() => file.clone(),
        tst_a_b.clone() => file.clone(),
    };

    let expected = indexmap!{
        req_foo.clone() => indexset![],
        req_foo_bar.clone() => indexset![req_foo.clone()],
        spc_foo.clone() => indexset![req_foo.clone()],
        tst_foo.clone() => indexset![spc_foo.clone()],
        // contains no auto -- it doesn't exist
        tst_foo_bar.clone() => indexset![tst_foo.clone()],
        spc_a_b.clone() => indexset![],
        tst_a_b.clone() => indexset![spc_a_b.clone()],
    };
    let auto = auto_partofs(&names);
    assert_eq!(expected, auto);
}

#[test]
fn sanity_strip_auto_partofs() {
    let mut result = indexset![
        name!("REQ-bar"),
        name!("REQ-foo"),
        name!("REQ-foo-bar"),
        name!("SPC-bar"),
        name!("SPC-foo"),
    ];
    let expected = indexset![name!("REQ-bar"), name!("REQ-foo"), name!("SPC-bar"),];
    strip_auto_partofs(&name!("SPC-foo-bar"), &mut result);
    assert_eq!(expected, result);
}

proptest! {
    #[test]
    fn fuzz_name_parent(ref name in arb_name()) {
        // Basically do the same thing as the code but in a slightly different way
        let mut items = name.raw.split('-').map(|s| s.to_string()).collect::<Vec<_>>();
        if items.len() > 2 {
            items.pop();
            let expected_raw = items.join("-");
            let expected = Name::from_str(&expected_raw).unwrap();
            let result = name.parent().unwrap();
            assert_eq!(expected_raw, result.raw);
            assert_eq!(expected, result);
        } else {
            assert!(name.parent().is_none());
        }
    }

    #[test]
    fn fuzz_name_auto_partof(ref name in arb_name()) {
        let ty = match name.ty {
            Type::REQ => {
                assert!(name.auto_partof().is_none());
                return Ok(());
            },
            Type::SPC => "REQ",
            Type::TST => "SPC",
        };
        let mut items = name.raw.split('-').map(|s| s.to_string()).collect::<Vec<_>>();
        items[0] = ty.into();
        let expected_raw = items.join("-");
        let expected = Name::from_str(&expected_raw).unwrap();
        let result = name.auto_partof().unwrap();
        assert_eq!(expected_raw, result.raw);
        assert_eq!(expected, result);
    }

    #[test]
    fn fuzz_collapse_name_roundtrip(ref names in arb_names(25)) {
        let collapsed = collapse_names(names);
        let expanded = expand_names(&collapsed).expect("failed expanding names");
        assert_eq!(*names, Names::from(expanded))
    }

    #[test]
    /// This actually creates expected json by first sorting the names
    fn fuzz_names_serde(ref names in arb_names(25)) {
        // construct expected json by sorting and formatting
        let expected_json = {
            let mut sorted = names.iter().cloned().collect::<Vec<_>>();
            sorted.sort();
            let strs = sorted.iter()
                .map(|s| format!("{:?}", s.as_str()))
                .collect::<Vec<_>>();
            format!("[{}]", strs.join(","))
        };
        // do serde-roundtrip as well
        let result_json = json::to_string(&names).unwrap();
        let result: Names = json::from_str(&result_json).unwrap();
        assert_eq!(*names, result);
        assert_eq!(expected_json, result_json);
    }
}
