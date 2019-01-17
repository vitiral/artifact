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
//! Module for constructing and processing graphs of artifacts.
use petgraph;
use petgraph::graphmap::DiGraphMap;

use dev_prelude::*;

pub type GraphId = u32;

pub struct Graphs {
    /// Map of `id => name`
    pub lookup_name: IndexMap<GraphId, Name>,
    /// Map of `name => id`
    pub lookup_id: IndexMap<Name, GraphId>,
    /// Full graph (all artifacts)
    pub full: DiGraphMap<GraphId, ()>,
}

/// #SPC-read-artifact.graph
/// Create the family graph from their given+auto partof values.
pub fn determine_graphs(partofs: &IndexMap<Name, IndexSet<Name>>) -> Graphs {
    let ids = create_ids(partofs);

    let mut graph_full: DiGraphMap<GraphId, ()> = DiGraphMap::new();
    for (name, partof) in partofs.iter() {
        let id = ids[name];
        graph_full.add_node(id);
        for p in partof.iter() {
            graph_full.add_edge(ids[p], id, ());
        }
    }

    let lookup_name = ids.iter().map(|(n, i)| (*i, n.clone())).collect();
    Graphs {
        lookup_id: ids,
        lookup_name: lookup_name,
        full: graph_full,
    }
}

/// Determine the `parts` of each artifact based on its neighbors in the graph.
pub fn determine_parts(graphs: &Graphs) -> IndexMap<Name, IndexSet<Name>> {
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

/// #SPC-read-artifact.completed
/// Determine the completeness of the artifacts.
pub fn determine_completed(
    graphs: &Graphs,
    impls: &IndexMap<Name, Impl>,
    subnames: &IndexMap<Name, IndexSet<SubName>>,
) -> IndexMap<Name, Completed> {
    // If there is a cycle we just return everything as 0% complete for spc+tst
    // We ignore `done` because there will be an ERROR lint later anyway.
    let uncomputed = || {
        impls
            .keys()
            .map(|n| (n.clone(), Completed::default()))
            .collect()
    };
    let sorted_graph = match petgraph::algo::toposort(&graphs.full, None) {
        Ok(s) => s,
        // cycle detected
        Err(_) => return uncomputed(),
    };

    // convert to by-id
    let impls: IndexMap<GraphId, &_> = impls
        .iter()
        .map(|(name, v)| (graphs.lookup_id[name], v))
        .collect();

    /// compute ratio but ignore count=0
    fn ratio(value: f64, count: usize) -> f64 {
        if count == 0 {
            0.0
        } else {
            value / count as f64
        }
    }

    let mut implemented: IndexMap<GraphId, f64> = IndexMap::with_capacity(impls.len());
    let mut tested: IndexMap<GraphId, f64> = IndexMap::with_capacity(impls.len());

    for id in sorted_graph.iter().rev() {
        let name = expect!(graphs.lookup_name.get(id));
        let sub = match subnames.get(name) {
            Some(s) => s,
            None => continue, // Will cause warning lint error.
        };
        let impl_ = expect!(impls.get(id));
        let (mut count_spc, mut value_spc, mut count_tst, mut value_tst) = impl_.to_statistics(sub);

        if matches!(graphs.lookup_name[id].ty, Type::TST) {
            for part_id in graphs.full.neighbors(*id) {
                value_spc += implemented[&part_id];
                count_spc += 1;
            }
            value_tst = value_spc;
            count_tst = count_spc;
        } else {
            for part_id in graphs.full.neighbors(*id) {
                value_tst += tested[&part_id];
                count_tst += 1;

                if !matches!(graphs.lookup_name[&part_id].ty, Type::TST) {
                    // TST's dont contribute towards spc in other types
                    value_spc += implemented[&part_id];
                    count_spc += 1;
                }
            }
        }
        tested.insert(*id, ratio(value_tst, count_tst));
        implemented.insert(*id, ratio(value_spc, count_spc));
    }

    debug_assert_eq!(impls.len(), implemented.len());
    debug_assert_eq!(impls.len(), tested.len());
    let out: IndexMap<Name, Completed> = implemented
        .iter()
        .map(|(id, spc)| {
            // throw away digits after 1000 significant digit
            // (note: only at end of all calculations!)
            let compl = Completed {
                spc: round_ratio(*spc),
                tst: round_ratio(tested[id]),
            };
            (graphs.lookup_name[id].clone(), compl)
        })
        .collect();
    debug_assert_eq!(impls.len(), out.len());
    out
}

pub fn round_ratio(ratio: f64) -> f32 {
    ((ratio * 1000.).round() / 1000.) as f32
}

// IMPLEMENTATION DETAILS

/// Create unique ids for the graph based on the names.
///
/// They are just guaranteed to be unique... not in any kind of order at all.
fn create_ids(names: &IndexMap<Name, IndexSet<Name>>) -> IndexMap<Name, GraphId> {
    let mut out: IndexMap<Name, GraphId> = IndexMap::new();
    let mut id = 1;

    for (name, partof) in names.iter() {
        out.insert(name.clone(), id);
        id += 1;
        for pof in partof.iter() {
            out.insert(pof.clone(), id);
            id += 1;
        }
    }
    out
}
