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
use path_abs::PathAbs;
use graph;
use implemented::{CodeLoc, Impl, ImplCode};

/// create the `partof`s and the graphs
fn simple_graph() -> (OrderMap<Name, OrderSet<Name>>, graph::Graphs) {
    let partofs = ordermap!{
        name!("REQ-aaa") => orderset!{},
        name!("REQ-bbb") => orderset!{name!("REQ-aaa")},
        name!("REQ-ccc") => orderset!{name!("REQ-bbb")},
        name!("SPC-bbb") => orderset!{name!("REQ-bbb")},
        name!("SPC-bbb-a") => orderset!{name!("SPC-bbb")},
        name!("SPC-bbb-b") => orderset!{name!("SPC-bbb")},
    };

    let graphs = graph::determine_graphs(&partofs);
    (partofs, graphs)
}

#[test]
fn sanity_parts() {
    let (_, graphs) = simple_graph();
    let mut parts = graph::determine_parts(&graphs);
    let mut expected = ordermap!{
        name!("REQ-aaa") => orderset!{name!("REQ-bbb")},
        name!("REQ-bbb") => orderset!{name!("REQ-ccc"), name!("SPC-bbb")},
        name!("REQ-ccc") => orderset!{},
        name!("SPC-bbb") => orderset!{name!("SPC-bbb-a"), name!("SPC-bbb-b")},
        name!("SPC-bbb-a") => orderset!{},
        name!("SPC-bbb-b") => orderset!{},
    };

    sort_ordermap(&mut parts);
    sort_ordermap(&mut expected);
    assert_eq!(parts, expected);
}

#[test]
fn sanity_completed() {
    let (_, graphs) = simple_graph();

    let loc = CodeLoc::new(&PathAbs::fake("/fake"), 1);
    let impls = ordermap!{
        name!("REQ-aaa") => Impl::NotImpl,
        name!("REQ-bbb") => Impl::NotImpl,
        name!("REQ-ccc") => Impl::Done("foo".into()),
        name!("SPC-bbb") => Impl::Code(ImplCode {
            primary: Some(loc.clone()),
            secondary: ordermap!{
                subname!(".done1") => loc.clone(),
                subname!(".done2") => loc.clone(),
            },
        }),
        name!("SPC-bbb-a") => Impl::NotImpl,
        name!("SPC-bbb-b") => Impl::Code(ImplCode {
            primary: None,
            secondary: ordermap!{
                subname!(".done") => loc.clone(),
            },
        }),
    };
    let subnames = ordermap!{
        name!("REQ-aaa") => orderset!{},
        name!("REQ-bbb") => orderset!{subname!(".notdone")},
        name!("REQ-ccc") => orderset!{},
        name!("SPC-bbb") => orderset!{
            subname!(".done1"),
            subname!(".done2"),
            subname!(".notdone"),
        },
        name!("SPC-bbb-a") => orderset!{},
        name!("SPC-bbb-b") => orderset!{
            subname!(".done"),
            subname!(".notdone1"),
            subname!(".notdone2"),
        },
    };

    type C = graph::Completed;
    let mut completed = graph::determine_completed(&graphs, &impls, &subnames);
    let spc_bbb_b = 1. / 4.;
    let spc_bbb = (1. + 2. + spc_bbb_b) / (4. + 2.);
    let req_bbb = (1. + spc_bbb) / 4.; // one subname so self-count == 2
    let mut expected = ordermap!{
        name!("REQ-aaa") => C {tst: 0.5, spc: req_bbb},
        name!("REQ-bbb") => C {tst: 0.5, spc: req_bbb},
        name!("REQ-ccc") => C {tst: 1.0, spc: 1.},
        name!("SPC-bbb") => C {tst: 0.0, spc: spc_bbb},
        name!("SPC-bbb-a") => C {tst: 0.0, spc: 0.},
        name!("SPC-bbb-b") => C {tst: 0.0, spc: spc_bbb_b},
    };
    sort_ordermap(&mut completed);
    sort_ordermap(&mut expected);
    assert_eq!(expected, completed);
}
