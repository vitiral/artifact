#![allow(dead_code, unused_imports, unused_variables)]

use dev_prefix::*;
use super::*; // data directory constants
use core::types::*;
use core::locs::*;
use core::utils;
use super::super::super::init_logger;

#[test]
fn test_find_repo() {
    let simple = &TSIMPLE_DIR;
    assert_eq!(utils::find_repo(simple.as_path()).unwrap(),
               simple.as_path());
    assert_eq!(utils::find_repo(simple.join("lvl_1").as_path()).unwrap(),
               simple.as_path());
    assert!(utils::find_repo(env::temp_dir().as_path()).is_none());
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

    assert_eq!(spc_who.line_col, (1, 0));
    assert_eq!(spc_what.line_col, (2, 4));
    assert_eq!(spc_where.line_col, (3, 4));
    assert_eq!(tst_long.line_col, (4, 15));
    assert_eq!(spc_error.line_col, (6, 4));
}
