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
//! #TST-data-interop
//!
//! This module defines the interop "framework" that is leveraged for
//! a variety of integration/interop testing.

use std::sync::mpsc::channel;
use serde_yaml;

use test::dev_prelude::*;
use implemented::{self, CodeLoc, ImplCode};
use name::{Name, SubName};
use path_abs::PathAbs;
use settings::{load_project_paths, ProjectPaths};
use project::Project;
use lint;

/// Run the interop test on an example project.
pub fn run_interop_test<P: AsRef<Path>>(path: P) {
    let project_path = PathAbs::new(path).unwrap();
    let (project_paths, _load_lints) = {
        let (paths, mut lints) = load_project_paths(&project_path).unwrap();
        lints.sort();
        (paths, lints)
    };
    let (implemented, impl_lints) = {
        let (send_lints, recv_lints) = channel();
        let raw = implemented::load_locations(send_lints.clone(), &project_paths.code);
        let implemented = implemented::join_locations(&send_lints, raw);
        drop(send_lints);
        let mut lints: Vec<_> = recv_lints.into_iter().collect();
        (implemented, lints)
    };
    let mut project = Project {
        project_paths: project_paths,
        implemented: implemented,
        implemented_lints: impl_lints,
    };
    let mut expected = Project::load_from_assertions(&project_path);
    project.sort();
    expected.sort();
    assert_eq!(project, expected);
}

// # IMPLEMENTATION DETAILS
// This whole module is pretty much just creating a way to specify assertions as strings and then
// deserialize them into the actual types.
//
// Quite a lot of pain has to do with paths, which makes some sense since they ARE a pain.

impl Project {
    /// Load the assertions from the `project_path/assert.yaml` file
    pub fn load_from_assertions(project_path: &PathAbs) -> Project {
        let s = join_abs(project_path, "assert.yaml").read().unwrap();
        let raw: AssertionsRaw = serde_yaml::from_str(&s).unwrap();
        Project {
            implemented_lints: vec![],
            project_paths: ProjectPaths {
                code: prefix_paths(project_path, &raw.code_paths)
                    .iter()
                    .cloned()
                    .collect(),
                // TODO: implement this
                artifacts: orderset!(),
            },
            implemented: convert_implemented_raw(project_path, raw.implemented),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AssertionsRaw {
    // load_lints: Vec<lint::Lint>,
    code_paths: Vec<String>,

    implemented: OrderMap<Name, ImplCodeRaw>,
    implemented_lints: Vec<lint::Lint>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImplCodeRaw {
    primary: Option<CodeLocRaw>,
    secondary: OrderMap<String, CodeLocRaw>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeLocRaw {
    file: String,
    line: u64,
}

fn convert_code_loc(prefix: &PathAbs, raw: CodeLocRaw) -> CodeLoc {
    CodeLoc {
        file: join_abs(prefix, raw.file),
        line: raw.line,
    }
}

fn convert_implemented_raw(
    prefix: &PathAbs,
    mut raw: OrderMap<Name, ImplCodeRaw>,
) -> OrderMap<Name, ImplCode> {
    raw.drain(..)
        .map(|(name, mut raw)| {
            let code = ImplCode {
                primary: raw.primary.map(|r| convert_code_loc(prefix, r)),
                secondary: raw.secondary
                    .drain(..)
                    .map(|(s, r)| (subname!(s), convert_code_loc(prefix, r)))
                    .collect(),
            };
            (name, code)
        })
        .collect()
}

/// Prepend all paths with the project, sort and dedup
fn convert_lints(project_path: &PathAbs, mut lints: Vec<lint::Lint>) -> Vec<lint::Lint> {
    let lints: Vec<_> = lints
        .drain(0..)
        .map(|mut l| {
            l.path = l.path.map(|p| project_path.join(p));
            l
        })
        .collect();
    lints
}
