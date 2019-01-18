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
//! Unit/Fuzz Tests:
//! - #TST-unit.artifact
//! - #TST-fuzz.artifact

use artifact_data::artifact;
use artifact_test::dev_prelude::*;

#[test]
fn sanity_determine_partofs() {
    fn with_partof(mut partof: Vec<Name>) -> ArtifactIm {
        partof.sort();
        ArtifactIm {
            name: name!("TST-fake"),
            file: PathArc::new("/fake"),
            partof: partof.drain(..).collect(),
            done: None,
            text: "".into(),
        }
    }

    let arts = indexmap! {
        name!("REQ-aaa") => with_partof(vec![]),
        // test auto-parent
        name!("REQ-aaa-a") => with_partof(vec![]),
        // test auto-partof (no parent)
        name!("SPC-aaa-a") => with_partof(vec![]),
        name!("SPC-bbb") => with_partof(vec![]),
        // test explcit-link + parent
        name!("SPC-bbb-p") => with_partof(vec![name!("REQ-aaa")]),
        // test explcit-link only
        name!("SPC-ccc") => with_partof(vec![name!("REQ-aaa")]),
    };

    let mut partofs = artifact::determine_partofs(&arts);
    let mut expected = indexmap! {
        name!("REQ-aaa") => indexset![],
        name!("REQ-aaa-a") => indexset![name!("REQ-aaa")],
        name!("SPC-aaa-a") => indexset![name!("REQ-aaa-a")],
        name!("SPC-bbb") => indexset![],
        name!("SPC-bbb-p") => indexset![name!("SPC-bbb"), name!("REQ-aaa")],
        name!("SPC-ccc") => indexset![name!("REQ-aaa")],
    };
    partofs.sort_keys();
    expected.sort_keys();
    assert_eq!(expected, partofs);
}
