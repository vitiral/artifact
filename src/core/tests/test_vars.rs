#![allow(dead_code, unused_imports, unused_variables)]

use std::env;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use super::*;  // data directory constants
use super::super::vars::*;
use super::super::types::*;

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
fn test_resolve_vars() {
    // we are getting a race condition with variables where sometimes not all
    // variables are resolving. We need to find it and destroy it.
    let mut variables: Variables = Variables::new();
    let mut var_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut repo_map: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut repo_names = HashSet::new();

    println!("simple dir: {:?}", TSIMPLE_DIR.as_path());
    let fpath = TSIMPLE_DIR.join(PathBuf::from("fake.rsk"));
    repo_names.insert(String::from(".tst_repo_name"));
    var_paths.insert("foo".to_string(), fpath.clone());
    var_paths.insert("bar".to_string(), fpath.clone());
    var_paths.insert("bar-2".to_string(), fpath.clone());

    for i in 0..3 {
        println!("*** run {}", i);
        variables.insert("foo".to_string(), "{repo}/FOO".to_string());
        variables.insert("bar".to_string(), "{foo}/BAR".to_string());
        variables.insert("bar-2".to_string(), "{bar}/BAR2".to_string());

        resolve_vars(&mut variables, &var_paths, &mut repo_map, &repo_names).unwrap();
        let foo = TSIMPLE_DIR.join("FOO");
        let bar = foo.join("BAR");
        let bar2 = bar.join("BAR2");

        assert_eq!(variables.get("foo").unwrap(), foo.to_str().unwrap());
        assert_eq!(variables.get("bar").unwrap(), bar.to_str().unwrap());
        assert_eq!(variables.get("bar-2").unwrap(), bar2.to_str().unwrap());
    }
}
