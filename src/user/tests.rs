//! tests for user module

use toml::Value;

use dev_prefix::*;
use types::*;

use test_data;
use user;
use user::save::ProjectText;
use user::link;
use user::settings;
use user::artifact;

#[test]
fn test_toml_assumptions() {
    // This test is to prove that is impossible to write artifact names
    // such as ART-foo.1
    //
    // Since this is made impossible by the toml format itself, we can assume that such artifacts
    // don't exist when we create them automatically by parsing the text.
    let toml = "\
    [test]
    key = 'value'

    [test.1]
    key = 'value.1'
    ";

    let tbl = test_data::parse_text(toml);
    let test = test_data::get_table(&tbl, "test");
    let value = match test.get("key").unwrap() {
        &Value::String(ref s) => s,
        _ => unreachable!(),
    };
    assert_eq!(value, "value");
    let test_1 = test_data::get_table(&test, "1");
    let value = match test_1.get("key").unwrap() {
        &Value::String(ref s) => s,
        _ => unreachable!(),
    };
    assert_eq!(value, "value.1");
}

#[test]
// partof: #TST-load-simple
fn test_load_repo() {
    // init_logger_test();
    info!("running test_load_repo");
    assert!(user::load_repo(test_data::TINVALID_DIR.join(&PathBuf::from("attr")).as_path())
                .is_err());
    assert!(user::load_repo(test_data::TINVALID_DIR.join(&PathBuf::from("same_names")).as_path())
                .is_err());

    let simple = &test_data::TSIMPLE_DIR;
    let design = simple.join("design");

    let p = user::load_repo(simple.as_path()).unwrap();
    assert_eq!(p.origin, simple.as_path());
    assert!(p.files.contains(&design.join("config.toml")),
            "config.toml does not exist in: {:?}",
            p.files);
    assert!(p.files.contains(&design.join("deep/reqs/deep.toml")));
    assert!(p.files.contains(&design.join("lvl_1/req.toml")));

    assert_eq!(p.origin, simple.as_path());
    let artifacts = p.artifacts;
    assert!(artifacts.contains_key(&Name::from_str("REQ-purpose").unwrap()));

    artifacts.get(&Name::from_str("REQ-purpose").unwrap()).unwrap();

    // load all artifacts that should exist
    artifacts.get(&Name::from_str("REQ-lvl-1").unwrap()).unwrap();
    let spc_lvl1 = artifacts.get(&Name::from_str("SPC-lvl-1").unwrap()).unwrap();
    artifacts.get(&Name::from_str("SPC-loc-dne").unwrap()).unwrap();
    let spc_loc = artifacts.get(&Name::from_str("SPC-loc").unwrap()).unwrap();

    artifacts.get(&Name::from_str("REQ-lvl-2").unwrap()).unwrap();
    artifacts.get(&Name::from_str("SPC-lvl-2").unwrap()).unwrap();
    artifacts.get(&Name::from_str("TST-lvl-2").unwrap()).unwrap();
    assert!(!artifacts.contains_key(&Name::from_str("REQ-unreachable").unwrap()));
    assert!(!artifacts.contains_key(&Name::from_str("SPC-exclude").unwrap()));

    let src_dir = simple.join(PathBuf::from("src"));
    let lvl1_dir = simple.join(PathBuf::from("lvl_1"));
    lvl1_dir.as_path().to_str().unwrap(); // make sure it converts

    // settings
    assert_eq!(p.settings.artifact_paths,
               HashSet::from_iter(vec![design.to_path_buf()]));
    assert_eq!(p.settings.code_paths,
               HashSet::from_iter(vec![src_dir.to_path_buf()]));
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

    assert!(p.dne_locs.contains_key(&Name::from_str("SPC-dne").unwrap()));
    assert!(p.dne_locs.contains_key(&Name::from_str("TST-dne").unwrap()));

    // TODO: more validation
    // TODO: need to check that completeness makes sense: TST-core-load-loc-resolve
}

