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

use test::dev_prelude::*;
use name::{Name, SubName};
use artifact;
use implemented;
use intermediate::{self, ArtifactIm};
use settings;
use project;
use modify::{self, ArtifactOp};
use lint::{self, Categorized};

/// This runs the interop tests.
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
pub fn run_interop_tests<P: AsRef<Path>>(test_base: P) {
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
        run_interop_test(project_path);
    }
}

/// Run the interop test on an example project.
fn run_interop_test(project_path: PathDir) {
    static MODIFY_NAME: &'static str = "modify.yaml";

    // Run the project against the copied directory
    let start = time::get_time();
    let expect_load_lints = load_lints(&project_path, "assert_load_lints.yaml");
    let expect_project_lints = load_lints(&project_path, "assert_project_lints.yaml");
    let expect_project = ProjectAssert::load(&project_path).map(|p| p.expected(&project_path));
    let modify_path = project_path.join(MODIFY_NAME);
    let expect_modify_fail = load_lints(&project_path, "assert_modify_fail.yaml");
    let expect_modify_lints = load_lints(&project_path, "assert_modify_lints.yaml");

    eprintln!("loaded asserts in {:.3}", time::get_time() - start);

    let (load_lints, project) = match project::read_project(&project_path) {
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
        Some(operations) => match modify::modify_project(&project_path, operations) {
            Ok((lints, project)) => {
                if let Some(expect) = expect_modify_lints {
                    eprintln!("asserting modify lints");
                    assert_eq!(expect, lints);
                }

                let (load_lints, expect) = project::read_project(&project_path).unwrap();
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

fn assert_stuff(
    expect_load_lints: Option<Categorized>,
    expect_project_lints: Option<Categorized>,
    expect_project: Option<project::Project>,
    load_lints: Categorized,
    project: Option<project::Project>,
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

    if let Some(expect_project) = expect_project {
        eprintln!("asserting projects");
        assert_eq!(expect_project, project);
    }

    if let Some(expect) = expect_project_lints {
        let lints = project.lint();
        eprintln!("asserting project_lints");
        assert_eq!(expect, lints);
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
enum ArtifactOpAssert {
    Create {
        artifact: ArtifactImAssert,
    },
    Update {
        artifact: ArtifactImAssert,
        name: Name,
        /// Example: "gQ7cdQ7bvyIoaUTEUsxMsg"
        id: Option<intermediate::HashIm>,
    },
    Delete {
        name: Name,
        /// Example: "gQ7cdQ7bvyIoaUTEUsxMsg"
        id: Option<intermediate::HashIm>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ArtifactImAssert {
    name: Name,
    file: String,
    partof: OrderSet<Name>,
    done: Option<String>,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectAssert {
    paths: ProjectPathsAssert,
    code_impls: OrderMap<Name, ImplCodeAssert>,
    artifacts: OrderMap<Name, ArtifactAssert>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectPathsAssert {
    code_paths: Vec<String>,
    exclude_code_paths: Vec<String>,
    artifact_paths: Vec<String>,
    exclude_artifact_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArtifactAssert {
    name: Name,
    file: String,
    partof: OrderSet<Name>,
    parts: OrderSet<Name>,
    completed: ::graph::Completed,
    text: String,
    impl_: ImplAssert,
    subnames: OrderSet<SubName>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ImplAssert {
    Done(String),
    Code(ImplCodeAssert),
    NotImpl,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImplCodeAssert {
    primary: Option<CodeLocAssert>,
    secondary: OrderMap<String, CodeLocAssert>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeLocAssert {
    file: String,
    line: u64,
}

#[derive(Debug, Serialize, Deserialize)]
/// Assertions for categorized lints.
struct CategorizedAssert {
    error: Vec<lint::Lint>,
    other: Vec<lint::Lint>,
}

impl ArtifactImAssert {
    fn expected(self, base: &PathDir) -> ArtifactIm {
        let mut out = ArtifactIm {
            name: self.name,
            file: PathArc::new(base.join(self.file)),
            partof: self.partof,
            done: self.done,
            text: self.text,
        };
        out.clean();
        out
    }
}

impl ProjectAssert {
    /// Load the assertions from the `project_path/assert.yaml` file
    fn load(base: &PathDir) -> Option<ProjectAssert> {
        match PathFile::new(base.join("assert_project.yaml")) {
            Ok(p) => Some(yaml::from_str(&p.read_string().unwrap()).unwrap()),
            Err(_) => None,
        }
    }

    /// Get the "expected" value based on this assertion object.
    fn expected(mut self, base: &PathDir) -> project::Project {
        let mut out = project::Project {
            paths: Arc::new(self.paths.expected(base)),
            code_impls: self.code_impls
                .drain(..)
                .map(|(name, impl_)| (name, impl_.expected(base)))
                .collect(),
            artifacts: self.artifacts
                .drain(..)
                .map(|(name, art)| (name, art.expected(base)))
                .collect(),
        };
        out.sort();
        out
    }
}

impl ProjectPathsAssert {
    fn expected(self, base: &PathDir) -> settings::ProjectPaths {
        settings::ProjectPaths {
            base: base.clone(),
            code_paths: prefix_paths(base, &self.code_paths),
            exclude_code_paths: prefix_paths(base, &self.exclude_code_paths),
            artifact_paths: prefix_paths(base, &self.artifact_paths),
            exclude_artifact_paths: prefix_paths(base, &self.exclude_artifact_paths),
        }
    }
}

impl ArtifactAssert {
    fn expected(self, base: &PathAbs) -> artifact::Artifact {
        let mut art = artifact::Artifact {
            id: intermediate::HashIm([0; 16]),
            name: self.name,
            file: PathArc::new(base.join(&self.file)),
            partof: self.partof,
            parts: self.parts,
            completed: self.completed,
            text: self.text,
            impl_: self.impl_.expected(base),
            subnames: self.subnames,
        };

        art.id = ArtifactIm::from(art.clone()).hash_im();
        art
    }
}

impl ImplAssert {
    fn expected(self, base: &PathAbs) -> implemented::Impl {
        match self {
            ImplAssert::Done(d) => implemented::Impl::Done(d),
            ImplAssert::Code(c) => implemented::Impl::Code(c.expected(base)),
            ImplAssert::NotImpl => implemented::Impl::NotImpl,
        }
    }
}

impl ImplCodeAssert {
    fn expected(mut self, base: &PathAbs) -> implemented::ImplCode {
        implemented::ImplCode {
            primary: self.primary.map(|c| c.expected(base)),
            secondary: self.secondary
                .drain(..)
                .map(|(s, c)| (subname!(s), c.expected(base)))
                .collect(),
        }
    }
}

impl CodeLocAssert {
    fn expected(self, base: &PathAbs) -> implemented::CodeLoc {
        implemented::CodeLoc {
            file: join_abs(base, self.file),
            line: self.line,
        }
    }
}

impl lint::Lint {
    /// just mutate the lint to be correct
    fn make_expected(&mut self, base: &PathAbs) {
        if let Some(ref mut p) = self.path {
            *p = PathArc::new(base.join(&p));
        }
    }
}

impl CategorizedAssert {
    fn expected(mut self, base: &PathAbs) -> Categorized {
        let convert_lints = |lints: &mut Vec<lint::Lint>| {
            lints
                .iter_mut()
                .map(|l| {
                    l.make_expected(base);
                })
                .count();
        };
        convert_lints(&mut self.error);
        convert_lints(&mut self.other);
        Categorized {
            error: self.error,
            other: self.other,
        }
    }
}

fn load_modify(base: &PathDir, project: &project::Project, fname: &str) -> Option<Vec<ArtifactOp>> {
    match PathFile::new(base.join(fname)) {
        Ok(p) => {
            let mut assert: Vec<ArtifactOpAssert> =
                expect!(yaml::from_str(&expect!(p.read_string())));
            // If the id is given, just use that.
            //
            // Otherwise pull it from the artifact name
            let get_id = |id, name: &Name| -> intermediate::HashIm {
                if let Some(id) = id {
                    return id;
                }
                eprintln!("Getting id for name: {}", name.as_str());
                match project.artifacts.get(name) {
                    Some(art) => art.id,
                    None => intermediate::HashIm([0; 16]),
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

// HELPERS

/// Add the path prefix to a list of strings
fn prefix_paths(base: &PathAbs, ends: &[String]) -> OrderSet<PathAbs> {
    ends.iter()
        .map(|e| match PathAbs::new(base.join(e)) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        })
        .collect()
}
