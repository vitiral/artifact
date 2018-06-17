//! Unit/Fuzz Tests:
//! - #TST-unit.raw_name
//! - #TST-fuzz.raw_name
extern crate artifact_test;
use artifact_test::artifact_data::raw_names::NamesRaw;
use artifact_test::raw_names::*;
use artifact_test::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartofSingle<'a> {
    partof: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartofMulti {
    partof: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartofNames {
    partof: NamesRaw,
}

static SINGLE_PARTOFS: &[PartofSingle] = &[
    PartofSingle { partof: "REQ-foo" },
    PartofSingle {
        partof: "REQ-foo, SPC-bar",
    },
    PartofSingle {
        partof: "REQ-[foo, bar]",
    },
    PartofSingle {
        partof: "SPC-[foo, bar, bar-[baz, bom]], REQ-baz",
    },
];

lazy_static! {
    static ref MULTI_PARTOFS: Vec<PartofMulti> = vec![
        PartofMulti {
            partof: vec!["REQ-foo".into()],
        },
        PartofMulti {
            partof: vec!["REQ-foo".into(), "SPC-bar".into()],
        },
        PartofMulti {
            partof: vec!["REQ-foo".into(), "REQ-bar".into()],
        },
        PartofMulti {
            partof: vec!["REQ-[foo, baz]".into(), "REQ-bar".into()],
        },
        PartofMulti {
            partof: vec!["REQ-[foo, bar]".into(), "REQ-bar".into()],
        },
    ];
}

/// conv: the conversion expression from the struct -> Names
/// module: the serialization module to test
#[macro_export]
macro_rules! assert_partof_serde {
    ($values:expr, $conv:expr, $module:tt) => {{
        // convert the singles to the serialization's format
        let singles = $values
            .iter()
            .map(|s| ($module::to_string(s).unwrap(), Some($conv(&s.partof))))
            .collect::<Vec<_>>();

        fn from_str(s: &String) -> StrResult<NamesRaw> {
            println!("Deserializing {}:\n{}", stringify!($module), s);
            let s: PartofNames = $module::from_str(s).map_err(|e| e.to_string())?;
            Ok(s.partof)
        }

        assert_generic(from_str, singles.as_slice());
    }};
}

fn names_raw_from_str(s: &str) -> NamesRaw {
    names_raw!(s)
}

fn names_raw_from_strs(s: &Vec<String>) -> NamesRaw {
    let mut out = IndexSet::new();
    for n in s {
        out.extend(names_raw!(n).iter().cloned())
    }
    NamesRaw::from(out)
}

#[test]
fn sanity_serde_names_raw_single_json() {
    assert_partof_serde!(SINGLE_PARTOFS, names_raw_from_str, json);
}

#[test]
fn sanity_serde_names_raw_multi_json() {
    assert_partof_serde!(MULTI_PARTOFS, names_raw_from_strs, json);
}

#[test]
fn sanity_serde_names_raw_single_toml() {
    assert_partof_serde!(SINGLE_PARTOFS, names_raw_from_str, toml);
}

#[test]
fn sanity_serde_names_raw_multi_toml() {
    assert_partof_serde!(MULTI_PARTOFS, names_raw_from_strs, toml);
}

#[test]
fn sanity_serde_names_raw_single_yaml() {
    assert_partof_serde!(SINGLE_PARTOFS, names_raw_from_str, yaml);
}

#[test]
fn sanity_serde_names_raw_multi_yaml() {
    assert_partof_serde!(MULTI_PARTOFS, names_raw_from_strs, yaml);
}

proptest! {
    #[test]
    /// This actually creates expected json by first sorting the names
    fn fuzz_names_raw_serde(ref names in arb_names_raw(25)) {
        prop_assume!(!names.is_empty());
        // construct expected json by sorting and formatting
        let expected_json = {
            let mut sorted = names.iter().cloned().collect::<Vec<_>>();
            sorted.sort();
            let strs = sorted.iter()
                .map(|s| format!("{:?}", s.as_str()))
                .collect::<Vec<_>>();
            if strs.len() == 1 {
                // if single item, just a string
                strs[0].clone()
            } else {
                format!("[{}]", strs.join(","))
            }
        };
        // do serde-roundtrip as well
        let result_json = json::to_string(&names).unwrap();
        let result: NamesRaw = json::from_str(&result_json).unwrap();
        assert_eq!(*names, result);
        assert_eq!(expected_json, result_json);
    }
}
