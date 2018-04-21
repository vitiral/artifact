/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
//! #TST-framework
//!
//! This module defines the interop "framework" that is leveraged for
//! a variety of integration/interop testing.

use time;
use ergo::yaml;

use super::dev_prelude::*;
use artifact_lib::expected::*;
use artifact_lib::*;
use artifact_data;

/// This runs the interop tests for artifact-data.
///
/// TODO: move this to artifact data
pub fn run_interop_tests<P: AsRef<Path>>(test_base: P) {
    run_generic_interop_tests(
        test_base,
        read_project_shim,
        modify_project_shim,
        assert_stuff_direct,
    );
}

/// Run the generic interop tests.
///
/// Directory structure:
/// ```no_compile
/// test/
///     assert-cases/
///         test-case-a/
///             modify.yaml  <-- modification commands to execute
///             project.yaml
///             ... etc
///         test-case-b/
///             modify.yaml
///             project.yaml
///             ... etc
///     # assert-case  <-- this is created by the framework
///         meta.json
///         modify.yaml
///         project.yaml
///         ... etc
/// ```
pub fn run_generic_interop_tests<P, READ, MODIFY, ASSERT>(
    test_base: P,
    read_project: READ,
    modify_project: MODIFY,
    assert_stuff: ASSERT,
) where
    P: AsRef<Path>,
    READ: Clone + Fn(PathDir) -> result::Result<(lint::Categorized, Project), lint::Categorized>,
    MODIFY: Clone + Fn(PathDir, Vec<ArtifactOp>)
        -> result::Result<(lint::Categorized, Project), artifact_data::ModifyError>,
    ASSERT: Clone + Fn(
        Option<Categorized>,
        Option<Categorized>,
        Option<Project>,
        Categorized,
        Option<Project>
    ),
{
    eprintln!(
        "Running interop test suite: {}",
        test_base.as_ref().display()
    );
    let test_base = expect!(PathDir::new(test_base));
    let testcases = expect!(PathDir::new(test_base.join("assert-cases")));
    for testcase in expect!(testcases.list()) {
        let testcase = expect!(testcase).unwrap_dir();

        // Do a deepcopy to a tmpdir and run the test out of there.
        let tmp = expect!(PathTmp::create("test-"));
        let project_path = {
            let project_path = tmp.join(expect!(test_base.file_name()));
            let (send_err, recv_err) = ch::bounded(128);
            deep_copy(send_err, test_base.clone(), project_path.clone());
            let errs: Vec<_> = recv_err.iter().collect();
            assert!(errs.is_empty(), "Got IO Errors:\n{:#?}", errs);
            expect!(PathDir::new(project_path))
        };

        // Copy the assertions into the root
        for assert in expect!(testcase.list()) {
            let assert = expect!(assert).unwrap_file();
            let fname = expect!(assert.file_name());
            expect!(assert.copy(project_path.join(fname)));
        }

        eprintln!(
            "  ----- Running Testcase {:?}:{:?} -----",
            expect!(test_base.file_name()),
            expect!(testcase.file_name())
        );
        run_generic_interop_test(
            project_path.clone(),
            read_project.clone(),
            modify_project.clone(),
            assert_stuff.clone(),
        );
    }
}

/// Simply calls `artifact_data::read_project(project_path)`
///
/// Used to satisfy the type requirements of `Fn` (cannot accept `AsRef`)
pub fn read_project_shim(project_path: PathDir
) -> result::Result<(lint::Categorized, Project), lint::Categorized> {
    artifact_data::read_project(project_path)
}

/// Simply calls `artifact_data::modify_project(project_path, operations)`
///
/// Used to satisfy the type requirements of `Fn` (cannot accept `AsRef`)
pub fn modify_project_shim(
    project_path: PathDir,
    operations: Vec<ArtifactOp>,
) -> ::std::result::Result<(lint::Categorized, Project), artifact_data::ModifyError> {
    artifact_data::modify_project(project_path, operations)
}

