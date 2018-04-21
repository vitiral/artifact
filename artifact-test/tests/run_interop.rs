extern crate artifact_test;
use artifact_test::{run_interop_tests, INTEROP_TESTS_PATH};

// From Implemented

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
