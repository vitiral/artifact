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
use path_abs::PathFile;
use graph::{self, round_ratio};
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
        name!("TST-aaa") => orderset!{name!("SPC-bbb")},
        name!("TST-aaa-a") => orderset!{name!("TST-aaa")},
    };

    let graphs = graph::determine_graphs(&partofs);
    (partofs, graphs)
}

#[test]
fn sanity_determine_parts() {
    let (_, graphs) = simple_graph();
    let mut parts = graph::determine_parts(&graphs);
    let mut expected = ordermap!{
        name!("REQ-aaa") => orderset!{name!("REQ-bbb")},
        name!("REQ-bbb") => orderset!{name!("REQ-ccc"), name!("SPC-bbb")},
        name!("REQ-ccc") => orderset!{},
        name!("SPC-bbb") => orderset!{name!("SPC-bbb-a"), name!("SPC-bbb-b"), name!("TST-aaa")},
        name!("SPC-bbb-a") => orderset!{},
        name!("SPC-bbb-b") => orderset!{},
        name!("TST-aaa") => orderset!{name!("TST-aaa-a")},
        name!("TST-aaa-a") => orderset!{},
    };

    parts.sort_keys();
    expected.sort_keys();
    assert_eq!(parts, expected);
}

#[test]
fn sanity_determine_graphs() {
    let partofs = ordermap!{
        name!("REQ-a") => orderset!{},
        name!("TST-a") => orderset!{},
    };
    let graphs = graph::determine_graphs(&partofs);
    assert_eq!(graphs.full.node_count(), 2);
    assert_eq!(graphs.full.edge_count(), 0);
    assert_eq!(graphs.lookup_id.len(), 2);
}

#[test]
fn sanity_determine_completed() {
    let (_, graphs) = simple_graph();

    let loc = CodeLoc::new(&PathFile::mock("/fake"), 1);
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
        name!("TST-aaa") => Impl::NotImpl,
        name!("TST-aaa-a") => Impl::Code(ImplCode {
            primary: Some(loc.clone()),
            secondary: ordermap!{},
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
        name!("TST-aaa") => orderset!{},
        name!("TST-aaa-a") => orderset!{
            subname!(".notdone"),
        },
    };

    type C = graph::Completed;
    let mut completed = graph::determine_completed(&graphs, &impls, &subnames);
    let spc_bbb_b = 1.0_f64 / 4.0_f64;
    let spc_bbb = (3.0_f64 + spc_bbb_b + 0.0_f64) / (4.0_f64 + 2.0_f64);
    let req_bbb = (1.0_f64 + spc_bbb) / 4.0_f64; // one subname so self-count == 2

    // test-ratios
    let tr_tst_aaa_a = 0.5;
    let tr_spc_bbb = tr_tst_aaa_a / 3.;
    let tr_req_bbb = (tr_spc_bbb  + 1. /*req-ccc*/) / 2.;

    // FIXME: remove adjustmeents -- why the heck are they needed???
    let mut expected = ordermap!{
        name!("REQ-aaa") => C {tst: round_ratio(tr_req_bbb), spc: round_ratio(req_bbb)},
        name!("REQ-bbb") => C {tst: round_ratio(tr_req_bbb), spc: round_ratio(req_bbb)},
        name!("REQ-ccc") => C {tst: 1.0, spc: 1.},
        name!("SPC-bbb") => C {tst: round_ratio(tr_spc_bbb), spc: round_ratio(spc_bbb)},
        name!("SPC-bbb-a") => C {tst: 0.0, spc: 0.},
        name!("SPC-bbb-b") => C {tst: 0.0, spc: round_ratio(spc_bbb_b)},
        name!("TST-aaa") => C {tst: round_ratio(tr_tst_aaa_a), spc: round_ratio(tr_tst_aaa_a)},
        name!("TST-aaa-a") => C {tst: round_ratio(tr_tst_aaa_a), spc: round_ratio(tr_tst_aaa_a)},
    };
    completed.sort_keys();
    expected.sort_keys();
    assert_eq!(expected, completed);
}
