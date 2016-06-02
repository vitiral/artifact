#![allow(dead_code, unused_imports, unused_variables)]

use std::iter::FromIterator;

use std::env;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;


use super::*;  // data directory constants
use super::super::vars::*;
use super::super::types::*;
use super::super::super::init_logger;

#[test]
fn test_find_repo() {
    let mut repo_names = HashSet::new();
    repo_names.insert(String::from(".tst_repo_name"));
    assert_eq!(find_repo(TSIMPLE_DIR.as_path(), &repo_names).unwrap(),
               TSIMPLE_DIR.as_path());
    assert_eq!(find_repo(TSIMPLE_DIR.join("lvl_1").as_path(), &repo_names).unwrap(),
               TSIMPLE_DIR.as_path());
    assert!(find_repo(env::temp_dir().as_path(), &repo_names).is_none());
}


#[test]
/// TST-core-vars-resolve
fn test_resolve_vars() {
    // we are getting a race condition with variables where sometimes not all
    // variables are resolving. We need to find it and destroy it.
    let mut loaded_vars: Variables = Variables::new();
    let mut variables: Variables = Variables::new();
    let mut var_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut repo_map: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut repo_names = HashSet::new();

    println!("simple dir: {:?}", TSIMPLE_DIR.as_path());
    let fpath = TSIMPLE_DIR.join(PathBuf::from("fake.rsk"));
    repo_names.insert(String::from(".tst_repo_name"));

    for i in 0..3 { // do it a few times
        loaded_vars.clear();
        variables.clear();
        loaded_vars.insert("foo".to_string(), "{repo}/FOO".to_string());
        loaded_vars.insert("bar".to_string(), "{foo}/BAR".to_string());
        loaded_vars.insert("bar-2".to_string(), "{bar}/BAR2".to_string());

        // TST-core-vars-resolve-default
        resolve_default_vars(&loaded_vars, fpath.as_path(), &mut variables,
                             &mut repo_map, &repo_names).unwrap();
        // TST-core-vars-resolve-user
        resolve_vars(&mut variables).unwrap();
        let foo = TSIMPLE_DIR.join("FOO");
        let bar = foo.join("BAR");
        let bar2 = bar.join("BAR2");

        assert_eq!(variables.get("foo").unwrap(), foo.to_str().unwrap());
        assert_eq!(variables.get("bar").unwrap(), bar.to_str().unwrap());
        assert_eq!(variables.get("bar-2").unwrap(), bar2.to_str().unwrap());
    }
}


pub static LOC_TEST: &'static str = "\
SPC-who
   #SPC-what
 // SPC-where
  //kjsdlfkjwe TST-foo-what-where-2-b-3 kljasldkjf
// TST-dont-care
/// SPC-core-load-erro: <load file error>
";

#[test]
fn test_resolve_loc_text() {
    // [TST-core-load-loc-text]
    let mut locs: HashMap<ArtName, (PathBuf, usize, usize)> = HashMap::new();
    let mut looking_for: HashSet<ArtName> = HashSet::from_iter(
        vec!["SPC-who", "SPC-what", "SPC-where", "TST-foo-what-where-2-b-3",
             "SPC-core-load-erro"]
        .iter().map(|n| ArtName::from_str(n).unwrap()));
    let path = PathBuf::from("hi/there");
    resolve_locs_text(LOC_TEST, &path, &mut locs, &looking_for).unwrap();
    assert!(!locs.contains_key(&ArtName::from_str("TST-dont-care").unwrap()));

    let spc_who = locs.get(&ArtName::from_str("SPC-who").unwrap()).unwrap();
    let spc_what = locs.get(&ArtName::from_str("SPC-what").unwrap()).unwrap();
    let spc_where = locs.get(&ArtName::from_str("SPC-where").unwrap()).unwrap();
    let tst_long = locs.get(&ArtName::from_str("TST-foo-what-where-2-b-3").unwrap()).unwrap();
    let spc_error = locs.get(&ArtName::from_str("SPC-core-load-erro").unwrap()).unwrap();

    fn get_linecol(l: &(PathBuf, usize, usize)) -> (usize, usize) {
        (l.1, l.2)
    }
    assert_eq!(get_linecol(spc_who),     (1, 0));
    assert_eq!(get_linecol(spc_what),    (2, 4));
    assert_eq!(get_linecol(spc_where),   (3, 4));
    assert_eq!(get_linecol(tst_long),    (4, 15));
    assert_eq!(get_linecol(spc_error),   (6, 4));
}
