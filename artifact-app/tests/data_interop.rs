/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
//! Interop Tests:
//! - #TST-read-artifact
#[macro_use]
extern crate expect_macro;
use artifact_data;

use artifact_lib::*;
use artifact_test::{
    assert_stuff_data, run_generic_interop_test, run_generic_interop_tests, INTEROP_TESTS_PATH,
};
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
) -> Result<(lint::Categorized, Project), ModifyError> {
    // Do basic round-trip serialization
    let result = expect!(round_ser!(Vec<ArtifactOp>, operations));
    assert_eq!(operations, result);

    // Do round trip through `*Ser` types
    let operations_ser = expect!(round_ser!(Vec<ArtifactOpSer>, operations));
    let _result = expect!(round_ser!(Vec<ArtifactOp>, operations_ser));

    artifact_data::modify_project(project_path, operations)
}

#[test]
/// #TST-read-artifact.empty
fn data_interop_project_empty() {
    run_interop_tests(INTEROP_TESTS_PATH.join("empty"));
}

#[test]
/// #TST-read-artifact.source_only
fn data_interop_source_only() {
    run_interop_tests(INTEROP_TESTS_PATH.join("source_only"));
}

#[test]
/// #TST-read-artifact.source_invalid
fn data_interop_source_invalid() {
    run_interop_tests(INTEROP_TESTS_PATH.join("source_invalid"));
}

#[test]
/// #TST-read-artifact.design_only
fn data_interop_design_only() {
    run_interop_tests(INTEROP_TESTS_PATH.join("design_only"));
}

#[test]
/// #TST-read-artifact.basic
fn data_interop_basic() {
    run_interop_tests(INTEROP_TESTS_PATH.join("basic"));
}

#[test]
/// #TST-read-artifact.lints
fn data_interop_lints_error1() {
    run_interop_tests(INTEROP_TESTS_PATH.join("lints"));
}

#[test]
fn data_interop_lints_error2() {
    run_interop_tests(INTEROP_TESTS_PATH.join("lints2"));
}
