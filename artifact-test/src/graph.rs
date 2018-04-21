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
