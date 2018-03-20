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

//! Test serializing/deserializing raw names

use ergo::{json, toml, yaml};

use raw_names::NamesRaw;
use test::dev_prelude::*;
use test::name::arb_name;

pub fn arb_names_raw(size: usize) -> BoxedStrategy<NamesRaw> {
    prop::collection::hash_set(arb_name(), 0..size)
        .prop_map(|hs| NamesRaw::from(hs))
        .boxed()
}

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
        PartofMulti { partof: vec!["REQ-foo".into()] },
        PartofMulti { partof: vec!["REQ-foo".into(), "SPC-bar".into()] },
        PartofMulti { partof: vec!["REQ-foo".into(), "REQ-bar".into()] },
        PartofMulti { partof: vec!["REQ-[foo, baz]".into(), "REQ-bar".into()] },
        PartofMulti { partof: vec!["REQ-[foo, bar]".into(), "REQ-bar".into()] },
    ];
}

/// conv: the conversion expression from the struct -> Names
/// module: the serialization module to test
macro_rules! assert_partof_serde {
    ($values:expr, $conv:expr, $module:tt) => (
        {
            // convert the singles to the serialization's format
            let singles = $values
                .iter()
                .map(|s| ($module::to_string(s).unwrap(), Some($conv(&s.partof))))
                .collect::<Vec<_>>();

            fn from_str(s: &String) -> StrResult<NamesRaw> {
                println!("Deserializing {}:\n{}", stringify!($module), s);
                let s: PartofNames = $module::from_str(s)
                    .map_err(|e| e.to_string())?;
                Ok(s.partof)
            }

            assert_generic(from_str, singles.as_slice());
        }
    );
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