pub fn run_generic_interop_test<P, READ, MODIFY, ASSERT>(
    project_path: P,
    read_project: READ,
    modify_project: MODIFY,
    assert_stuff: ASSERT,
) where
    P: AsRef<Path>,
    READ: Fn(PathDir) -> result::Result<(lint::Categorized, Project), lint::Categorized>,
    MODIFY: Fn(PathDir, Vec<ArtifactOp>)
        -> result::Result<(lint::Categorized, Project), artifact_data::ModifyError>,
    ASSERT: Fn(
        Option<Categorized>,
        Option<Categorized>,
        Option<Project>,
        Categorized,
        Option<Project>
    ),
{
    static MODIFY_NAME: &'static str = "modify.yaml";
    let project_path = PathDir::new(project_path).unwrap();

    // Run the project against the copied directory
    let start = time::get_time();
    let expect_load_lints = load_lints(&project_path, "assert_load_lints.yaml");
    let expect_project_lints = load_lints(&project_path, "assert_project_lints.yaml");
    let expect_project = load_project(&project_path).map(|p| p.expected(&project_path));
    let modify_path = project_path.join(MODIFY_NAME);
    let expect_modify_fail = load_lints(&project_path, "assert_modify_fail.yaml");
    let expect_modify_lints = load_lints(&project_path, "assert_modify_lints.yaml");

    eprintln!("loaded asserts in {:.3}", time::get_time() - start);

    let (load_lints, project) = match read_project(project_path.clone()) {
        Ok(v) => v,
        Err(load_lints) => {
            assert!(!modify_path.exists(), "cannot modify non-existant project");
            assert_stuff(
                expect_load_lints,
                expect_project_lints,
                expect_project,
                load_lints,
                None,
            );
            return;
        }
    };

    match load_modify(&project_path, &project, MODIFY_NAME) {
        None => {
            assert_stuff(
                expect_load_lints,
                expect_project_lints,
                expect_project,
                load_lints,
                Some(project),
            );
        }
        Some(operations) => match modify_project(project_path.clone(), operations) {
            Ok((lints, project)) => {
                if let Some(expect) = expect_modify_lints {
                    eprintln!("asserting modify lints");
                    assert_eq!(expect, lints);
                }

                let (load_lints, expect) = read_project(project_path.clone()).unwrap();
                assert_eq!(expect, project);
                assert_stuff(
                    expect_load_lints,
                    expect_project_lints,
                    expect_project,
                    load_lints,
                    Some(project),
                );
                assert!(expect_modify_fail.is_none());
            }
            Err(err) => {
                assert_eq!(expect_modify_fail, Some(err.lints));
                assert!(expect_load_lints.is_none());
                assert!(expect_project.is_none());
                assert!(expect_project_lints.is_none());
                assert!(expect_modify_lints.is_none());
            }
        },
    };
}

pub fn assert_stuff_direct(
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

/// Load the assertions from the `project_path/assert.yaml` file
fn load_project(base: &PathDir) -> Option<ProjectAssert> {
    match PathFile::new(base.join("assert_project.yaml")) {
        Ok(p) => Some(yaml::from_str(&p.read_string().unwrap()).unwrap()),
        Err(_) => None,
    }
}

fn load_modify(base: &PathDir, project: &Project, fname: &str) -> Option<Vec<ArtifactOp>> {
    match PathFile::new(base.join(fname)) {
        Ok(p) => {
            let mut assert: Vec<ArtifactOpAssert> =
                expect!(yaml::from_str(&expect!(p.read_string())));
            // If the id is given, just use that.
            //
            // Otherwise pull it from the artifact name
            let get_id = |id, name: &Name| -> HashIm {
                if let Some(id) = id {
                    return id;
                }
                eprintln!("Getting id for name: {}", name.as_str());
                match project.artifacts.get(name) {
                    Some(art) => art.id,
                    None => HashIm([0; 16]),
                }
            };

            let out = assert
                .drain(..)
                .map(|m| match m {
                    ArtifactOpAssert::Create { artifact } => ArtifactOp::Create {
                        artifact: artifact.expected(base),
                    },
                    ArtifactOpAssert::Update { artifact, name, id } => ArtifactOp::Update {
                        orig_id: get_id(id, &name),
                        artifact: artifact.expected(base),
                    },
                    ArtifactOpAssert::Delete { name, id } => ArtifactOp::Delete {
                        orig_id: get_id(id, &name),
                        name: name,
                    },
                })
                .collect();
            Some(out)
        }
        Err(_) => None, // no modifications
    }
}

fn load_lints(base: &PathDir, fname: &str) -> Option<Categorized> {
    match PathFile::new(base.join(fname)) {
        Ok(p) => {
            let out: CategorizedAssert = yaml::from_str(&p.read_string().unwrap()).unwrap();
            let mut out = out.expected(base);
            out.sort();
            Some(out)
        }
        Err(_) => None,
    }
}
