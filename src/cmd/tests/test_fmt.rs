/// test the fmt command

use std::panic;
use dev_prefix::*;
use cmd;
use core;
use super::TSIMPLE_DIR;


#[test]
fn test_fmt_security() {
    // make sure that we can't load invalid stuff
    let mut w: Vec<u8> = Vec::new();
    let design = core::tests::TINVALID_BOUNDS.join("repo").join("design");
    let repo = core::find_repo(&design).unwrap();
    let project = core::load_path(&repo).unwrap();
    let c = cmd::fmt::Cmd::Write;
    match cmd::fmt::run_cmd(&mut w, &repo, &project, &c) {
        Err(e) => {
            match *e.kind() {
                ErrorKind::Security(_) => { /* expected */ }
                _ => panic!("unexpected error: {:?}", e.display()),
            }
        }
        Ok(_) => panic!("fmt accidentally suceeded -- may need to reset with git"),
    }
}

/// partof: #TST-cmd-fmt
#[test]
fn test_fmt() {
    let mut w: Vec<u8> = Vec::new();
    let simple = TSIMPLE_DIR.lock().unwrap();
    let design = simple.join("design");

    let mut original_text = core::types::ProjectText::default();
    let mut _throw = HashSet::new();

    original_text.load(&design, &mut _throw).unwrap();

    // basically try/finally for rust -- need to make sure we don't change
    // the actual data
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let repo = simple.as_path();
        let project = core::load_path(&repo).unwrap();

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
    let mut new_text = core::types::ProjectText::default();
    _throw.clear();
    new_text.load(&design, &mut _throw).unwrap();
    assert_eq!(original_text, new_text);
    result.unwrap();
}
