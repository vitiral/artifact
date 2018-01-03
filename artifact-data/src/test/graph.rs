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

use test::dev_prelude::*;
use name::{Name, SubName};
use graph;

#[test]
fn sanity_parts() {
    let partofs = ordermap!{
        name!("REQ-top") => orderset!{},
        name!("REQ-mid") => orderset!{name!("REQ-top")},
        name!("REQ-bot") => orderset!{name!("REQ-mid")},
        name!("SPC-mid") => orderset!{name!("REQ-mid")},
        name!("SPC-mid-a") => orderset!{name!("SPC-mid")},
    };

    let graphs = graph::determine_graphs(&partofs);
    let mut parts = graph::determine_parts(&graphs);
    let mut expected = ordermap!{
        name!("REQ-top") => orderset!{name!("REQ-mid")},
        name!("REQ-mid") => orderset!{name!("REQ-bot"), name!("SPC-mid")},
        name!("REQ-bot") => orderset!{},
        name!("SPC-mid") => orderset!{name!("SPC-mid-a")},
        name!("SPC-mid-a") => orderset!{},
    };

    sort_ordermap(&mut parts);
    sort_ordermap(&mut expected);
    assert_eq!(parts, expected);
}
