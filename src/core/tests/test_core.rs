#![allow(dead_code, unused_imports, unused_variables)]

use dev_prefix::*;

use toml::{Parser, Value, Table};
use core;
use core::types::*;
use core::tests;
use core::load;
use core::locs;

#[test]
// partof: #TST-load-simple
fn test_load_path() {
    // init_logger_test();
    info!("running test_load_path");
    // see: TST-load-dir-invalid
    assert!(core::load_path(tests::TINVALID_DIR.join(&PathBuf::from("attr")).as_path()).is_err());
    assert!(core::load_path(tests::TINVALID_DIR.join(&PathBuf::from("same_names")).as_path())
        .is_err());

    let simple = &tests::TSIMPLE_DIR;
    let design = simple.join("design");

    let p = core::load_path(simple.as_path()).unwrap();
    assert!(p.files.contains(&design.join("config.toml")),
            "config.toml does not exist in: {:?}",
            p.files);
    assert!(p.files.contains(&design.join("deep/reqs/deep.toml")));
    assert!(p.files.contains(&design.join("lvl_1/req.toml")));


    assert_eq!(p.origin, simple.as_path());
    let artifacts = p.artifacts;
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

    let simple_dir_str = simple.as_path().to_str().unwrap().to_string();
    let extra_dir = simple.join(PathBuf::from("extra"));
    let src_dir = simple.join(PathBuf::from("src"));
    let lvl1_dir = simple.join(PathBuf::from("lvl_1"));
    let lvl1_dir_str = lvl1_dir.as_path().to_str().unwrap().to_string();

    // settings
    assert_eq!(p.settings.artifact_paths,
               HashSet::from_iter(vec![design.to_path_buf()]));
    assert_eq!(p.settings.code_paths,
               VecDeque::from_iter(vec![src_dir.to_path_buf()]));
    assert_eq!(p.dne_locs.len(), 2);

    // locations
    assert_eq!(spc_lvl1.text, "level one does FOO");
    let loc_lvl1 = match spc_lvl1.done {
        Done::Code(ref l) => l.clone(),
        _ => panic!(),
    };
    let loc_loc = match spc_loc.done {
        Done::Code(ref l) => l.clone(),
        _ => panic!(),
    };

    assert_eq!(loc_lvl1.path, src_dir.join(PathBuf::from("lvl_1.rs")));

    debug!("checking loc");
    assert_eq!(loc_lvl1.line, 3);
    assert_eq!(loc_loc.line, 4);

    assert!(p.dne_locs.contains_key(&ArtName::from_str("SPC-dne").unwrap()));
    assert!(p.dne_locs.contains_key(&ArtName::from_str("TST-dne").unwrap()));

    // TODO: more validation
    // TODO: need to check that completeness makes sense: TST-core-load-loc-resolve
}

fn remove_parents(project: &mut Project) {
    let names: Vec<_> = project.artifacts.keys().cloned().collect();
    for n in &names {
        if project.artifacts.get(n).unwrap().path == PARENT_PATH.as_path() {
            project.artifacts.remove(n).unwrap();
        }
    }
}

fn remove_loc(project: &mut Project) {
    for (_, a) in &mut project.artifacts {
        a.done = Done::NotDone;
    }
}

#[test]
/// make sure that serializing/deserializing and then
/// processing results in the same project
fn test_process_project() {
    //core::init_logger_test();
    let simple = &tests::TSIMPLE_DIR;

    let p = core::load_path(simple.as_path()).unwrap();
    let original_p = p.clone();

    // should be able to process twice without issue
    // (with parents removed)
    {
        let mut new_p = p.clone();
        remove_parents(&mut new_p);
        load::process_project(&mut new_p).unwrap();
        p.equal(&new_p).expect("no-change");
        p.equal(&original_p).expect("original")
    }
    // location should be able to be re-processed
    {
        let mut new_p = p.clone();
        remove_parents(&mut new_p);
        remove_loc(&mut new_p);
        load::process_project(&mut new_p).unwrap();
        p.equal(&new_p).unwrap();
        p.equal(&original_p).expect("original")
    }

    // should be able to convert artifacts to data and
    // back and then process
    {
        let data_artifacts: Vec<_> = p.artifacts
            .iter()
            .map(|(n, a)| a.to_data(n))
            .collect();
        let new_artifacts = HashMap::from_iter(data_artifacts.iter()
            .map(|d| Artifact::from_data(d).unwrap()));

        let mut new_p = Project { artifacts: new_artifacts, ..p.clone() };

        remove_parents(&mut new_p);
        remove_loc(&mut new_p);
        load::process_project(&mut new_p).unwrap();

        p.equal(&new_p).unwrap();
        p.equal(&original_p).expect("original")
    }
}
