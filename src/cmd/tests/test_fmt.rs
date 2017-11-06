/// test the fmt command

use std::panic;
use dev_prefix::*;
use cmd;
use user;
use test_data;
use utils;

use tempdir;
use fs_extra::dir;

/// partof: #TST-cmd-fmt
#[test]
fn test_fmt() {
    let tmpdir = tempdir::TempDir::new("artifact").unwrap();
    let writedir = tmpdir.path();
    dir::copy(
        &test_data::TSIMPLE_DIR.as_path(),
        &writedir,
        &dir::CopyOptions::new(),
    ).unwrap();
    let simple = writedir.join("simple");

    let mut w: Vec<u8> = Vec::new();

    let design = simple.join("design");

    let mut original_text = user::ProjectText::default();
    let mut _throw = HashSet::new();

    user::load_file_path(&mut original_text, &design, &mut _throw).unwrap();

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
    user::load_file_path(&mut new_text, &design, &mut _throw).unwrap();
    assert_eq!(original_text, new_text);
    result.unwrap();
}