fn remove_parents(project: &mut Project) {
    let names: Vec<_> = project.artifacts
        .keys()
        .cloned()
        .collect();
    for n in &names {
        if project.artifacts
               .get(n)
               .unwrap()
               .path == PARENT_PATH.as_path() {
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
    //init_logger_test();
    let simple = &test_data::TSIMPLE_DIR;

    let p = user::load_repo(simple.as_path()).unwrap();
    let original_p = p.clone();

    // should be able to process twice without issue
    // (with parents removed)
    {
        let mut new_p = p.clone();
        remove_parents(&mut new_p);
        user::process_project(&mut new_p).unwrap();
        p.equal(&new_p).expect("no-change");
        p.equal(&original_p).expect("original")
    }
    // location should be able to be re-processed
    {
        let mut new_p = p.clone();
        remove_parents(&mut new_p);
        remove_loc(&mut new_p);
        user::process_project(&mut new_p).unwrap();
        p.equal(&new_p).unwrap();
        p.equal(&original_p).expect("original")
    }

    // should be able to convert artifacts to data and
    // back and then process
    {
        let data_artifacts: Vec<_> = p.artifacts
            .iter()
            .map(|(n, a)| a.to_data(&p.origin, n))
            .collect();
        let new_artifacts =
            HashMap::from_iter(data_artifacts.iter().map(|d| {
                                                             Artifact::from_data(&p.origin, d)
                                                                 .unwrap()
                                                         }));

        let mut new_p = Project { artifacts: new_artifacts, ..p.clone() };

        remove_parents(&mut new_p);
        remove_loc(&mut new_p);
        user::process_project(&mut new_p).unwrap();

        p.equal(&new_p).unwrap();
        p.equal(&original_p).expect("original")
    }
}

#[test]
fn test_basic_link() {
    let mut artifacts = test_data::load_toml_simple(test_data::TOML_RST);

    // note: SPC-bar is done via attribute
    let path = PathBuf::from("hi/there");
    for sname in &["SPC-foo", "TST-foo"] {
        let art = artifacts.get_mut(&NameRc::from_str(sname).unwrap()).unwrap();
        art.done = Done::Code(Loc {
                                  path: path.clone(),
                                  line: 1,
                              });
    }

    link::validate_done(&artifacts).unwrap();
    link::link_named_partofs(&mut artifacts);

    link::create_parents(&mut artifacts);
    let req_name = Arc::new(NameRc::from_str("REQ-1").unwrap().parent().unwrap());
    assert!(artifacts.contains_key(&req_name));
    assert!(artifacts.contains_key(&NameRc::from_str("REQ-parts").unwrap()));
    assert!(artifacts.contains_key(&NameRc::from_str("REQ-parts-p1").unwrap()));
    assert!(artifacts.contains_key(&NameRc::from_str("REQ-parts-p1-a").unwrap()));

    // test linking
    link::link_parents(&mut artifacts);
    link::validate_partof(&artifacts).unwrap();
    assert_eq!(link::link_parts(&mut artifacts), 3);
    assert_eq!(link::set_completed(&mut artifacts), 0);
    assert_eq!(link::set_tested(&mut artifacts), 0);

    let req = artifacts.get(&req_name).unwrap();
    let req_parts = artifacts.get(&NameRc::from_str("REQ-parts").unwrap()).unwrap();
    let req_parts_p1 = artifacts.get(&NameRc::from_str("REQ-parts-p1").unwrap()).unwrap();
    let req_parts_p1_a = artifacts.get(&NameRc::from_str("REQ-parts-p1-a").unwrap()).unwrap();
    let spc_foo = artifacts.get(&NameRc::from_str("SPC-foo").unwrap()).unwrap();
    let req_foo = artifacts.get(&NameRc::from_str("REQ-foo").unwrap()).unwrap();
    let tst_foo = artifacts.get(&NameRc::from_str("TST-foo").unwrap()).unwrap();

    // test parts
    assert_eq!(req.partof, HashSet::new());
    assert_eq!(req.parts,
               HashSet::from_iter(["REQ-parts", "REQ-foo"].iter().map(|n| {
                                                                          NameRc::from_str(n)
                                                                              .unwrap()
                                                                      })));

    assert_eq!(req_parts.partof, Names::from_iter(vec![req_name.clone()]));
    assert_eq!(req_parts.parts,
               HashSet::from_iter(["REQ-parts-p1", "REQ-parts-p2"]
                   .iter()
                   .map(|n| NameRc::from_str(n).unwrap())));

    assert_eq!(req_foo.parts,
               HashSet::from_iter(["SPC-foo", "SPC-bar"].iter().map(|n| {
                                                                        NameRc::from_str(n).unwrap()
                                                                    })));
    assert_eq!(spc_foo.partof,
               HashSet::from_iter(["REQ-foo", "SPC"].iter().map(|n| NameRc::from_str(n).unwrap())));

    assert_eq!(req_parts_p1_a.partof,
               HashSet::from_iter(["REQ-parts-p1"].iter().map(|n| NameRc::from_str(n).unwrap())));
    assert_eq!(req_parts_p1_a.parts, HashSet::new());

    // test completed %
    assert_eq!(spc_foo.completed, 1.);
    assert_eq!(req_foo.completed, 1.);
    assert_eq!(req_parts.completed, 0.);
    assert_eq!(req_parts_p1.completed, 0.);
    assert_eq!(req_parts_p1_a.completed, 0.);

    // test tested %
    assert_eq!(tst_foo.tested, 1.);
    assert_eq!(spc_foo.tested, 1.);
    assert_eq!(req_foo.tested, 0.5);
    assert_eq!(req_parts.tested, 0.);
    assert_eq!(req_parts_p1.tested, 0.);
    assert_eq!(req_parts_p1_a.tested, 0.);
}

#[test]
/// extensive testing to make sure that link, completed and tested
/// all work as expected
fn test_link_completed_tested() {
    let mut artifacts = test_data::load_toml_simple(&test_data::TOML_LINK);

    let path = PathBuf::from("hi/there");
    for sname in &["SPC-core-bob-1",
                   "TST-core-bob-1-a",
                   "TST-core-bob-1-b-2",
                   "SPC-core-bob-2-b",
                   "TST-core-bob-2-a"] {
        let art = artifacts.get_mut(&NameRc::from_str(sname).unwrap()).unwrap();
        art.done = Done::Code(Loc {
                                  path: path.clone(),
                                  line: 1,
                              });
    }

    link::link_named_partofs(&mut artifacts);
    link::create_parents(&mut artifacts);
    link::link_parents(&mut artifacts);
    link::validate_partof(&artifacts).unwrap();

    // just checking that this artifact is good throughout the process
    assert_eq!(artifacts.get(&NameRc::from_str("SPC-core-bob").unwrap()).unwrap().partof,
               HashSet::from_iter(["REQ-core-bob", "SPC-core"].iter().map(|n| {
                                                                              NameRc::from_str(n)
                                                                                  .unwrap()
                                                                          })));

    assert_eq!(link::link_parts(&mut artifacts), 0);
    assert_eq!(link::set_completed(&mut artifacts), 0);
    assert_eq!(link::set_tested(&mut artifacts), 0);

    let req_name = Arc::new(NameRc::from_str("REQ-1").unwrap().parent().unwrap());
    let req = artifacts.get(&req_name).unwrap();
    artifacts.get(&NameRc::from_str("REQ-core").unwrap()).unwrap();
    let req_bob = artifacts.get(&NameRc::from_str("REQ-core-bob").unwrap()).unwrap();
    artifacts.get(&NameRc::from_str("SPC-core-bob").unwrap()).unwrap();
    let spc_bob = artifacts.get(&NameRc::from_str("SPC-core-bob").unwrap()).unwrap();

    // bob 1
    let spc_bob_1 = artifacts.get(&NameRc::from_str("SPC-core-bob-1").unwrap()).unwrap();
    let tst_bob_1 = artifacts.get(&NameRc::from_str("TST-core-bob-1").unwrap()).unwrap();
    let tst_bob_1_a = artifacts.get(&NameRc::from_str("TST-core-bob-1-a").unwrap()).unwrap();
    let tst_bob_1_b = artifacts.get(&NameRc::from_str("TST-core-bob-1-b").unwrap()).unwrap();
    artifacts.get(&NameRc::from_str("TST-core-bob-1-b-1").unwrap()).unwrap();
    let tst_bob_1_b_2 = artifacts.get(&NameRc::from_str("TST-core-bob-1-b-2").unwrap()).unwrap();

    // bob 2
    let spc_bob_2 = artifacts.get(&NameRc::from_str("SPC-core-bob-2").unwrap()).unwrap();
    artifacts.get(&NameRc::from_str("SPC-core-bob-2-a").unwrap()).unwrap();
    artifacts.get(&Name::from_str("SPC-core-bob-2-b").unwrap()).unwrap();

    assert_eq!(tst_bob_1_b_2.tested, 1.);

    // jane and joe
    artifacts.get(&NameRc::from_str("REQ-core-joe").unwrap()).unwrap();
    artifacts.get(&NameRc::from_str("REQ-core-jane").unwrap()).unwrap();

    // assert parts make some sense
    // SPC-core-bob automatically has REQ-core-bob
    // SPC-core-bob automatically has SPC-core
    assert_eq!(req.parts,
               HashSet::from_iter(["REQ-core"].iter().map(|n| NameRc::from_str(n).unwrap())));
    assert_eq!(spc_bob.partof,
               HashSet::from_iter(["SPC-core", "REQ-core-bob"].iter().map(|n| {
                                                                              NameRc::from_str(n)
                                                                                  .unwrap()
                                                                          })));
    assert_eq!(req_bob.parts,
               HashSet::from_iter(["SPC-core-bob"].iter().map(|n| NameRc::from_str(n).unwrap())));
    assert_eq!(spc_bob_1.parts,
               HashSet::from_iter(["TST-core-bob-1"].iter().map(|n| NameRc::from_str(n).unwrap())));

    // assert completed
    let bob_complete = (1. + (1. + 0.) / 2.) / 2.;
    assert_eq!(spc_bob_1.completed, 1.);
    assert_eq!(spc_bob.completed, bob_complete);
    assert_eq!(req_bob.completed, bob_complete);
    assert_eq!(req.completed, (bob_complete + 0. + 0.) / 3.0);

    // assert tested
    assert_eq!(tst_bob_1_a.tested, 1.);
    assert_eq!(tst_bob_1_b_2.tested, 1.);
    assert_eq!(tst_bob_1_b.tested, 0.5);
    let bob_1_tested = (1. + 0.5) / 2.;
    assert_eq!(tst_bob_1.tested, bob_1_tested);
    assert_eq!(spc_bob_1.tested, bob_1_tested);
    assert_eq!(spc_bob_2.tested, 0.5);
    let bob_tested = (0.5 + bob_1_tested) / 2.;
    assert_eq!(spc_bob.tested, bob_tested);
    assert_eq!(req_bob.tested, bob_tested);
    assert_eq!(req.tested, (bob_tested + 0. + 0.) / 3.);
}

#[test]
/// load a project as text and then convert
/// #TST-save
fn test_save_idempotent() {
    //init_logger_test();
    // load tsimple and process
    let simple = &test_data::TSIMPLE_DIR;
    let mut original_text = ProjectText::default();
    let mut loaded_dirs = HashSet::new();
    let settings = settings::load_settings(simple.as_path()).unwrap();
    artifact::load_text(&mut original_text, &simple.join("design"), &mut loaded_dirs).unwrap();
    let original = user::process_project_text(settings.clone(), &original_text).unwrap();

    // serialize tsimple like it would be saved
    // and convert back
    let result_text = ProjectText::from_project(&original).unwrap();
    let result = user::process_project_text(settings.clone(), &result_text).unwrap();

    // make assertions
    original.equal(&result).unwrap();
    assert_ne!(original_text, result_text);

    // make sure that saving twice does nothing
    let result_text2 = ProjectText::from_project(&result).unwrap();
    let result2 = user::process_project_text(settings.clone(), &result_text2).unwrap();

    result.equal(&result2).unwrap();
    assert_eq!(result_text, result_text2);
}

#[test]
fn test_exclude() {
    let exclude = &test_data::TEXCLUDE_DIR;
    let p = user::load_repo(exclude.as_path()).unwrap();

    assert_eq!(p.artifacts
                   .get(&NameRc::from_str("SPC-implemented").unwrap())
                   .unwrap()
                   .completed,
               1.0);
    assert_eq!(p.artifacts
                   .get(&NameRc::from_str("SPC-file").unwrap())
                   .unwrap()
                   .completed,
               1.0);
    assert_eq!(p.artifacts
                   .get(&NameRc::from_str("SPC-not-implemented").unwrap())
                   .unwrap()
                   .completed,
               0.0);
    assert!(!p.artifacts.contains_key(&NameRc::from_str("SPC-excluded").unwrap()));
}
