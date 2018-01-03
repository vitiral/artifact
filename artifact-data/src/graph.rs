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

#[derive(Debug, Default, Clone, PartialEq, Copy)]
/// #SPC-data-structures.completed
pub struct Completed {
    /// The specification completion ratio.
    spc: f32,
    /// The tested completion ratio.
    tst: f32,
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

/// Create the family graph from the previously computed values.
///
/// - `parts` can be determined by `graph.neighbors(id)`
/// - `partof` can be determined by `graph.neighbors_directed(id, Direction::Incomming)`
pub(crate) fn determine_graphs(partofs: &OrderMap<Name, OrderSet<Name>>) -> Graphs {
    let ids = create_ids(&partofs);
    let mut graph_full: DiGraphMap<GraphId, ()> = DiGraphMap::new();
    let mut graph_req_spc: DiGraphMap<GraphId, ()> = DiGraphMap::new();
    let mut graph_tst: DiGraphMap<GraphId, ()> = DiGraphMap::new();
    for (name, partof) in partofs.iter() {
        for p in partof.iter() {
            let edge = (ids[p], ids[name].clone());
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

/// Determine the completeness of the artifacts.
///
/// Basic idea:
/// - topologically sort the tests and calculate completeness (impl+test)
/// - topologically sort req_spc and calculate completeness
///     - keep in mind that tests don't contribute to impl.
///     - everything else always contributes both
pub(crate) fn determine_completeness(
    graphs: &Graphs,
    impls: &OrderMap<Name, Impl>,
    subnames: &OrderMap<Name, OrderSet<SubName>>,
) -> OrderMap<Name, Completed> {
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
    for id in &sorted_tst {
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
    for id in &sorted_req_spc {
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
