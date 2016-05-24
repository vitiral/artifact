#![allow(dead_code, unused_imports, unused_variables)]

use std::ascii::AsciiExt;
use std::fs;
use std::clone::Clone;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::collections::{HashMap, HashSet, VecDeque};

// Traits
use std::io::{Read, Write};
use std::fmt::Write as WriteStr;
use std::iter::FromIterator;

use toml::{Parser, Value, Table};

use super::*;  // data directory constants
use super::super::types::*;
use super::super::load::*;

// // Tests

#[test]
/// LOC-tst-name-check:<check that name combinations raise correct errors>
fn test_artifact_name() {
    // valid names
    for name in vec!["REQ-foo", "REQ-foo-2", "REQ-foo2", "REQ-foo2", "REQ-foo-bar-2_3",
                     "SPC-foo", "RSK-foo", "TST-foo", "LOC-foo"] {
        assert!(ArtName::from_str(name).is_ok());
    }
    for name in vec!["REQ-foo*", "REQ-foo\n", "REQ-foo-"] {
        assert!(ArtName::from_str(name).is_err())
    }
    // remove spaces
    assert_eq!(ArtName::from_str("   R E Q    -    f   o  o   ").unwrap().value, ["REQ", "FOO"]);
}

#[test]
fn test_get_attr() {
    let tbl_good = parse_text(TOML_GOOD);
    let df_str = "".to_string();
    let df_tbl = Table::new();
    let ref df_vec: Vec<String> = Vec::new();

    // LOC-tst-core-load-attrs-unit-1:<Test loading valid existing types>
    let test = get_attr!(tbl_good, "REQ-bar", df_tbl, Table).unwrap();
    assert!(get_attr!(&test, "disabled", false, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "disabled", true, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "bar");
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "bar");
    assert!(get_vecstr(&test, "refs", df_vec).unwrap() == ["hello", "ref"]);

    // LOC-tst-core-load-attrs-unit-2:<Test loading invalid existing types>
    assert!(get_attr!(&test, "disabled", df_str, String).is_none());
    assert!(get_attr!(&test, "text", false, Boolean).is_none());
    assert!(get_vecstr(&test, "text", df_vec).is_none());
    let test = get_attr!(tbl_good, "SPC-foo", Table::new(), Table).unwrap();
    assert!(get_vecstr(&test, "refs", df_vec).is_none());

    // LOC-tst-core-load-attrs-unit-3:<Test loading valid default types>
    let test = get_attr!(tbl_good, "REQ-foo", Table::new(), Table).unwrap();
    assert!(get_attr!(&test, "disabled", false, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "");
}

#[test]
fn test_check_type() {
    let tbl_good = parse_text(TOML_GOOD);
    let df_tbl = Table::new();

    let test = get_attr!(tbl_good, "REQ-bar", df_tbl, Table).unwrap();
    // LOC-tst-core-load-attrs-unit-1:<Test loading valid type>
    fn check_valid(test: &Table) -> LoadResult<Vec<String>> {
        Ok(check_type!(get_vecstr(test, "refs", &Vec::new()), "refs", "name"))
    }
    assert!(check_valid(&test).is_ok());

    let test = get_attr!(tbl_good, "SPC-foo", df_tbl, Table).unwrap();
    fn check_invalid(test: &Table) -> LoadResult<Vec<String>> {
        Ok(check_type!(get_vecstr(test, "refs", &Vec::new()), "refs", "name"))
    }
    assert!(check_invalid(&test).is_err());
}

#[test]
fn test_settings() {
    let tbl_good = parse_text(TOML_GOOD);
    let df_tbl = Table::new();
    let set = Settings::from_table(
        &get_attr!(tbl_good, "settings", df_tbl, Table).unwrap()).unwrap();
    assert!(set.paths ==
            VecDeque::from_iter(vec![PathBuf::from("{cwd}/test"), PathBuf::from("{repo}/test")]));
    assert!(set.disabled == false);
    let mut expected = HashSet::new();
    expected.insert(".test".to_string());
    assert!(set.repo_names == expected);
}


#[test]
/// LOC-tst-core-load-valid:<load some valid toml files>
fn test_load_toml() {
    let mut artifacts = Artifacts::new();
    let mut settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut variables: Vec<(PathBuf, Variables)> = Vec::new();

    let path = PathBuf::from("hi/there");

    // LOC-tst-core-load-invalid:<load some invalid toml files>
    assert!(load_toml(&path, TOML_BAD, &mut artifacts, &mut settings, &mut variables).is_err());

    let num = load_toml(&path, TOML_RSK, &mut artifacts, &mut settings, &mut variables).unwrap();
    assert_eq!(num, 8);
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-foo").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("SPC-foo").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("RSK-foo").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("TST-foo").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-bar").unwrap()));

    // will be loaded later
    assert!(!artifacts.contains_key(&ArtName::from_str("REQ-baz").unwrap()));
    assert!(!artifacts.contains_key(&ArtName::from_str("RSK-foo-2").unwrap()));
    assert!(!artifacts.contains_key(&ArtName::from_str("TST-foo-2").unwrap()));

    {
        // test defaults
        let art = artifacts.get(&ArtName::from_str("RSK-foo").unwrap()).unwrap();
        assert_eq!(art.ty, ArtType::RSK);
        assert_eq!(art.path, path);
        assert_eq!(art.text, "");
        let expected: Vec<String> = Vec::new();
        assert_eq!(art.refs, expected);
        let expected: HashSet<ArtName> = HashSet::new();
        assert_eq!(art.partof, expected);
        assert_eq!(art.loc, None);
        assert_eq!(art.completed, -1.0);
        assert_eq!(art.tested, -1.0);

        // test non-defaults
        let art = artifacts.get(&ArtName::from_str("REQ-bar").unwrap()).unwrap();
        assert_eq!(art.ty, ArtType::REQ);
        assert_eq!(art.path, path);
        assert_eq!(art.text, "bar");
        assert_eq!(art.refs, ["hello", "ref"]);
        let expected = ["REQ-Foo", "REQ-Bar-1", "REQ-Bar-2", "tst-foo"]
            .iter().map(|n| ArtName::from_str(n).unwrap()).collect();
        assert_eq!(art.partof, expected);
        let expected = Loc{
            loc: ArtName::from_str("LOC-Foo").unwrap(),
            path: PathBuf::from("{core}/foo.rs")};
        assert_eq!(art.loc.as_ref().unwrap(), &expected);
        assert_eq!(art.completed, -1.0);
        assert_eq!(art.tested, -1.0);
    }

    // REQ-foo already exists, so this must throw an error
    assert!(load_toml(&path, TOML_OVERLAP, &mut artifacts, &mut settings, &mut variables).is_err());

    let num = load_toml(&path, TOML_RSK2, &mut artifacts, &mut settings, &mut variables).unwrap();
    assert_eq!(num, 3);
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-baz").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("RSK-foo-2").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("TST-foo-2").unwrap()));
}

