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
//! This module is for testing fully realized artifacts.

use super::dev_prelude::*;
use super::implemented::random_impl_links;
use super::raw::arb_raw_artifacts;
use artifact_lib::*;

const GEN_REL_FILE_PATH_RE: &str = r#"(?x)
([a-zA-Z0-9_]{1,7}/){0,3}          # an optional number of subdirs
[a-zA-Z0-9_]{1,7}.(md|json|toml)   # the required file name
"#;

pub fn arb_rel_file_path() -> BoxedStrategy<String> {
    GEN_REL_FILE_PATH_RE.prop_map(|s| s.to_string()).boxed()
}

/// Arbitrary _relative_ file paths.
///
/// Always generates at least 1. size must be `> 0`
pub fn arb_rel_file_paths(size: usize) -> BoxedStrategy<HashSet<String>> {
    prop::collection::hash_set(arb_rel_file_path(), 1..size).boxed()
}

// TODO: use the file paths above to construct the artifacts.
// pub fn arb_artifacts(size: usize) -> BoxedStrategy<BTreeMap<Name, Artifact>> {
//     arb_raw_artifacts(size)
//         .prop_perturb(|artifacts, mut rng| {
//             unimplemented!()
//         })
//         .boxed()
// }

// fn get_settings(raw: BTreeMap<Name, ArtifactRaw>) -> (Settings, BTreeMap<Name, Path>) {
//     unimplemented!();
// }
