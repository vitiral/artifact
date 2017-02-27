#![allow(dead_code, unused_imports, unused_variables)]

use rustc_serialize::Decodable;

use super::super::init_logger_test;
use dev_prefix::*;
use core::types::*;
use core::tests;
use core::load;
use core::locs;

use strfmt;
use toml::{Parser, Value, Table, Decoder};

// // Tests

#[test]
fn test_artifact_name() {
    // valid names
    for name in vec!["REQ-foo",
                     "REQ-foo-2",
                     "REQ-foo2",
                     "REQ-foo2",
                     "REQ-foo-bar-2_3",
                     "SPC-foo",
                     "RSK-foo",
                     "TST-foo"] {
        assert!(ArtName::from_str(name).is_ok());
    }
    for name in vec!["REQ-foo*", "REQ-foo\n", "REQ-foo-"] {
        assert!(ArtName::from_str(name).is_err())
    }
    // remove spaces
    assert_eq!(ArtName::from_str("   R E Q    -    f   o  o   ").unwrap().value,
               ["REQ", "FOO"]);
}

#[test]
fn test_get_attr() {
    let tbl_good = load::parse_toml(tests::TOML_GOOD).unwrap();
    let df_str = "".to_string();
    let df_tbl = Table::new();
    let ref df_vec: Vec<String> = Vec::new();

    let test = get_attr!(tbl_good, "REQ-bar", df_tbl, Table).unwrap();
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "bar");
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "bar");

    assert!(get_attr!(&test, "text", false, Boolean).is_none());
    let test = get_attr!(tbl_good, "SPC-foo", Table::new(), Table).unwrap();

    let test = get_attr!(tbl_good, "REQ-foo", Table::new(), Table).unwrap();
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "");
}

#[test]
fn test_settings() {
    let tbl = load::parse_toml(tests::TOML_SETTINGS).unwrap();
    let df_tbl = Table::new();
    let (_, set) = Settings::from_table(&tbl).unwrap();
    assert!(set.artifact_paths ==
            HashSet::from_iter(vec![PathBuf::from("{cwd}/test"), PathBuf::from("{repo}/test")]));
    assert!(set.code_paths ==
            VecDeque::from_iter(vec![PathBuf::from("{cwd}/src"), PathBuf::from("{repo}/src2")]));

    let toml_invalid = r#"
    artifact_paths = ['hi']
    paths = ['invalid']
    "#;
    let tbl = load::parse_toml(toml_invalid).unwrap();

    assert!(Settings::from_table(&tbl).is_err());
}

#[test]
fn test_load_raw_impl() {
    // this is just a sanity-check/playground for how to implement
    // loading using toml decoder
    let text = r#"
    [REQ-one]
    partof = "REQ-1"
    text = '''
    I am text
    '''
    "#;
    let file_table = Parser::new(text).parse().unwrap();
    let mut artifacts: HashMap<String, RawArtifact> = HashMap::new();
    for (name, value) in file_table.iter() {
        let mut decoder = Decoder::new(value.clone());
        let raw = RawArtifact::decode(&mut decoder).unwrap();
        artifacts.insert(name.clone(), raw);
    }
    assert_eq!(artifacts.get("REQ-one").unwrap().text,
               Some("    I am text\n    ".to_string()));
    assert_eq!(artifacts.get("REQ-one").unwrap().partof,
               Some("REQ-1".to_string()));
}

