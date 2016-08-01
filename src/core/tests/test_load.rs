#![allow(dead_code, unused_imports, unused_variables)]

use std::ascii::AsciiExt;

use super::*;  // data directory constants
use super::super::init_logger_test;
use super::super::types::*;
use super::super::load::*;
use super::super::vars;
use super::super::locs::*;

// // Tests

#[test]
/// #TST-core-artifact-name-check:<check that name combinations raise correct errors>
fn test_artifact_name() {
    // valid names
    for name in vec!["REQ-foo", "REQ-foo-2", "REQ-foo2", "REQ-foo2", "REQ-foo-bar-2_3",
                     "SPC-foo", "RSK-foo", "TST-foo"] {
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

    // #TST-core-load-attrs-unit-1-a:<Test loading valid existing types>
    let test = get_attr!(tbl_good, "REQ-bar", df_tbl, Table).unwrap();
    assert!(get_attr!(&test, "disabled", false, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "disabled", true, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "bar");
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "bar");
    assert!(get_vecstr(&test, "refs", df_vec).unwrap() == ["hello", "ref"]);

    // #TST-core-load-attrs-unit-2:<Test loading invalid existing types>
    assert!(get_attr!(&test, "disabled", df_str, String).is_none());
    assert!(get_attr!(&test, "text", false, Boolean).is_none());
    assert!(get_vecstr(&test, "text", df_vec).is_none());
    let test = get_attr!(tbl_good, "SPC-foo", Table::new(), Table).unwrap();
    assert!(get_vecstr(&test, "refs", df_vec).is_none());

    // #TST-core-load-attrs-unit-3:<Test loading valid default types>
    let test = get_attr!(tbl_good, "REQ-foo", Table::new(), Table).unwrap();
    assert!(get_attr!(&test, "disabled", false, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "");
}

#[test]
fn test_check_type() {
    let tbl_good = parse_text(TOML_GOOD);
    let df_tbl = Table::new();

    let test = get_attr!(tbl_good, "REQ-bar", df_tbl, Table).unwrap();
    // #TST-core-load-attrs-unit-1-b:<Test loading valid type>
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
/// partof: #TST-settings-load
fn test_settings() {
    let tbl_good = parse_text(TOML_GOOD);
    let df_tbl = Table::new();
    let set = Settings::from_table(
        &get_attr!(tbl_good, "settings", df_tbl, Table).unwrap()).unwrap();
    assert!(set.paths == VecDeque::from_iter(
                vec![PathBuf::from("{cwd}/test"), PathBuf::from("{repo}/test")]));
    assert!(set.code_paths == VecDeque::from_iter(
        vec![PathBuf::from("{cwd}/src"), PathBuf::from("{repo}/src2")]));
    assert!(set.disabled == false);

    // see: 4
    let toml_invalid = r#"
    [settings]
    artifact_paths = ['hi']
    paths = ['invalid']
    "#;
    let tbl_invalid = parse_text(toml_invalid);
    let df_tbl = Table::new();

    assert!(Settings::from_table(&get_attr!(tbl_invalid,
                                            "settings", df_tbl, Table).unwrap()
                                 ).is_err());
}


#[test]
fn test_load_toml() {
    let mut artifacts = Artifacts::new();
    let mut settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut variables: Vec<(PathBuf, Variables)> = Vec::new();

    let path = PathBuf::from("hi/there");

    // #TST-load-toml-invalid
    assert!(load_toml(&path, TOML_BAD, &mut artifacts, &mut settings, &mut variables).is_err());
    assert!(load_toml(&path, TOML_BAD_ATTR1, &mut artifacts,
                      &mut settings, &mut variables).is_err());
    assert!(load_toml(&path, TOML_BAD_ATTR2, &mut artifacts,
                      &mut settings, &mut variables).is_err());
    assert!(load_toml(&path, TOML_BAD_NAMES1, &mut artifacts,
                      &mut settings, &mut variables).is_err());
    assert!(load_toml(&path, TOML_BAD_NAMES2, &mut artifacts,
                      &mut settings, &mut variables).is_err());

    // #TST-disabled-1
    assert_eq!(load_toml(&path, TOML_DISABLED, &mut artifacts,
                         &mut settings, &mut variables).unwrap(), 0);
    assert_eq!(artifacts.len(), 0);
    assert_eq!(settings.len(), 0);
    assert_eq!(variables.len(), 0);

    // #TST-artifact-load: basic loading unit tests
    let num = load_toml(&path, TOML_RSK, &mut artifacts, &mut settings, &mut variables).unwrap();
    let locs = HashMap::from_iter(
        vec![(ArtName::from_str("SPC-foo").unwrap(), Loc::fake()),
             (ArtName::from_str("SPC-bar").unwrap(), Loc::fake())]);
    attach_locs(&mut artifacts, &locs);
    assert_eq!(num, 8);
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-foo").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("SPC-foo").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("RSK-foo").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("TST-foo").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("SPC-bar").unwrap()));

    // will be loaded later
    assert!(!artifacts.contains_key(&ArtName::from_str("REQ-baz").unwrap()));
    assert!(!artifacts.contains_key(&ArtName::from_str("RSK-foo-2").unwrap()));
    assert!(!artifacts.contains_key(&ArtName::from_str("TST-foo-2").unwrap()));

    {
        // #TST-artifact-attrs-defaults
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
        let art = artifacts.get(&ArtName::from_str("SPC-bar").unwrap()).unwrap();
        assert_eq!(art.ty, ArtType::SPC);
        assert_eq!(art.path, path);
        assert_eq!(art.text, "bar");
        assert_eq!(art.refs, ["hello", "ref"]);

        // #TST-artifact-partof-1: test loading of partof
        let expected = ["REQ-Foo", "REQ-Bar-1", "REQ-Bar-2"]
            .iter().map(|n| ArtName::from_str(n).unwrap()).collect();
        assert_eq!(art.partof, expected);
        let expected = Loc::fake();
        assert_eq!(art.loc.as_ref().unwrap(), &expected);
        assert_eq!(art.completed, -1.0);
        assert_eq!(art.tested, -1.0);

        // see TST-settings-load
        let set = &settings.iter().next().unwrap().1;
        assert_eq!(set.paths, VecDeque::from_iter(vec![PathBuf::from("{cwd}/data/empty")]));

    }

    // must be loaded afterwards, uses already existing artifacts
    assert!(load_toml(&path, TOML_OVERLAP, &mut artifacts, &mut settings, &mut variables).is_err());

    let num = load_toml(&path, TOML_RSK2, &mut artifacts, &mut settings, &mut variables).unwrap();
    assert_eq!(num, 3);
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-baz").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("RSK-foo-2").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("TST-foo-2").unwrap()));
}

/// do the raw load with variable resolultion
pub fn load_raw_extra(path: &Path)
                      -> LoadResult<(Artifacts, Settings)> {
    let (mut artifacts, mut settings, loaded_vars, mut repo_map) = try!(load_raw(path));
    let mut variables = try!(vars::resolve_loaded_vars(loaded_vars, &mut repo_map));
    try!(vars::fill_text_fields(&mut artifacts, &settings, &mut variables, &mut repo_map));
    try!(vars::fill_text_fields(&mut artifacts, &settings, &mut variables, &mut repo_map));
    Ok((artifacts, settings))
}

#[test]
fn test_load_raw() {
    init_logger_test();
    info!("running test_load_raw");
    // partof: #TST-load-dir-invalid
    // see: invalid.1: load with invalid attribute
    assert!(load_raw_extra(TINVALID_DIR.join(&PathBuf::from("attr")).as_path()).is_err());
    // see: invalid.2: load two files that have the same key
    assert!(load_raw_extra(TINVALID_DIR.join(&PathBuf::from("same_names")).as_path()).is_err());

    info!("loading only valid now");
    // The TSIMPL_DIR has several tests set up in it, including valid
    // "back references" to make sure that directories don't load multiple
    // times, valid loc, etc.
    // partof: #TST-load-loop, #TST-load-dir-valid
    let (artifacts, settings) = load_raw_extra(TSIMPLE_DIR.as_path()).unwrap();
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-purpose").unwrap()));

    // load all artifacts that should exist
    let req_purpose = artifacts.get(&ArtName::from_str("REQ-purpose").unwrap()).unwrap();
    // see valid.1
    let req_lvl1 = artifacts.get(&ArtName::from_str("REQ-lvl-1").unwrap()).unwrap();
    let spc_lvl1 = artifacts.get(&ArtName::from_str("SPC-lvl-1").unwrap()).unwrap();
    let spc_loc  = artifacts.get(&ArtName::from_str("SPC-loc").unwrap()).unwrap();

    let req_lvl2 = artifacts.get(&ArtName::from_str("REQ-lvl-2").unwrap()).unwrap();
    let spc_lvl2 = artifacts.get(&ArtName::from_str("SPC-lvl-2").unwrap()).unwrap();
    let tst_lvl2 = artifacts.get(&ArtName::from_str("TST-lvl-2").unwrap()).unwrap();

    // deep loading
    // #TST-core-load-path
    // #TST-core-load-dir-unit-4
    assert!(!artifacts.contains_key(&ArtName::from_str("REQ-unreachable").unwrap()));

    let req_deep = artifacts.get(&ArtName::from_str("REQ-deep").unwrap()).unwrap();
    let scp_deep = artifacts.get(&ArtName::from_str("SPC-deep").unwrap()).unwrap();

    let simple_dir_str = TSIMPLE_DIR.as_path().to_str().unwrap().to_string();
    let extra_dir = TSIMPLE_DIR.join(PathBuf::from("extra"));
    let lvl1_dir = TSIMPLE_DIR.join(PathBuf::from("lvl_1"));
    let lvl1_dir_str = lvl1_dir.as_path().to_str().unwrap().to_string();

    // #TST-core-load-dir-unit-5
    assert_eq!(req_purpose.refs, [extra_dir.join(PathBuf::from("README.md")).to_str().unwrap()]);
    assert_eq!(spc_lvl1.text, "level one does FOO");
}
