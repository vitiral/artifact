use std::ascii::AsciiExt;

use super::*;  // data directory constants
use super::super::types::*;
use super::super::load::*;
use super::super::link::*;


#[test]
fn test_basic_link() {
    let mut artifacts = Artifacts::new();
    let mut settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut variables: Vec<(PathBuf, Variables)> = Vec::new();
    let path = PathBuf::from("hi/there");
    let req_name = Rc::new(ArtNameRc::from_str("REQ-1").unwrap().parent().unwrap());

    // get te artifacts
    let num = load_toml(&path, TOML_RST, &mut artifacts, &mut settings, &mut variables).unwrap();
    for sname in &["REQ-foo", "SPC-foo", "TST-foo", "SPC-bar"] {
        let art = artifacts.get_mut(&ArtNameRc::from_str(sname).unwrap()).unwrap();
        art.loc = Some(Loc{path: path.clone(), line_col: (1, 2)});
    }

    link_named_partofs(&mut artifacts);

    create_parents(&mut artifacts);
    assert!(artifacts.contains_key(&req_name));
    assert!(artifacts.contains_key(&ArtNameRc::from_str("REQ-parts").unwrap()));
    assert!(artifacts.contains_key(&ArtNameRc::from_str("REQ-parts-p1").unwrap()));
    assert!(artifacts.contains_key(&ArtNameRc::from_str("REQ-parts-p1-a").unwrap()));

    // test linking
    link_parents(&mut artifacts);
    validate_partof(&artifacts).unwrap();
    assert_eq!(link_parts(&mut artifacts), 3);
    assert_eq!(set_completed(&mut artifacts), 0);
    assert_eq!(set_tested(&mut artifacts), 0);

    let req = artifacts.get(&req_name).unwrap();
    let req_parts = artifacts.get(&ArtNameRc::from_str("REQ-parts").unwrap()).unwrap();
    let req_parts_p1 = artifacts.get(&ArtNameRc::from_str("REQ-parts-p1").unwrap()).unwrap();
    let req_parts_p1_a = artifacts.get(&ArtNameRc::from_str("REQ-parts-p1-a").unwrap()).unwrap();
    let spc_foo = artifacts.get(&ArtNameRc::from_str("SPC-foo").unwrap()).unwrap();
    let req_foo = artifacts.get(&ArtNameRc::from_str("REQ-foo").unwrap()).unwrap();
    let tst_foo = artifacts.get(&ArtNameRc::from_str("TST-foo").unwrap()).unwrap();

    // test parts
    assert_eq!(req.partof, HashSet::new());
    assert_eq!(req.parts, HashSet::from_iter(
        ["REQ-parts", "REQ-foo"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));

    assert_eq!(req_parts.partof, ArtNames::from_iter(vec![req_name.clone()]));
    assert_eq!(req_parts.parts, HashSet::from_iter(
        ["REQ-parts-p1", "REQ-parts-p2"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));

    // [#TST-core-links-named_partof]
    assert_eq!(req_foo.parts, HashSet::from_iter(
        ["SPC-foo", "SPC-bar"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));
    assert_eq!(spc_foo.partof, HashSet::from_iter(
        ["REQ-foo", "SPC"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));

    assert_eq!(req_parts_p1_a.partof, HashSet::from_iter(
        ["REQ-parts-p1"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));
    assert_eq!(req_parts_p1_a.parts, HashSet::new());

    // test completed %
    // [#TST-core-coverage-percent-done-1]
    assert_eq!(spc_foo.completed, 1.);
    assert_eq!(req_foo.completed, 1.);
    assert_eq!(req_parts.completed, 0.);
    assert_eq!(req_parts_p1.completed, 0.);
    assert_eq!(req_parts_p1_a.completed, 0.);

    // test tested %
    // [#TST-core-coverage-percent-tested-1]
    assert_eq!(tst_foo.tested, 1.);
    assert_eq!(spc_foo.tested, 1.);
    assert_eq!(req_foo.tested, 0.5);
    assert_eq!(req_parts.tested, 0.);
    assert_eq!(req_parts_p1.tested, 0.);
    assert_eq!(req_parts_p1_a.tested, 0.);
}

#[test]
fn test_link_completed_tested() {
    let mut artifacts = Artifacts::new();
    let mut settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut variables: Vec<(PathBuf, Variables)> = Vec::new();
    let path = PathBuf::from("hi/there");
    let req_name = Rc::new(ArtNameRc::from_str("REQ-1").unwrap().parent().unwrap());

    let num = load_toml(&path, TOML_LINK, &mut artifacts, &mut settings, &mut variables).unwrap();
    for sname in &["SPC-core-bob-1", "TST-core-bob-1-a", "TST-core-bob-1-b-2",
                   "SPC-core-bob-2-b", "TST-core-bob-2-a"] {
        let art = artifacts.get_mut(&ArtNameRc::from_str(sname).unwrap()).unwrap();
        art.loc = Some(Loc{path: path.clone(), line_col: (1, 2)});
    }

    link_named_partofs(&mut artifacts);
    create_parents(&mut artifacts);
    link_parents(&mut artifacts);
    validate_partof(&artifacts).unwrap();

    // just checking that this artifact is good throughout the process
    assert_eq!(artifacts.get(&ArtNameRc::from_str("SPC-core-bob").unwrap()).unwrap().partof,
               HashSet::from_iter(
        ["REQ-core-bob", "SPC-core"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));

    assert_eq!(link_parts(&mut artifacts), 0);
    assert_eq!(set_completed(&mut artifacts), 0);
    assert_eq!(set_tested(&mut artifacts), 0);

    let req            = artifacts.get(&req_name).unwrap();
    let req_core       = artifacts.get(&ArtNameRc::from_str("REQ-core").unwrap()).unwrap();
    let req_bob        = artifacts.get(&ArtNameRc::from_str("REQ-core-bob").unwrap()).unwrap();
    let spc_bob        = artifacts.get(&ArtNameRc::from_str("SPC-core-bob").unwrap()).unwrap();
    let spc_bob        = artifacts.get(&ArtNameRc::from_str("SPC-core-bob").unwrap()).unwrap();

    // bob 1
    let spc_bob_1      = artifacts.get(&ArtNameRc::from_str("SPC-core-bob-1").unwrap()).unwrap();
    let tst_bob_1      = artifacts.get(&ArtNameRc::from_str("TST-core-bob-1").unwrap()).unwrap();
    let tst_bob_1_a    = artifacts.get(&ArtNameRc::from_str("TST-core-bob-1-a").unwrap()).unwrap();
    let tst_bob_1_b    = artifacts.get(&ArtNameRc::from_str("TST-core-bob-1-b").unwrap()).unwrap();
    let tst_bob_1_b_1  = artifacts.get(&ArtNameRc::from_str("TST-core-bob-1-b-1").unwrap()).unwrap();
    let tst_bob_1_b_2  = artifacts.get(&ArtNameRc::from_str("TST-core-bob-1-b-2").unwrap()).unwrap();

    // bob 2
    let spc_bob_2      = artifacts.get(&ArtNameRc::from_str("SPC-core-bob-2").unwrap()).unwrap();
    let spc_bob_2_a    = artifacts.get(&ArtNameRc::from_str("SPC-core-bob-2-a").unwrap()).unwrap();
    let spc_bob_2_b    = artifacts.get(&ArtName::from_str("SPC-core-bob-2-b").unwrap()).unwrap();

    assert_eq!(tst_bob_1_b_2.tested,    1.);

    // jane and joe
    let spc_bob_2_a    = artifacts.get(&ArtNameRc::from_str("REQ-core-joe").unwrap()).unwrap();
    let spc_bob_2_b    = artifacts.get(&ArtNameRc::from_str("REQ-core-jane").unwrap()).unwrap();

    // assert parts make some sense
    // #TST-artifact-partof-2: SPC-core-bob automatically has REQ-core-bob
    // #TST-artifact-partof-3: SPC-core-bob automatically has SPC-core
    assert_eq!(req.parts, HashSet::from_iter(
        ["REQ-core"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));
    assert_eq!(spc_bob.partof, HashSet::from_iter(
        ["SPC-core", "REQ-core-bob"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));
    assert_eq!(req_bob.parts, HashSet::from_iter(
        ["SPC-core-bob"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));
    assert_eq!(spc_bob_1.parts, HashSet::from_iter(
        ["TST-core-bob-1"].iter().map(|n| ArtNameRc::from_str(n).unwrap())));

    // assert completed
    // [#TST-core-coverage-percent-done-2]
    let bob_complete = (1. + (1.+0.)/2.) / 2.;
    assert_eq!(spc_bob_1.completed,     1.);
    assert_eq!(spc_bob.completed,       bob_complete);
    assert_eq!(req_bob.completed,       bob_complete);
    assert_eq!(req.completed, (bob_complete + 0. + 0.) / 3.0);

    // assert tested
    // [#TST-core-coverage-percent-tested-2]
    assert_eq!(tst_bob_1_a.tested,      1.);
    assert_eq!(tst_bob_1_b_2.tested,    1.);
    assert_eq!(tst_bob_1_b.tested,      0.5);
    let bob_1_tested = (1. + 0.5) / 2.;
    assert_eq!(tst_bob_1.tested,        bob_1_tested);
    assert_eq!(spc_bob_1.tested,        bob_1_tested);
    assert_eq!(spc_bob_2.tested,        0.5);
    let bob_tested = (0.5 + bob_1_tested) / 2.;
    assert_eq!(spc_bob.tested,          bob_tested);
    assert_eq!(req_bob.tested,          bob_tested);
    assert_eq!(req.tested,              (bob_tested + 0. + 0.) / 3.);
}

#[test]
fn test_invalid_partof() {
    // [#TST-core-links-valid-req]
    let artifacts = load_toml_simple("[REQ-foo]\npartof = 'SPC-bar'\n");
    assert!(validate_partof(&artifacts).is_err());
    let artifacts = load_toml_simple("[REQ-foo]\npartof = 'RSK-bar'\n");
    assert!(validate_partof(&artifacts).is_err());
    let artifacts = load_toml_simple("[REQ-foo]\npartof = 'TST-bar'\n");
    assert!(validate_partof(&artifacts).is_err());

    // [#TST-core-links-valid-rsk]
    let artifacts = load_toml_simple("[RSK-foo]\npartof = 'TST-bar'\n");
    assert!(validate_partof(&artifacts).is_err());
    let artifacts = load_toml_simple("[RSK-foo]\npartof = 'SPC-bar'\n");
    assert!(validate_partof(&artifacts).is_err());

    // [#TST-core-links-valid-spc]
    let artifacts = load_toml_simple("[SPC-foo]\npartof = 'TST-bar'\n");
    assert!(validate_partof(&artifacts).is_err());
    let artifacts = load_toml_simple("[SPC-foo]\npartof = 'RSK-bar'\n");
    assert!(validate_partof(&artifacts).is_err());

    // [#TST-core-links-valid-tst]
    let artifacts = load_toml_simple("[TST-foo]\npartof = 'REQ-bar'\n");
    assert!(validate_partof(&artifacts).is_err());
}
