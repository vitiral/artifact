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
pub fn run_generic_interop_tests<P, TEST>(test_base: P, run_test: TEST)
where
    P: AsRef<Path>,
    TEST: Clone + Fn(PathDir),
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
        run_test(project_path.clone());
    }
}

/// This is the basic "data" way to run the test.
///
/// Other interop tests may want to wrap this to:
///
/// - Set up a server.
/// - Other test harness setup (i.e. selenium)
/// - etc.
pub fn run_generic_interop_test<P, STATE, READ, MODIFY, ASSERT>(
    project_path: P,
    state: STATE,
    read_project: READ,
    modify_project: MODIFY,
    assert_stuff: ASSERT,
) where
    P: AsRef<Path>,
    READ: Fn(PathDir, STATE) -> result::Result<(lint::Categorized, Project), lint::Categorized>,
    MODIFY: Fn(PathDir, Vec<ArtifactOp>, STATE)
        -> result::Result<(lint::Categorized, Project), artifact_data::ModifyError>,
    ASSERT: Fn(PathDir, STATE, Categorized, Option<Project>, ExpectStuff),
    STATE: Debug + Clone,
{
    static MODIFY_NAME: &'static str = "modify.yaml";
    let project_path = PathDir::new(project_path).unwrap();

    // Run the project against the copied directory
    let start = time::get_time();
    let modify_path = project_path.join(MODIFY_NAME);

    let expect = ExpectStuff {
        load_lints: load_lints(&project_path, "assert_load_lints.yaml"),
        project_lints: load_lints(&project_path, "assert_project_lints.yaml"),
        project: load_project(&project_path).map(|p| p.expected(&project_path)),
        modify_fail: load_lints(&project_path, "assert_modify_fail.yaml"),
        modify_lints: load_lints(&project_path, "assert_modify_lints.yaml"),
    };

    eprintln!("loaded ExpectStuff in {:.3}", time::get_time() - start);

    let (load_lints, project) = match read_project(project_path.clone(), state.clone()) {
        Ok(v) => v,
        Err(load_lints) => {
            assert!(!modify_path.exists(), "cannot modify non-existant project");
            assert_stuff(
                project_path.clone(),
                state.clone(),
                load_lints,
                None,
                expect,
            );
            return;
        }
    };

    match load_modify(&project_path, &project, MODIFY_NAME) {
        None => {
            assert_stuff(
                project_path.clone(),
                state.clone(),
                load_lints,
                Some(project),
                expect,
            );
        }
        Some(operations) => match modify_project(project_path.clone(), operations, state.clone()) {
            Ok((lints, project)) => {
                if let Some(ref expect_modify_lints) = expect.modify_lints {
                    eprintln!("asserting modify lints");
                    assert_eq!(expect_modify_lints, &lints);
                }

                let (load_lints, expect_project) =
                    read_project(project_path.clone(), state.clone()).unwrap();
                assert_eq!(expect_project, project);
                assert!(expect.modify_fail.is_none());
                assert_stuff(
                    project_path.clone(),
                    state.clone(),
                    load_lints,
                    Some(project),
                    expect,
                );
            }
            Err(err) => {
                assert_eq!(expect.modify_fail, Some(err.lints));
                assert!(expect.load_lints.is_none());
                assert!(expect.project.is_none());
                assert!(expect.project_lints.is_none());
                assert!(expect.modify_lints.is_none());
            }
        },
    };
}

/// Loaded stuff to expect
pub struct ExpectStuff {
    /// The project data itself
    pub project: Option<Project>,

    /// Lints during loading
    pub load_lints: Option<Categorized>,

    /// Lints from the project
    pub project_lints: Option<Categorized>,

    /// Lints from a failed modify
    pub modify_fail: Option<Categorized>,

    /// Lints during modify
    pub modify_lints: Option<Categorized>,
}

/// The "standard" assertions, used in artifact-data interop tests.
///
/// These are generally useful, but other suites probably want to build on them.
pub fn assert_stuff_data(
    _project_path: PathDir,
    state: (),
    load_lints: Categorized,
    project: Option<Project>,
    expect: ExpectStuff,
) {
    if let Some(expect) = expect.load_lints {
        eprintln!("asserting load lints");
        assert_eq!(expect, load_lints);
    }

    let project = match project {
        Some(p) => p,
        None => {
            assert!(
                expect.project.is_none(),
                "expected project but no project exists."
            );
            assert!(
                expect.project_lints.is_none(),
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

    if let Some(expect_project) = expect.project {
        eprintln!("asserting projects");
        assert_eq!(expect_project, project);
    }

    if let Some(expect) = expect.project_lints {
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
