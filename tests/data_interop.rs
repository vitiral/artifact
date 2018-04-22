extern crate artifact_data;
extern crate artifact_lib;
extern crate artifact_test;
extern crate ergo;
use artifact_test::{assert_stuff_data, run_generic_interop_test, run_generic_interop_tests,
                    INTEROP_TESTS_PATH};
use artifact_lib::*;
use ergo::*;

/// This runs the interop tests for artifact-data.
fn run_interop_tests<P: AsRef<Path>>(test_base: P) {
    run_generic_interop_tests(test_base, run_data_test);
}

fn run_data_test(project_path: PathDir) {
    run_generic_interop_test(
        project_path,
        (),
        read_project_shim,
        modify_project_shim,
        assert_stuff_data,
    );
}

/// Simply calls `artifact_data::read_project(project_path)`
///
/// Used to satisfy the type requirements of `Fn` (cannot accept `AsRef`)
fn read_project_shim(
    project_path: PathDir,
    _state: (),
) -> Result<(lint::Categorized, Project), lint::Categorized> {
    artifact_data::read_project(project_path)
}

/// Simply calls `artifact_data::modify_project(project_path, operations)`
///
/// Used to satisfy the type requirements of `Fn` (cannot accept `AsRef`)
fn modify_project_shim(
    project_path: PathDir,
    operations: Vec<ArtifactOp>,
    _state: (),
) -> Result<(lint::Categorized, Project), artifact_data::ModifyError> {
    artifact_data::modify_project(project_path, operations)
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