#[test]
fn test_load_toml() {
    let mut p = Project::default();

    let path = PathBuf::from("hi/there");

    // #TST-load-invalid
    assert!(load::load_toml(&path, tests::TOML_BAD, &mut p).is_err());
    assert!(load::load_toml(&path, tests::TOML_BAD_JSON, &mut p).is_err());
    assert!(load::load_toml(&path, tests::TOML_BAD_ATTR1, &mut p).is_err());
    assert!(load::load_toml(&path, tests::TOML_BAD_ATTR2, &mut p).is_err());
    assert!(load::load_toml(&path, tests::TOML_BAD_NAMES1, &mut p).is_err());
    assert!(load::load_toml(&path, tests::TOML_BAD_NAMES2, &mut p).is_err());

    // basic loading unit tests
    let num = load::load_toml(&path, tests::TOML_RST, &mut p).unwrap();

    let locs = HashMap::from_iter(vec![(ArtName::from_str("SPC-foo").unwrap(), Loc::fake()),
                                       (ArtName::from_str("SPC-bar").unwrap(), Loc::fake())]);
    let dne_locs = locs::attach_locs(&mut p.artifacts, locs);
    assert_eq!(num, 8);
    assert_eq!(dne_locs.len(), 0);
    assert!(p.artifacts.contains_key(&ArtName::from_str("REQ-foo").unwrap()));
    assert!(p.artifacts.contains_key(&ArtName::from_str("SPC-foo").unwrap()));
    assert!(p.artifacts.contains_key(&ArtName::from_str("RSK-foo").unwrap()));
    assert!(p.artifacts.contains_key(&ArtName::from_str("TST-foo").unwrap()));
    assert!(p.artifacts.contains_key(&ArtName::from_str("SPC-bar").unwrap()));

    // will be loaded later
    assert!(!p.artifacts.contains_key(&ArtName::from_str("REQ-baz").unwrap()));
    assert!(!p.artifacts.contains_key(&ArtName::from_str("RSK-foo-2").unwrap()));
    assert!(!p.artifacts.contains_key(&ArtName::from_str("TST-foo-2").unwrap()));

    {
        // test to make sure default attrs are correct
        let rsk_foo = ArtName::from_str("RSK-foo").unwrap();
        let art = p.artifacts.get(&rsk_foo).unwrap();
        assert_eq!(rsk_foo.ty, ArtType::RSK);
        assert_eq!(art.path, path);
        assert_eq!(art.text, "");
        let expected: ArtNames = HashSet::new();
        assert_eq!(art.partof, expected);
        assert_eq!(art.loc, None);
        assert_eq!(art.completed, -1.0);
        assert_eq!(art.tested, -1.0);

        // test non-defaults
        let spc_bar = ArtName::from_str("SPC-bar").unwrap();
        let art = p.artifacts.get(&spc_bar).unwrap();
        assert_eq!(spc_bar.ty, ArtType::SPC);
        assert_eq!(art.path, path);
        assert_eq!(art.text, "bar");

        let expected = ["REQ-Foo", "REQ-Bar-1", "REQ-Bar-2"]
            .iter()
            .map(|n| ArtNameRc::from_str(n).unwrap())
            .collect();
        assert_eq!(art.partof, expected);
        let expected = Loc::fake();
        assert_eq!(art.loc.as_ref().unwrap(), &expected);
        assert_eq!(art.completed, -1.0);
        assert_eq!(art.tested, -1.0);
    }

    // must be loaded afterwards, uses already existing artifacts
    assert!(load::load_toml(&path, tests::TOML_OVERLAP, &mut p).is_err());

    let num = load::load_toml(&path, tests::TOML_RST2, &mut p).unwrap();
    assert_eq!(num, 3);
    assert!(p.artifacts.contains_key(&ArtName::from_str("REQ-baz").unwrap()));
    assert!(p.artifacts.contains_key(&ArtName::from_str("RSK-foo-2").unwrap()));
    assert!(p.artifacts.contains_key(&ArtName::from_str("TST-foo-2").unwrap()));
}

/// just get the artifacts and settings
pub fn load_raw_extra(path: &Path) -> Result<(Artifacts, Settings)> {
    let project = load::load_project(path)?;
    assert_eq!(project.origin, path);
    Ok((project.artifacts, project.settings))
}

#[test]
/// #TST-artifact
fn test_load_raw() {
    // init_logger_test();
    info!("running test_load_raw");
    // see: TST-load-invalid
    // - load with invalid attribute
    assert!(load_raw_extra(tests::TINVALID_DIR.join(&PathBuf::from("attr")).as_path()).is_err());
    // - load two files that have the same key
    assert!(load_raw_extra(tests::TINVALID_DIR.join(&PathBuf::from("same_names")).as_path())
        .is_err());

    info!("loading only valid now");
    // The TSIMPL_DIR has several tests set up in it, including valid
    // "back references" to make sure that directories don't load multiple
    // times, valid loc, etc.
    // partof: #TST-load-valid
    let simple = &tests::TSIMPLE_DIR;
    let (artifacts, settings) = load_raw_extra(simple.as_path()).unwrap();
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-purpose").unwrap()));

    // load all artifacts that should exist
    let req_purpose = artifacts.get(&ArtName::from_str("REQ-purpose").unwrap()).unwrap();
    // see valid.1
    let req_lvl1 = artifacts.get(&ArtName::from_str("REQ-lvl-1").unwrap()).unwrap();
    let spc_lvl1 = artifacts.get(&ArtName::from_str("SPC-lvl-1").unwrap()).unwrap();
    let spc_loc = artifacts.get(&ArtName::from_str("SPC-loc").unwrap()).unwrap();

    let req_lvl2 = artifacts.get(&ArtName::from_str("REQ-lvl-2").unwrap()).unwrap();
    let spc_lvl2 = artifacts.get(&ArtName::from_str("SPC-lvl-2").unwrap()).unwrap();
    let tst_lvl2 = artifacts.get(&ArtName::from_str("TST-lvl-2").unwrap()).unwrap();

    // deep loading
    assert!(!artifacts.contains_key(&ArtName::from_str("REQ-unreachable").unwrap()));

    let req_deep = artifacts.get(&ArtName::from_str("REQ-deep").unwrap()).unwrap();
    let scp_deep = artifacts.get(&ArtName::from_str("SPC-deep").unwrap()).unwrap();

    let simple_dir_str = simple.as_path().to_str().unwrap().to_string();
    let extra_dir = simple.join(PathBuf::from("extra"));
    let lvl1_dir = simple.join(PathBuf::from("lvl_1"));
    let lvl1_dir_str = lvl1_dir.as_path().to_str().unwrap().to_string();

    assert_eq!(spc_lvl1.text, "level one does FOO");
}
