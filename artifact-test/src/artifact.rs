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
//! This module is for testing fully realized artifacts.

use super::dev_prelude::*;
use artifact_lib::*;
use super::raw::arb_raw_artifacts;
use super::implemented::random_impl_links;

const GEN_REL_FILE_PATH_RE: &str = r#"(?x)
([a-zA-Z0-9_]{1,7}/){0,3}          # an optional number of subdirs
[a-zA-Z0-9_]{1,7}.(md|json|toml)   # the required file name
"#;

pub fn arb_rel_file_path() -> BoxedStrategy<String> {
    GEN_REL_FILE_PATH_RE.prop_map(|s| s.to_string())
        .boxed()
}

/// Arbitrary _relative_ file paths.
///
/// Always generates at least 1. size must be `> 0`
pub fn arb_rel_file_paths(size: usize) -> BoxedStrategy<HashSet<String>> {
    prop::collection::hash_set(arb_rel_file_path(), 1..size)
        .boxed()
}

// TODO: use the file paths above to construct the artifacts.
// pub fn arb_artifacts(size: usize) -> BoxedStrategy<BTreeMap<Name, Artifact>> {
//     arb_raw_artifacts(size)
//         .prop_perturb(|artifacts, mut rng| {
//             unimplemented!()
//         })
//         .boxed()
// }

// fn get_project_paths(raw: BTreeMap<Name, ArtifactRaw>) -> (ProjectPaths, BTreeMap<Name, Path>) {
//     unimplemented!();
// }
