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

use serde_yaml;

use test::dev_prelude::*;
use implemented::{CodeLoc, ImplCode};
use name::{Name, SubName};
use path_abs::{self, PathAbs};
use settings::{load_project_paths, ProjectPaths};
use lint;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct AssertionsRaw {
    code_paths: Vec<String>,
    // implemented: BTreeMap<Name, ImplCode>,
}

pub struct Assertions {
    project_paths: ProjectPaths,
}

impl Assertions {
    pub fn load(project_path: &PathAbs) -> Assertions {
        let s = project_path
            .join_abs("assert.yaml")
            .unwrap()
            .read()
            .unwrap();
        let raw: AssertionsRaw = serde_yaml::from_str(&s).unwrap();
        Assertions {
            project_paths: ProjectPaths {
                code: path_abs::prefix_paths(project_path, &raw.code_paths)
                    .unwrap()
                    // TODO(nll): drain doesn't work because of lifetimes?
                    .iter()
                    .cloned()
                    .collect(),
            },
        }
    }
}

/// Run the interop test on an example project.
pub fn run_interop_test<P: AsRef<Path>>(path: P) {
    let project_path = PathAbs::new(path).unwrap();
    let (project_paths, lints) = load_project_paths(&project_path).unwrap();
    let assertions = Assertions::load(&project_path);
    assert_eq!(lints, vec![]);
    assert_eq!(project_paths, assertions.project_paths);
}
