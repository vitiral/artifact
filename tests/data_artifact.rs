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
//! Unit/Fuzz Tests:
//! - #TST-unit.artifact
//! - #TST-fuzz.artifact
extern crate artifact_data;
extern crate artifact_test;
extern crate ergo;

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

    let arts = indexmap!{
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
    let mut expected = indexmap!{
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
