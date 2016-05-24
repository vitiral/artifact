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
    let num = load_toml(&path, TOML_RSK, &mut artifacts, &mut settings, &mut variables).unwrap();

    let req_name = &ArtName::from_str("REQ-1").unwrap().parent().unwrap();
    // test create parents
    create_parents(&mut artifacts);
    assert!(artifacts.contains_key(&req_name));
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-parts").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-parts-p1").unwrap()));
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-parts-p1-a").unwrap()));

    // test linking
    link_parents(&mut artifacts);
    link_parts(&mut artifacts);
    {
        let req = artifacts.get(&req_name).unwrap();
        let req_parts = artifacts.get(&ArtName::from_str("REQ-parts").unwrap()).unwrap();
        let req_parts_p1 = artifacts.get(&ArtName::from_str("REQ-parts-p1").unwrap()).unwrap();
        let req_parts_p1_a = artifacts.get(&ArtName::from_str("REQ-parts-p1-a").unwrap()).unwrap();

        assert_eq!(req.partof, HashSet::new());
        assert_eq!(req.parts, HashSet::from_iter(
            ["REQ-parts", "REQ-foo", "REQ-bar"].iter().map(|n| ArtName::from_str(n).unwrap())));

        assert_eq!(req_parts.partof, HashSet::from_iter(vec![req_name.clone()]));
        assert_eq!(req_parts.parts, HashSet::from_iter(
            ["REQ-parts-p1", "REQ-parts-p2"].iter().map(|n| ArtName::from_str(n).unwrap())));

        assert_eq!(req_parts_p1_a.partof, HashSet::from_iter(
            ["REQ-parts-p1"].iter().map(|n| ArtName::from_str(n).unwrap())));
        assert_eq!(req_parts_p1_a.parts, HashSet::new());
    }
}
