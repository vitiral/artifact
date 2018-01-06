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
//! Module for constructing and processing graphs of artifacts.
use petgraph::graphmap::DiGraphMap;
use petgraph::{self, Direction};

use dev_prelude::*;
use raw::ArtifactRaw;
use raw_names::NamesRaw;
use name::{self, Name, SubName, Type};
use implemented::{Impl, ImplCode};
use path_abs::PathAbs;
use family;

pub(crate) type GraphId = u32;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Copy, Serialize, Deserialize)]
/// #SPC-data-structures.completed
pub struct Completed {
    /// The specification completion ratio.
    pub spc: f32,
    /// The tested completion ratio.
    pub tst: f32,
}

pub(crate) struct Graphs {
    /// Map of `id => name`
    pub lookup_name: OrderMap<GraphId, Name>,
    /// Map of `name => id`
    pub lookup_id: OrderMap<Name, GraphId>,
    /// Full graph (all artifacts)
    pub full: DiGraphMap<GraphId, ()>,
    /// Graph of only REQ and SPC types
    pub req_spc: DiGraphMap<GraphId, ()>,
    /// Graph of only TST types
    pub tst: DiGraphMap<GraphId, ()>,
}

/// #SPC-data-artifact.graph
/// Create the family graph from their given+auto partof values.
pub(crate) fn determine_graphs(partofs: &OrderMap<Name, OrderSet<Name>>) -> Graphs {
    let ids = create_ids(partofs);

    let mut graph_full: DiGraphMap<GraphId, ()> = DiGraphMap::new();
    let mut graph_req_spc: DiGraphMap<GraphId, ()> = DiGraphMap::new();
    let mut graph_tst: DiGraphMap<GraphId, ()> = DiGraphMap::new();
    for (name, partof) in partofs.iter() {
        for p in partof.iter() {
            let edge = (ids[p], ids[name]);
            graph_full.add_edge(edge.0, edge.1, ());
            if matches!(name.ty, Type::TST) && matches!(p.ty, Type::TST) {
                graph_tst.add_edge(edge.0, edge.1, ());
            } else {
                graph_req_spc.add_edge(edge.0, edge.1, ());
            }
        }
    }

    let lookup_name = ids.iter().map(|(n, i)| (*i, n.clone())).collect();
    Graphs {
        lookup_id: ids,
        lookup_name: lookup_name,
        full: graph_full,
        req_spc: graph_req_spc,
        tst: graph_tst,
    }
}

/// Determine the `parts` of each artifact based on its neighbors in the graph.
pub(crate) fn determine_parts(graphs: &Graphs) -> OrderMap<Name, OrderSet<Name>> {
    graphs
        .lookup_name
        .iter()
        .map(|(id, name)| {
            let parts = graphs
                .full
                .neighbors(*id)
                .map(|i| graphs.lookup_name[&i].clone())
                .collect();
            (name.clone(), parts)
        })
        .collect()
}

/// #SPC-data-artifact.completed
/// Determine the completeness of the artifacts.
///
/// Basic idea:
/// - topologically sort the `graphs.tst` and calculate completeness (impl+test)
/// - topologically sort `graphs.req_spc` and calculate completeness
pub(crate) fn determine_completed(
    graphs: &Graphs,
    impls: &OrderMap<Name, Impl>,
    subnames: &OrderMap<Name, OrderSet<SubName>>,
) -> OrderMap<Name, Completed> {
    // If there is a cycle we just return everything as 0% complete for spc+tst
    // We ignore `done` because there will be an ERROR lint later anyway.
    let uncomputed = || {
        impls
            .keys()
            .map(|n| (n.clone(), Completed::default()))
            .collect()
    };
    let sorted_req_spc = match petgraph::algo::toposort(&graphs.req_spc, None) {
        Ok(s) => s,
        // cycle detected
        Err(_) => return uncomputed(),
    };
    let sorted_tst = match petgraph::algo::toposort(&graphs.tst, None) {
        Ok(s) => s,
        // cycle detected
        Err(_) => return uncomputed(),
    };

    // convert to by-id
    let impls: OrderMap<GraphId, &_> = impls
        .iter()
        .map(|(name, v)| (graphs.lookup_id[name], v))
        .collect();

    let mut implemented: OrderMap<GraphId, f32> = OrderMap::with_capacity(impls.len());

    /// compute ratio but ignore count=0
    fn ratio(value: f32, count: usize) -> f32 {
        if count == 0 {
            0.0
        } else {
            value / count as f32
        }
    }

    // topologically sorted means that we can always compute the results of any node
    // based on the previously computed values.
    for id in sorted_tst.iter().rev() {
        // ignore secondary since everything (code+done) always contributes to both.
        let (mut count, mut spc, _, _) =
            impls[id].to_statistics(&subnames[&graphs.lookup_name[id]]);
        for part_id in graphs.tst.neighbors(*id) {
            spc += implemented[&part_id];
            count += 1;
        }
        implemented.insert(*id, ratio(spc, count));
    }
    // TST types are as tested as they are implemented (by definition)
    let mut tested: OrderMap<GraphId, f32> = implemented.iter().map(|(a, b)| (*a, *b)).collect();

    // We already computed the TST types, so we just have to compute the req+spc types.
    for id in sorted_req_spc.iter().rev() {
        let (mut count_spc, mut spc, mut count_tst, mut tst) =
            impls[id].to_statistics(&subnames[&graphs.lookup_name[id]]);
        for part_id in graphs.req_spc.neighbors(*id) {
            // every type contributes to `tst` ratio
            tst += tested[&part_id];
            count_tst += 1;

            if !matches!(graphs.lookup_name[&part_id].ty, Type::TST) {
                // TST type does not contribute to `spc` ratio
                spc += implemented[&part_id];
                count_spc += 1;
            }
        }
        tested.insert(*id, ratio(tst, count_tst));
        implemented.insert(*id, ratio(spc, count_spc));
    }

    implemented
        .iter()
        .map(|(id, spc)| {
            let compl = Completed {
                spc: *spc,
                tst: tested[id],
            };
            (graphs.lookup_name[id].clone(), compl)
        })
        .collect()
}

// IMPLEMENTATION DETAILS

/// Create ids for the graph based on the names.
fn create_ids<T>(names: &OrderMap<Name, T>) -> OrderMap<Name, GraphId> {
    names
        .keys()
        .enumerate()
        .map(|(i, n)| (n.clone(), i as GraphId))
        .collect()
}