#[test]
fn test_load_path() {
    let (artifacts, settings) = load_path(TSIMPLE_DIR.as_path()).unwrap();
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-purpose").unwrap()));

    let req_purpose = artifacts.get(&ArtName::from_str("REQ-purpose").unwrap()).unwrap();

    // load all artifacts that should exist
    // LOC-core-load-dir-unit-1
    let req_lvl1 = artifacts.get(&ArtName::from_str("REQ-lvl-1").unwrap()).unwrap();
    let spc_lvl1 = artifacts.get(&ArtName::from_str("SPC-lvl-1").unwrap()).unwrap();

    let req_lvl2 = artifacts.get(&ArtName::from_str("REQ-lvl-2").unwrap()).unwrap();
    let spc_lvl2 = artifacts.get(&ArtName::from_str("SPC-lvl-2").unwrap()).unwrap();
    let tst_lvl2 = artifacts.get(&ArtName::from_str("TST-lvl-2").unwrap()).unwrap();
    let loc_lvl2 = artifacts.get(&ArtName::from_str("LOC-lvl-2").unwrap()).unwrap();
    let loc_tst_lvl2 = artifacts.get(&ArtName::from_str("LOC-tst-lvl-2").unwrap()).unwrap();

    // deep loading
    // LOC-tst-core-deep
    // LOC-core-load-dir-unit-4
    assert!(!artifacts.contains_key(&ArtName::from_str("REQ-unreachable").unwrap()));

    let req_deep = artifacts.get(&ArtName::from_str("REQ-deep").unwrap()).unwrap();
    let scp_deep = artifacts.get(&ArtName::from_str("SPC-deep").unwrap()).unwrap();

    let simple_dir_str = TSIMPLE_DIR.as_path().to_str().unwrap().to_string();
    let extra_dir = TSIMPLE_DIR.join(PathBuf::from("extra"));
    let lvl1_dir = TSIMPLE_DIR.join(PathBuf::from("lvl_1"));
    let lvl1_dir_str = lvl1_dir.as_path().to_str().unwrap().to_string();

    // LOC-core-load-dir-unit-5
    assert_eq!(req_purpose.refs, [extra_dir.join(PathBuf::from("README.md")).to_str().unwrap()]);
    assert_eq!(spc_lvl1.text, "level one does FOO");
    assert_eq!(spc_lvl1.loc.as_ref().unwrap().path, lvl1_dir.join(PathBuf::from("lvl_1.rs")));
    // TODO: more validation
}
