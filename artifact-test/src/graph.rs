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
//! This module defines tests for the graph implementation details of
//! computing artifacts.

use super::dev_prelude::*;
use artifact_data::graph::{self, round_ratio};

/// create the `partof`s and the graphs
pub fn simple_graph() -> (IndexMap<Name, IndexSet<Name>>, graph::Graphs) {
    let partofs = indexmap!{
        name!("REQ-aaa") => indexset!{},
        name!("REQ-bbb") => indexset!{name!("REQ-aaa")},
        name!("REQ-ccc") => indexset!{name!("REQ-bbb")},
        name!("SPC-bbb") => indexset!{name!("REQ-bbb")},
        name!("SPC-bbb-a") => indexset!{name!("SPC-bbb")},
        name!("SPC-bbb-b") => indexset!{name!("SPC-bbb")},
        name!("TST-aaa") => indexset!{name!("SPC-bbb")},
        name!("TST-aaa-a") => indexset!{name!("TST-aaa")},
    };

    let graphs = graph::determine_graphs(&partofs);
    (partofs, graphs)
}
