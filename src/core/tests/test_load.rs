use std::env;
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

use walkdir::WalkDir;
use toml::{Parser, Value, Table};
use strfmt::strfmt;

use super::*;  // data directory constants
use super::super::types::*;
use super::super::load::*;

// Data and helpers

// valid toml, not necessarily all valid artifacts
static TOML_GOOD: &'static str = "
[settings]
disabled = false
paths = ['{cwd}/test', '{repo}/test']
repo_names = ['.test']

[REQ-foo]
disabled = false
[SPC-foo]
refs = [1, 2]
[RSK-foo]
[TST-foo]
[REQ-bar]
text = 'bar'
disabled = false
refs = [\"hello\", \"ref\"]
";

// valid rsk file
static TOML_RSK: &'static str = "
[settings]
disabled = false
paths = ['{cwd}/data/empty']
repo_names = ['.test']

[REQ-foo]
disabled = false
[SPC-foo]
refs = ['1', '2']
[RSK-foo]
[TST-foo]
[REQ-bar]
disabled = false
partof = 'REQ-[foo, bar-[1,2]], TST-foo'
refs = [\"hello\", \"ref\"]
text = 'bar'
loc = 'LOC-foo: {core}/foo.rs'
";

static TOML_RSK2: &'static str = "
[settings]
paths = ['test/path']
repo_names = ['.tst']
[REQ-baz]
[RSK-foo-2]
[TST-foo-2]
";

static TOML_BAD: &'static str = "[REQ-bad]\nrefs = 'REQ-foo'";  // invalid type
static TOML_OVERLAP: &'static str = "[REQ-foo]\n";

fn parse_text(t: &str) -> Table {
    Parser::new(t).parse().unwrap()
}

fn get_table<'a>(tbl: &'a Table, attr: &str) -> &'a Table {
    match tbl.get(attr).unwrap() {
        &Value::Table(ref t) => t,
        _ => unreachable!()
    }
}

// // Tests

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
fn test_load_toml() {
    let mut artifacts = Artifacts::new();
    let mut settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut variables: Vec<(PathBuf, Variables)> = Vec::new();

    let path = PathBuf::from("hi/there");

    assert!(load_toml(&path, TOML_BAD, &mut artifacts, &mut settings, &mut variables).is_err());

    let num = load_toml(&path, TOML_RSK, &mut artifacts, &mut settings, &mut variables).unwrap();
    assert_eq!(num, 5);
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
        assert_eq!(art.completed, None);
        assert_eq!(art.tested, None);

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
        assert_eq!(art.completed, None);
        assert_eq!(art.tested, None);
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
fn test_find_repo() {
    let mut repo_names = HashSet::new();
    println!("dir: {:?}", TSIMPLE_DIR.as_path());
    repo_names.insert(String::from(".tst_repo_name"));
    assert_eq!(find_repo(TSIMPLE_DIR.as_path(), &repo_names).unwrap(),
               TSIMPLE_DIR.as_path());
    assert_eq!(find_repo(TSIMPLE_DIR.join("lvl_1").as_path(), &repo_names).unwrap(),
               TSIMPLE_DIR.as_path());
    assert!(find_repo(env::temp_dir().as_path(), &repo_names).is_none());
}

#[test]
fn test_load_path() {
    let (artifacts, settings, variables, repo_map) = load_path(TSIMPLE_DIR.as_path()).unwrap();
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-purpose").unwrap()));

    // lvl loaded
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-lvl-1").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-lvl-2").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("SPC-lvl-2").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("TST-lvl-2").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("LOC-lvl-2").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("LOC-tst-lvl-2").unwrap()));

    // deep loading
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-deep").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("SPC-deep").unwrap()));

    let simple_dir_str = TSIMPLE_DIR.as_path().to_str().unwrap().to_string();

    // variables
    assert_eq!(variables.get("lvl_1").unwrap(), &(simple_dir_str + "/lvl_1"))
}
