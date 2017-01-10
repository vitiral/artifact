#![allow(dead_code, unused_imports, unused_variables)]

use dev_prefix::*;
use super::*;  // data directory constants
use core::types::*;
use core::vars::*;
use core::locs::*;
use core::utils;
use super::super::super::init_logger;

#[test]
fn test_find_repo() {
    assert_eq!(utils::find_repo(TSIMPLE_DIR.as_path()).unwrap(),
               TSIMPLE_DIR.as_path());
    assert_eq!(utils::find_repo(TSIMPLE_DIR.join("lvl_1").as_path()).unwrap(),
               TSIMPLE_DIR.as_path());
    assert!(utils::find_repo(env::temp_dir().as_path()).is_none());
}

#[test]
/// #TST-vars-race
fn test_resolve_vars() {
    // we are getting a race condition with variables where sometimes not all
    // variables are resolving. We need to find it and destroy it.
    let mut loaded_vars: Variables = Variables::new();
    let mut variables: Variables = Variables::new();
    let var_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut repo_map: HashMap<PathBuf, PathBuf> = HashMap::new();

    let fpath = TSIMPLE_DIR.join(PathBuf::from("fake.toml"));

    for i in 0..3 { // do it a few times
        loaded_vars.clear();
        variables.clear();
        loaded_vars.insert("foo".to_string(), PathBuf::from("{repo}").join("FOO").to_string_lossy().to_string());
        loaded_vars.insert("bar".to_string(), PathBuf::from("{foo}").join("BAR").to_string_lossy().to_string());
        loaded_vars.insert("bar-2".to_string(), PathBuf::from("{bar}").join("BAR2").to_string_lossy().to_string());

        resolve_default_vars(&loaded_vars, fpath.as_path(), &mut variables,
                             &mut repo_map).unwrap();
        resolve_vars(&mut variables).unwrap();
        let foo = TSIMPLE_DIR.join("FOO");
        let bar = foo.join("BAR");
        let bar2 = bar.join("BAR2");

        assert_eq!(variables.get("foo").unwrap(), foo.to_str().unwrap());
        assert_eq!(variables.get("bar").unwrap(), bar.to_str().unwrap());
        assert_eq!(variables.get("bar-2").unwrap(), bar2.to_str().unwrap());
    }
}


pub const LOC_TEST: &'static str = "\
$SPC-who
   #$SPC-what
 // $SPC-where
  //kjsdlfkjwe $TST-foo-what-where-2-b-3 kljasldkjf
// $TST-dont-care
/// $SPC-core-load-erro: <load file error>
";

#[test]
/// partof: #TST-loc
fn test_resolve_loc_text() {
    let mut locs: HashMap<ArtName, Loc> = HashMap::new();
    let path = PathBuf::from("hi/there");
    let loc_test = LOC_TEST.replace("$", "#");
    assert!(!find_locs_text(&path, &loc_test, &mut locs));
    // change: all locations are found
    assert!(locs.contains_key(&ArtName::from_str("TST-dont-care").unwrap()));

    let spc_who = locs.get(&ArtName::from_str("SPC-who").unwrap()).unwrap();
    let spc_what = locs.get(&ArtName::from_str("SPC-what").unwrap()).unwrap();
    let spc_where = locs.get(&ArtName::from_str("SPC-where").unwrap()).unwrap();
    let tst_long = locs.get(&ArtName::from_str("TST-foo-what-where-2-b-3").unwrap()).unwrap();
    let spc_error = locs.get(&ArtName::from_str("SPC-core-load-erro").unwrap()).unwrap();

    assert_eq!(spc_who.line_col,     (1, 0));
    assert_eq!(spc_what.line_col,    (2, 4));
    assert_eq!(spc_where.line_col,   (3, 4));
    assert_eq!(tst_long.line_col,    (4, 15));
    assert_eq!(spc_error.line_col,   (6, 4));
}
