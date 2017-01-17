#![allow(dead_code, unused_imports, unused_variables)]

use dev_prefix::*;

use toml::{Parser, Value, Table};
use super::*;  // data directory constants
use super::super::*;

#[test]
// partof: #TST-load-simple, #TST-settings-resolve
fn test_load_path() {
    // init_logger_test();
    info!("running test_load_path");
    // see: TST-load-dir-invalid
    assert!(load_path(TINVALID_DIR.join(&PathBuf::from("attr")).as_path()).is_err());
    assert!(load_path(TINVALID_DIR.join(&PathBuf::from("same_names")).as_path()).is_err());

    // TODO: make assertions regarding files
    let p = load_path(TSIMPLE_DIR.as_path()).unwrap();
    let (artifacts, dne_locs) = (p.artifacts, p.dne_locs);
    assert!(artifacts.contains_key(&ArtName::from_str("REQ-purpose").unwrap()));

    let req_purpose = artifacts.get(&ArtName::from_str("REQ-purpose").unwrap()).unwrap();

    // load all artifacts that should exist
    let req_lvl1 = artifacts.get(&ArtName::from_str("REQ-lvl-1").unwrap()).unwrap();
    let spc_lvl1 = artifacts.get(&ArtName::from_str("SPC-lvl-1").unwrap()).unwrap();
    let spc_dne = artifacts.get(&ArtName::from_str("SPC-loc-dne").unwrap()).unwrap();
    let spc_loc = artifacts.get(&ArtName::from_str("SPC-loc").unwrap()).unwrap();

    let req_lvl2 = artifacts.get(&ArtName::from_str("REQ-lvl-2").unwrap()).unwrap();
    let spc_lvl2 = artifacts.get(&ArtName::from_str("SPC-lvl-2").unwrap()).unwrap();
    let tst_lvl2 = artifacts.get(&ArtName::from_str("TST-lvl-2").unwrap()).unwrap();

    // deep loading
    assert!(!artifacts.contains_key(&ArtName::from_str("REQ-unreachable").unwrap()));

    let req_deep = artifacts.get(&ArtName::from_str("REQ-deep").unwrap()).unwrap();
    let scp_deep = artifacts.get(&ArtName::from_str("SPC-deep").unwrap()).unwrap();

    let simple_dir_str = TSIMPLE_DIR.as_path().to_str().unwrap().to_string();
    let extra_dir = TSIMPLE_DIR.join(PathBuf::from("extra"));
    let src_dir = TSIMPLE_DIR.join(PathBuf::from("src"));
    let lvl1_dir = TSIMPLE_DIR.join(PathBuf::from("lvl_1"));
    let lvl1_dir_str = lvl1_dir.as_path().to_str().unwrap().to_string();

    assert_eq!(spc_lvl1.text.value, "level one does FOO");
    assert_eq!(spc_lvl1.loc.as_ref().unwrap().path,
               src_dir.join(PathBuf::from("lvl_1.rs")));

    debug!("checking loc");
    assert_eq!(spc_loc.loc.iter().next().unwrap().line_col, (4, 4));
    assert_eq!(spc_lvl1.loc.iter().next().unwrap().line_col, (3, 3));

    assert!(dne_locs.contains_key(&ArtName::from_str("SPC-dne").unwrap()));
    assert!(dne_locs.contains_key(&ArtName::from_str("TST-dne").unwrap()));

    // TODO: more validation
    // TODO: need to check that completeness makes sense: TST-core-load-loc-resolve
}
