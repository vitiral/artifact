//! Unit/Fuzz Tests:
//! - #TST-unit.name
//! - #TST-fuzz.name
extern crate artifact_test;
use artifact_test::name::*;
use artifact_test::*;

#[test]
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
