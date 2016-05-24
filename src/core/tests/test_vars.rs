#![allow(dead_code, unused_imports, unused_variables)]

use std::env;
use std::collections::{HashMap, HashSet, VecDeque};

use super::*;  // data directory constants
use super::super::vars::*;

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
