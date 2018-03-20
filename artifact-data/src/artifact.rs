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

//! #SPC-read-artifact

use rayon;

use dev_prelude::*;
use graph;

/// Loaded and somewhat processed artifacts, independent of source implementations.
pub(crate) struct ArtifactsLoaded {
    artifact_ims: IndexMap<Name, ArtifactIm>,
    graphs: graph::Graphs,
    partofs: IndexMap<Name, IndexSet<Name>>,
    parts: IndexMap<Name, IndexSet<Name>>,
    subnames: IndexMap<Name, IndexSet<SubName>>,
}

/// #SPC-read-artifact.load
/// Compute everything that is possible based on loaded raw artifacts only.
/// (no source impls).
pub(crate) fn finalize_load_artifact(artifact_ims: IndexMap<Name, ArtifactIm>) -> ArtifactsLoaded {
    let (subnames, (partofs, graphs, parts)) = rayon::join(
        || determine_subnames(&artifact_ims),
        || {
            // determine partofs, create graphs and use that to determine parts
            let partofs = determine_partofs(&artifact_ims);
            let graphs = graph::determine_graphs(&partofs);
            let parts = graph::determine_parts(&graphs);
            (partofs, graphs, parts)
        },
    );

    ArtifactsLoaded {
        artifact_ims: artifact_ims,
        graphs: graphs,
        partofs: partofs,
        parts: parts,
        subnames: subnames,
    }
}

/// #SPC-read-artifact.build
/// Given the fully loaded artifacts (+related pieces) and code implementations,
/// determine the impls+completeness and construct the artifacts.
pub(crate) fn determine_artifacts(
    mut loaded: ArtifactsLoaded,
    code_impls: &IndexMap<Name, ImplCode>,
    defined: &IndexMap<Name, PathArc>,
) -> IndexMap<Name, Artifact> {
    // TODO: paralize this

    let ((mut impls, mut completed), mut ids): (_, IndexMap<Name, HashIm>) = rayon::join(
        || {
            let impls = determine_impls(&loaded.artifact_ims, code_impls);
            let completed = graph::determine_completed(&loaded.graphs, &impls, &loaded.subnames);
            (impls, completed)
        },
        || {
            loaded
                .artifact_ims
                .iter()
                .map(|(name, art)| (name.clone(), art.hash_im()))
                .collect()
        },
    );

    macro_rules! remove {
        [$map:expr, $name:expr] => {
            $map.remove($name).unwrap()
        };
    }

    let out = defined
        .iter()
        .map(|(name, file)| {
            let art = Artifact {
                id: remove!(ids, name),
                name: name.clone(),
                partof: remove!(loaded.partofs, name),
                parts: remove!(loaded.parts, name),
                completed: remove!(completed, name),
                // The only thing left in `ArtifactIm` that we care
                // about is the `text`
                text: remove!(loaded.artifact_ims, name).text,
                impl_: remove!(impls, name),
                subnames: remove!(loaded.subnames, name),
                file: file.clone(),
            };
            (name.clone(), art)
        })
        .collect();

    debug_assert!(loaded.partofs.is_empty(), "{:#?}", loaded.partofs);
    // Note: Not necessarily true if someone specified invalid partof
    // debug_assert!(loaded.parts.is_empty(), "{:#?}", loaded.parts);
    debug_assert!(completed.is_empty(), "{:#?}", completed);
    debug_assert!(loaded.artifact_ims.is_empty(), "{:#?}", loaded.artifact_ims);
    debug_assert!(impls.is_empty(), "{:#?}", impls);
    debug_assert!(loaded.subnames.is_empty(), "{:#?}", loaded.subnames);
    out
}

/// Determine `partof` based on the user's definition + automatic relationships.
pub(crate) fn determine_partofs(
    artifact_ims: &IndexMap<Name, ArtifactIm>,
) -> IndexMap<Name, IndexSet<Name>> {
    let mut partofs = auto_partofs(artifact_ims);
    // extend the user defined partofs with the automatic ones
    for (name, partof) in partofs.iter_mut() {
        partof.extend(artifact_ims[name].partof.iter().cloned());
    }
    debug_assert_eq!(artifact_ims.len(), partofs.len());
    partofs
}

/// Parse the raw artifacts for their subnames.
fn determine_subnames(
    artifact_ims: &IndexMap<Name, ArtifactIm>,
) -> IndexMap<Name, IndexSet<SubName>> {
    artifact_ims
        .iter()
        .map(|(name, art)| (name.clone(), parse_subnames(&art.text)))
        .collect()
}

/// Merge the "done" field and the code implementations.
///
/// Note that the following may exist but will be linted against later:
/// - Some of the subnames in `ImplCode.secondary` may not be real subnames in the artifact's
///   `text`.
/// - Not all `code_impls` may be used (i.e. if they have an artifact that doesn't exist).
/// - Conflict with `done` is ignored here
///
/// None of these can affect later calculation of completeness or anythign else.
fn determine_impls(
    artifact_ims: &IndexMap<Name, ArtifactIm>,
    code_impls: &IndexMap<Name, ImplCode>,
) -> IndexMap<Name, Impl> {
    let mut impls = IndexMap::with_capacity(artifact_ims.len());
    for (name, raw) in artifact_ims.iter() {
        let impl_ = if let Some(ref done) = raw.done {
            Impl::Done(done.clone())
        } else if let Some(code) = code_impls.get(name) {
            Impl::Code(code.clone())
        } else {
            Impl::NotImpl
        };
        impls.insert(name.clone(), impl_);
    }
    impls
}
