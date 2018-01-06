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
//! #TST-data-framework
//!
//! This module defines the interop "framework" that is leveraged for
//! a variety of integration/interop testing.

use std::sync::mpsc::channel;
use serde_yaml;

use test::dev_prelude::*;
use name::{Name, SubName};
use path_abs::PathAbs;
use artifact;
use implemented;
use settings;
use project;
use lint;

/// Run the interop test on an example project.
pub fn run_interop_test<P: AsRef<Path>>(path: P) {
    let project_path = PathAbs::new(path.as_ref()).expect("project_path DNE");
    let expect_load_lints = load_lints(&project_path, "assert_load_lints.yaml").unwrap_or_default();
    let expect_project_lints = load_lints(&project_path, "assert_project_lints.yaml");
    let expect_project = ProjectAssert::load(&project_path).map(|p| p.expected(&project_path));

    let (load_lints, project) = project::load_project(path.as_ref());

    assert_eq!(expect_load_lints, load_lints);
    assert_eq!(expect_project, project);
    if let Some(project) = project {
        let expect_project_lints = expect_project_lints.expect("project lints DNE");
        let project_lints = project.lint();
        assert_eq!(expect_project_lints, project_lints);
    } else {
        assert_eq!(expect_project_lints, None);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectAssert {
    paths: ProjectPathsAssert,
    code_impls: OrderMap<Name, ImplCodeAssert>,
    artifacts: OrderMap<Name, ArtifactAssert>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectPathsAssert {
    code: Vec<String>,
    artifact: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArtifactAssert {
    file: String,
    partof: OrderSet<Name>,
    parts: OrderSet<Name>,
    completed: ::graph::Completed,
    text: String,
    impl_: ImplAssert,
    subnames: OrderSet<SubName>,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl ProjectAssert {
    /// Load the assertions from the `project_path/assert.yaml` file
    fn load(base: &PathAbs) -> Option<ProjectAssert> {
        match PathAbs::new(base.join("assert_project.yaml")) {
            Ok(p) => Some(serde_yaml::from_str(&p.read().unwrap()).unwrap()),
            Err(_) => None,
        }
    }

    /// Get the "expected" value based on this assertion object.
    fn expected(mut self, base: &PathAbs) -> project::Project {
        let mut out = project::Project {
            paths: self.paths.expected(base),
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
    fn expected(self, base: &PathAbs) -> settings::ProjectPaths {
        settings::ProjectPaths {
            base: base.clone(),
            code: prefix_paths(base, &self.code),
            artifact: prefix_paths(base, &self.artifact),
        }
    }
}

impl ArtifactAssert {
    fn expected(self, base: &PathAbs) -> artifact::Artifact {
        artifact::Artifact {
            file: join_abs(base, &self.file),
            partof: self.partof,
            parts: self.parts,
            completed: self.completed,
            text: self.text,
            impl_: self.impl_.expected(base),
            subnames: self.subnames,
        }
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
            *p = base.join(&p);
        }
    }
}

impl CategorizedAssert {
    fn expected(mut self, base: &PathAbs) -> lint::Categorized {
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
        lint::Categorized {
            error: self.error,
            other: self.other,
        }
    }
}

fn load_lints(base: &PathAbs, fname: &str) -> Option<lint::Categorized> {
    match PathAbs::new(base.join(fname)) {
        Ok(p) => {
            let out: CategorizedAssert = serde_yaml::from_str(&p.read().unwrap()).unwrap();
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
    ends.iter().map(|e| join_abs(base, e)).collect()
}
