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
use super::super::*;

#[test]
fn test_load_path() {
    // env_logger::init();
    info!("running test_load_path");
    assert!(load_path(TINVALID_DIR.join(&PathBuf::from("attr")).as_path()).is_err());
    assert!(load_path(TINVALID_DIR.join(&PathBuf::from("same_names")).as_path()).is_err());

    let (artifacts, settings) = load_path(TSIMPLE_DIR.as_path()).unwrap();
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-purpose").unwrap()));

    let req_purpose = artifacts.get(&ArtName::from_str("REQ-purpose").unwrap()).unwrap();

    // load all artifacts that should exist
    // LOC-core-load-dir-unit-1
    let req_lvl1 = artifacts.get(&ArtName::from_str("REQ-lvl-1").unwrap()).unwrap();
    let spc_lvl1 = artifacts.get(&ArtName::from_str("SPC-lvl-1").unwrap()).unwrap();
    let spc_loc  = artifacts.get(&ArtName::from_str("SPC-loc").unwrap()).unwrap();

    let req_lvl2 = artifacts.get(&ArtName::from_str("REQ-lvl-2").unwrap()).unwrap();
    let spc_lvl2 = artifacts.get(&ArtName::from_str("SPC-lvl-2").unwrap()).unwrap();
    let tst_lvl2 = artifacts.get(&ArtName::from_str("TST-lvl-2").unwrap()).unwrap();

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
    println!("here");
    assert_eq!(req_purpose.refs, [extra_dir.join(PathBuf::from("README.md")).to_str().unwrap()]);
    println!("here");
    assert_eq!(spc_lvl1.text, "level one does FOO");
    println!("here");
    assert_eq!(spc_lvl1.loc.as_ref().unwrap().path, lvl1_dir.join(PathBuf::from("lvl_1.rs")));

    // LOC-core-resolve-loc-unit-1<test that loc is loaded correctly>
    println!("here");
    println!("spc_loc: {:?}", spc_loc);
    assert_eq!(spc_loc.loc.iter().next().unwrap().line_col.unwrap(), (4, 4));
    println!("here");
    assert_eq!(spc_lvl1.loc.iter().next().unwrap().line_col.unwrap(), (3, 3));
    println!("here");

    // TODO: more validation
}
