/// test the fmt command

use std::panic;
use dev_prefix::*;
use cmd;
use user;
use test_data;
use utils;

use tempdir;
use fs_extra::dir;

// TODO: consider rewirting this
// #[test]
// fn test_fmt_security() {
//     use std::process;
//     let tmpdir = tempdir::TempDir::new("artifact").unwrap();
//     let writedir = tmpdir.path();
//     // let writedir = Path::new("junk");
//     println!("temppath: {}", writedir.display());
//     dir::copy(&test_data::TINVALID_BOUNDS.as_path(), &writedir, 
//               &dir::CopyOptions::new()).unwrap();


//     // make sure that we can't load invalid stuff
//     let mut w: Vec<u8> = Vec::new();
//     let design = writedir.join("out-bounds").join("repo").join("design");
//     let repo = utils::find_repo(&design).unwrap();
//     let project = user::load_repo(&repo).expect("load");
//     let c = cmd::fmt::Cmd::Write;
//     match cmd::fmt::run_cmd(&mut w, &repo, &project, &c) {
//         Err(e) => {
//             match *e.kind() {
//                 ErrorKind::Security(_) => { /* expected */ }
//                 _ => panic!("unexpected error: {:?}", e.display()),
//             }
//         }
//         Ok(_) => panic!("fmt accidentally suceeded -- may need to reset with git"),
//     }
// }

/// partof: #TST-cmd-fmt
#[test]
fn test_fmt() {
    let tmpdir = tempdir::TempDir::new("artifact").unwrap();
    let writedir = tmpdir.path();
    dir::copy(&test_data::TSIMPLE_DIR.as_path(), &writedir, 
              &dir::CopyOptions::new()).unwrap();
    let simple = writedir.join("simple");

    let mut w: Vec<u8> = Vec::new();

    let design = simple.join("design");

    let mut original_text = user::ProjectText::default();
    let mut _throw = HashSet::new();

    user::load_project_text(&mut original_text, &design, &mut _throw).unwrap();

    // basically try/finally for rust -- need to make sure we don't change
    // the actual data
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let repo = utils::find_repo(&simple).unwrap();
        let project = user::load_repo(&repo).unwrap();

        // validate several things about fmt:
        // - make sure that diff and list do something for an unformatted repo
        cmd::fmt::run_cmd(&mut w, &repo, &project, &cmd::fmt::Cmd::Diff).unwrap();
        assert_ne!(w.len(), 0);

        w.clear();
        cmd::fmt::run_cmd(&mut w, &repo, &project, &cmd::fmt::Cmd::List).unwrap();
        assert_ne!(w.len(), 0);

        // - actually run a fmt and make sure that diff/list detect no changes
        //    afterwards
        w.clear();
        cmd::fmt::run_cmd(&mut w, &repo, &project, &cmd::fmt::Cmd::Write).unwrap();
        cmd::fmt::run_cmd(&mut w, &repo, &project, &cmd::fmt::Cmd::Diff).unwrap();
        cmd::fmt::run_cmd(&mut w, &repo, &project, &cmd::fmt::Cmd::List).unwrap();
        assert_eq!(w.len(), 0);
    }));

    // restore original text
    original_text.dump().unwrap();
    let mut new_text = user::ProjectText::default();
    _throw.clear();
    user::load_project_text(&mut new_text, &design, &mut _throw).unwrap();
    assert_eq!(original_text, new_text);
    result.unwrap();
}
