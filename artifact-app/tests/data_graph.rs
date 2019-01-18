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
extern crate artifact_test;
use artifact_test::artifact_data::graph::{self, round_ratio};
use artifact_test::graph::*;
use artifact_test::*;

#[test]
fn test_completed_fmt() {
    fn assert(spc: f32, tst: f32, result: &str) {
        let c = Completed { spc: spc, tst: tst };
        assert_eq!(&c.to_string(), result);
    }

    // max of 2 decimals
    assert(1.0, 0.334323, "spc=1.00, tst=0.33");
    // exactly 2 decimals
    assert(0.0, 0.2, "spc=0.00, tst=0.20");
    // rounding works as expected
    assert(0.127, 0.231, "spc=0.13, tst=0.23");
}

#[test]
fn sanity_determine_parts() {
    let (_, graphs) = simple_graph();
    let mut parts = graph::determine_parts(&graphs);
    let mut expected = indexmap! {
        name!("REQ-aaa") => indexset!{name!("REQ-bbb")},
        name!("REQ-bbb") => indexset!{name!("REQ-ccc"), name!("SPC-bbb")},
        name!("REQ-ccc") => indexset!{},
        name!("SPC-bbb") => indexset!{name!("SPC-bbb-a"), name!("SPC-bbb-b"), name!("TST-aaa")},
        name!("SPC-bbb-a") => indexset!{},
        name!("SPC-bbb-b") => indexset!{},
        name!("TST-aaa") => indexset!{name!("TST-aaa-a")},
        name!("TST-aaa-a") => indexset!{},
    };

    parts.sort_keys();
    expected.sort_keys();
    assert_eq!(parts, expected);
}

#[test]
fn sanity_determine_graphs() {
    let partofs = indexmap! {
        name!("REQ-a") => indexset!{},
        name!("TST-a") => indexset!{},
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
    let impls = indexmap! {
        name!("REQ-aaa") => Impl::NotImpl,
        name!("REQ-bbb") => Impl::NotImpl,
        name!("REQ-ccc") => Impl::Done("foo".into()),
        name!("SPC-bbb") => Impl::Code(ImplCode {
            primary: Some(loc.clone()),
            secondary: indexmap!{
                subname!(".done1") => loc.clone(),
                subname!(".done2") => loc.clone(),
            },
        }),
        name!("SPC-bbb-a") => Impl::NotImpl,
        name!("SPC-bbb-b") => Impl::Code(ImplCode {
            primary: None,
            secondary: indexmap!{
                subname!(".done") => loc.clone(),
            },
        }),
        name!("TST-aaa") => Impl::NotImpl,
        name!("TST-aaa-a") => Impl::Code(ImplCode {
            primary: Some(loc.clone()),
            secondary: indexmap!{},
        }),
    };
    let subnames = indexmap! {
        name!("REQ-aaa") => indexset!{},
        name!("REQ-bbb") => indexset!{subname!(".notdone")},
        name!("REQ-ccc") => indexset!{},
        name!("SPC-bbb") => indexset!{
            subname!(".done1"),
            subname!(".done2"),
            subname!(".notdone"),
        },
        name!("SPC-bbb-a") => indexset!{},
        name!("SPC-bbb-b") => indexset!{
            subname!(".done"),
            subname!(".notdone1"),
            subname!(".notdone2"),
        },
        name!("TST-aaa") => indexset!{},
        name!("TST-aaa-a") => indexset!{
            subname!(".notdone"),
        },
    };

    type C = Completed;
    let mut completed = graph::determine_completed(&graphs, &impls, &subnames);
    let spc_bbb_b = 1.0_f64 / 4.0_f64;
    let spc_bbb = (3.0_f64 + spc_bbb_b + 0.0_f64) / (4.0_f64 + 2.0_f64);
    let req_bbb = (1.0_f64 + spc_bbb) / 4.0_f64; // one subname so self-count == 2

    // test-ratios
    let tr_tst_aaa_a = 0.5;
    let tr_spc_bbb = tr_tst_aaa_a / 3.;
    let tr_req_bbb = (tr_spc_bbb + 1./*req-ccc*/) / 2.;

    let mut expected = indexmap! {
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
