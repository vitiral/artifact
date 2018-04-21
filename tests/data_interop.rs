extern crate ergo;
extern crate artifact_lib;
extern crate artifact_data;
extern crate artifact_test;
use artifact_test::{run_generic_interop_tests, INTEROP_TESTS_PATH};
use artifact_lib::*;
use ergo::*;

/// This runs the interop tests for artifact-data.
fn run_interop_tests<P: AsRef<Path>>(test_base: P) {
    run_generic_interop_tests(
        test_base,
        read_project_shim,
        modify_project_shim,
        assert_stuff_data,
    );
}

/// Simply calls `artifact_data::read_project(project_path)`
///
/// Used to satisfy the type requirements of `Fn` (cannot accept `AsRef`)
fn read_project_shim(project_path: PathDir
) -> Result<(lint::Categorized, Project), lint::Categorized> {
    artifact_data::read_project(project_path)
}

/// Simply calls `artifact_data::modify_project(project_path, operations)`
///
/// Used to satisfy the type requirements of `Fn` (cannot accept `AsRef`)
fn modify_project_shim(
    project_path: PathDir,
    operations: Vec<ArtifactOp>,
) -> Result<(lint::Categorized, Project), artifact_data::ModifyError> {
    artifact_data::modify_project(project_path, operations)
}

pub fn assert_stuff_data(
    expect_load_lints: Option<Categorized>,
    expect_project_lints: Option<Categorized>,
    expect_project: Option<Project>,
    load_lints: Categorized,
    project: Option<Project>,
) {
    if let Some(expect) = expect_load_lints {
        eprintln!("asserting load lints");
        assert_eq!(expect, load_lints);
    }

    let project = match project {
        Some(p) => p,
        None => {
            assert!(
                expect_project.is_none(),
                "expected project but no project exists."
            );
            assert!(
                expect_project_lints.is_none(),
                "expected project lints but no project exists."
            );
            return;
        }
    };

    {
        // Do basic round-trip serialization
        let result = round_ser!(Project, project).unwrap();
        assert_eq!(project, result);

        // Do round trip through `*Ser` types
        let project_ser = round_ser!(ProjectSer, project).unwrap();
        let result = round_ser!(Project, project_ser).unwrap();
        assert_eq!(project, result);
    }

    if let Some(expect_project) = expect_project {
        eprintln!("asserting projects");
        assert_eq!(expect_project, project);
    }

    if let Some(expect) = expect_project_lints {
        // let lints = project.lint();
        eprintln!("asserting project_lints");
        assert_eq!(expect, load_lints);
    }
}


#[test]
fn interop_source_only() {
    run_interop_tests(INTEROP_TESTS_PATH.join("source_only"));
}

#[test]
fn interop_source_invalid() {
    run_interop_tests(INTEROP_TESTS_PATH.join("source_invalid"));
}

// From Artifact

#[test]
/// #TST-read-artifact.empty
fn interop_project_empty() {
    run_interop_tests(INTEROP_TESTS_PATH.join("empty"));
}

#[test]
/// #TST-read-artifact.design_only
fn interop_design_only() {
    run_interop_tests(INTEROP_TESTS_PATH.join("design_only"));
}

#[test]
/// #TST-read-artifact.basic
fn interop_basic() {
    run_interop_tests(INTEROP_TESTS_PATH.join("basic"));
}

#[test]
/// #TST-read-artifact.lints
fn interop_lints() {
    run_interop_tests(INTEROP_TESTS_PATH.join("lints"));
}
