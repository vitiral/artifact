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
use super::super::link::*;


#[test]
fn test_basic_link() {
    let mut artifacts = Artifacts::new();
    let mut settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut variables: Vec<(PathBuf, Variables)> = Vec::new();
    let path = PathBuf::from("hi/there");
    let req_name = &ArtName::from_str("REQ-1").unwrap().parent().unwrap();

    // get te artifacts
    let num = load_toml(&path, TOML_RSK, &mut artifacts, &mut settings, &mut variables).unwrap();

    // test create parents
    create_parents(&mut artifacts);
    assert!(artifacts.contains_key(&req_name));
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-parts").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-parts-p1").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-parts-p1-a").unwrap()));

    // test linking
    link_parents(&mut artifacts);
    validate_partof(&artifacts).unwrap();
    assert_eq!(link_parts(&mut artifacts), 3);
    assert_eq!(set_completed(&mut artifacts), 0);
    assert_eq!(set_tested(&mut artifacts), 0);

    let req = artifacts.get(&req_name).unwrap();
    let req_parts = artifacts.get(&ArtName::from_str("REQ-parts").unwrap()).unwrap();
    let req_parts_p1 = artifacts.get(&ArtName::from_str("REQ-parts-p1").unwrap()).unwrap();
    let req_parts_p1_a = artifacts.get(&ArtName::from_str("REQ-parts-p1-a").unwrap()).unwrap();
    let spc_foo = artifacts.get(&ArtName::from_str("SPC-foo").unwrap()).unwrap();
    let req_foo = artifacts.get(&ArtName::from_str("REQ-foo").unwrap()).unwrap();
    let tst_foo = artifacts.get(&ArtName::from_str("TST-foo").unwrap()).unwrap();

    // test parts
    assert_eq!(req.partof, HashSet::new());
    assert_eq!(req.parts, HashSet::from_iter(
        ["REQ-parts", "REQ-foo"].iter().map(|n| ArtName::from_str(n).unwrap())));

    assert_eq!(req_parts.partof, HashSet::from_iter(vec![req_name.clone()]));
    assert_eq!(req_parts.parts, HashSet::from_iter(
        ["REQ-parts-p1", "REQ-parts-p2"].iter().map(|n| ArtName::from_str(n).unwrap())));

    assert_eq!(req_parts_p1_a.partof, HashSet::from_iter(
        ["REQ-parts-p1"].iter().map(|n| ArtName::from_str(n).unwrap())));
    assert_eq!(req_parts_p1_a.parts, HashSet::new());

    // test completed %
    assert_eq!(spc_foo.completed, 100.0);
    assert_eq!(req_foo.completed, 100.0);
    assert_eq!(req_parts.completed, 0.0);
    assert_eq!(req_parts_p1.completed, 0.0);
    assert_eq!(req_parts_p1_a.completed, 0.0);

    // test tested %
    assert_eq!(tst_foo.tested, 100.0);
    assert_eq!(spc_foo.tested, 0.0);
    assert_eq!(req_foo.tested, 0.0);
    assert_eq!(req_parts.tested, 0.0);
    assert_eq!(req_parts_p1.tested, 0.0);
    assert_eq!(req_parts_p1_a.tested, 0.0);
}

#[test]
fn test_link_completed_tested() {
    let mut artifacts = Artifacts::new();
    let mut settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut variables: Vec<(PathBuf, Variables)> = Vec::new();
    let path = PathBuf::from("hi/there");
    let req_name = &ArtName::from_str("REQ-1").unwrap().parent().unwrap();

    let num = load_toml(&path, TOML_LINK, &mut artifacts, &mut settings, &mut variables).unwrap();

    // just checking that this artifact is good throughout the process
    assert_eq!(artifacts.get(&ArtName::from_str("SPC-core-bob").unwrap()).unwrap().partof,
               HashSet::from_iter(
        ["REQ-core-bob"].iter().map(|n| ArtName::from_str(n).unwrap())));

    create_parents(&mut artifacts);
    link_parents(&mut artifacts);
    validate_partof(&artifacts).unwrap();

    // just checking that this artifact is good throughout the process
    assert_eq!(artifacts.get(&ArtName::from_str("SPC-core-bob").unwrap()).unwrap().partof,
               HashSet::from_iter(
        ["REQ-core-bob", "SPC-core"].iter().map(|n| ArtName::from_str(n).unwrap())));

    assert_eq!(link_parts(&mut artifacts), 0);
    assert_eq!(set_completed(&mut artifacts), 0);
    assert_eq!(set_tested(&mut artifacts), 0);

    let req            = artifacts.get(&req_name).unwrap();
    let req_core       = artifacts.get(&ArtName::from_str("REQ-core").unwrap()).unwrap();
    let req_bob        = artifacts.get(&ArtName::from_str("REQ-core-bob").unwrap()).unwrap();
    let spc_bob        = artifacts.get(&ArtName::from_str("SPC-core-bob").unwrap()).unwrap();
    let spc_bob        = artifacts.get(&ArtName::from_str("SPC-core-bob").unwrap()).unwrap();

    // bob 1
    let spc_bob_1      = artifacts.get(&ArtName::from_str("SPC-core-bob-1").unwrap()).unwrap();
    let tst_bob_1_a    = artifacts.get(&ArtName::from_str("TST-core-bob-1-a").unwrap()).unwrap();
    let tst_bob_1_b    = artifacts.get(&ArtName::from_str("TST-core-bob-1-b").unwrap()).unwrap();

    // bob 2
    let spc_bob_2      = artifacts.get(&ArtName::from_str("SPC-core-bob-2").unwrap()).unwrap();
    let spc_bob_2_a    = artifacts.get(&ArtName::from_str("SPC-core-bob-2-a").unwrap()).unwrap();
    let spc_bob_2_b    = artifacts.get(&ArtName::from_str("SPC-core-bob-2-b").unwrap()).unwrap();

    // jane and joe
    let spc_bob_2_a    = artifacts.get(&ArtName::from_str("REQ-core-joe").unwrap()).unwrap();
    let spc_bob_2_b    = artifacts.get(&ArtName::from_str("REQ-core-jane").unwrap()).unwrap();

    // assert parts make some sense
    assert_eq!(req.parts, HashSet::from_iter(
        ["REQ-core"].iter().map(|n| ArtName::from_str(n).unwrap())));
    assert_eq!(spc_bob.partof, HashSet::from_iter(
        ["SPC-core", "REQ-core-bob"].iter().map(|n| ArtName::from_str(n).unwrap())));
    assert_eq!(req_bob.parts, HashSet::from_iter(
        ["SPC-core-bob"].iter().map(|n| ArtName::from_str(n).unwrap())));

    // assert completed
    assert_eq!(spc_bob_1.completed,     100.0);
    assert_eq!(spc_bob.completed,       50.0);
    assert_eq!(req_bob.completed,       50.0);
    assert_eq!(req.completed, (0.5 + 0. + 0.) * 100.0 / 3.0);

}
