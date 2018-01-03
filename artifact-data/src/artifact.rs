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

use petgraph::graphmap::DiGraphMap;
use petgraph::{self, Direction};

use dev_prelude::*;
use raw::ArtifactRaw;
use raw_names::NamesRaw;
use name::{self, Name, SubName, Type};
use implemented::{Impl, ImplCode};
use path_abs::PathAbs;
use family;
use graph;

/// #SPC-data-structs.artifact
/// The primary data structure of this library which encapsulates a majority of the useful
/// end product of a user's project.
pub struct Artifact {
    /// The user defined and calculated `partof` the artifact.
    pub partof: OrderSet<Name>,
    /// The (calculated) parts of the artifact (opposite of partof)
    pub parts: OrderSet<Name>,
    /// The (calculated) completion+tested ratios of the artifact.
    pub completed: graph::Completed,
    /// The user defined text
    pub text: String,
    /// Whether the artifact is implemented directly (in code or `done` field)
    pub impl_: Impl,
    /// Subnames in the text.
    pub subnames: OrderSet<SubName>,
    /// The file the artifact is defined in.
    pub file: PathAbs,
}

/// Loaded and somewhat processed artifacts, independent of source implementations.
pub(crate) struct ArtifactsLoaded {
    raw_artifacts: OrderMap<Name, ArtifactRaw>,
    graphs: graph::Graphs,
    partofs: OrderMap<Name, OrderSet<Name>>,
    parts: OrderMap<Name, OrderSet<Name>>,
    subnames: OrderMap<Name, OrderSet<SubName>>,
}

/// Compute everything that is possible based on loaded raw artifacts only.
/// (no source impls).
pub(crate) fn finalize_load_artifact(
    raw_artifacts: OrderMap<Name, ArtifactRaw>,
) -> ArtifactsLoaded {
    let subnames = determine_subnames(&raw_artifacts);

    // detemrine partofs, create graphs and use that to determine parts
    let partofs = determine_partofs(&raw_artifacts);
    let graphs = graph::determine_graphs(&partofs);
    let parts = graph::determine_parts(&graphs);

    ArtifactsLoaded {
        raw_artifacts: raw_artifacts,
        graphs: graphs,
        partofs: partofs,
        parts: parts,
        subnames: subnames,
    }
}

/// Given the fully loaded artifacts (and related pieces) and code implementations,
/// calculate the completeness and construct the artifacts.
pub(crate) fn determine_artifacts(
    mut loaded: ArtifactsLoaded,
    impl_codes: &OrderMap<Name, ImplCode>,
    files: &OrderMap<Name, PathAbs>,
) -> OrderMap<Name, Artifact> {
    let mut impls = determine_impls(&loaded.raw_artifacts, &loaded.subnames, impl_codes);
    let mut completed = graph::determine_completeness(&loaded.graphs, &impls, &loaded.subnames);

    fn remove<T>(map: &mut OrderMap<Name, T>, name: &Name) -> T {
        map.remove(name).unwrap()
    }

    files
        .iter()
        .map(|(name, file)| {
            let art = Artifact {
                partof: remove(&mut loaded.partofs, name),
                parts: remove(&mut loaded.parts, name),
                completed: remove(&mut completed, name),
                // The only thing left in `ArtifactRaw` that we care
                // about is the `text`
                text: remove(&mut loaded.raw_artifacts, name)
                    .text
                    .map(|t| t.0)
                    .unwrap_or("".into()),
                impl_: remove(&mut impls, name),
                subnames: remove(&mut loaded.subnames, name),
                file: file.clone(),
            };
            (name.clone(), art)
        })
        .collect()
}

// // TODO: lints are not covered here.
// // - partof that doesn't exist
// //   - parent that doesn't exist
// // - check for type validity in "partof"
// // - impl_code can not conflict with done
// // - impl_code with no corresponding raw artifact
// // - impl_code with no corresponding raw artifact subname
// fn lint_artifacts() {
//
// }

/// Determine `partof` based on the user's definition + automatic relationships.
fn determine_partofs(
    raw_artifacts: &OrderMap<Name, ArtifactRaw>,
) -> OrderMap<Name, OrderSet<Name>> {
    let mut partofs = family::auto_partofs(&raw_artifacts);
    // extend the user defined partofs with the automatic ones
    for (name, partof) in partofs.iter_mut() {
        if let Some(ref p) = raw_artifacts[name].partof {
            partof.extend(p.iter().cloned());
        }
    }
    partofs
}

/// Parse the raw artifacts for their subnames.
fn determine_subnames(
    raw_artifacts: &OrderMap<Name, ArtifactRaw>,
) -> OrderMap<Name, OrderSet<SubName>> {
    raw_artifacts
        .iter()
        .map(|(name, raw)| {
            let subnames = match raw.text {
                Some(ref t) => name::parse_subnames(t),
                None => OrderSet::new(),
            };
            (name.clone(), subnames)
        })
        .collect()
}

/// Determine the valid implementation as well as the subnames for each artifact
fn determine_impls(
    raw_artifacts: &OrderMap<Name, ArtifactRaw>,
    all_subnames: &OrderMap<Name, OrderSet<SubName>>,
    impl_codes: &OrderMap<Name, ImplCode>,
) -> OrderMap<Name, Impl> {
    let mut impls = OrderMap::with_capacity(raw_artifacts.len());
    for (name, raw) in raw_artifacts.iter() {
        let subnames = &all_subnames[name];
        let impl_ = if let Some(ref done) = raw.done {
            Impl::Done(done.clone())
        } else if let Some(ref code) = impl_codes.get(name) {
            // Remove subnames that do not exist.
            // Note: this is linted against later.
            let remove: Vec<_> = code.secondary
                .keys()
                .filter(|sub| !subnames.contains(*sub))
                .cloned()
                .collect();
            let mut code = (*code).clone();
            for sub in &remove {
                code.secondary.remove(sub);
            }
            Impl::Code(code)
        } else {
            Impl::NotImpl
        };
        impls.insert(name.clone(), impl_);
    }
    impls
}
