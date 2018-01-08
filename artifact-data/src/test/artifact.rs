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
//! #TST-data-artifact
//!
//! This module defines tests for the "full" artifact type itself.

use test::dev_prelude::*;
use test::framework::run_interop_test;
use name::Name;
use raw::ArtifactRaw;
use raw_names::NamesRaw;
use artifact;

#[test]
/// #TST-data-artifact.partofs
fn sanity_determine_partofs() {
    fn with_partof(partof: Option<OrderSet<Name>>) -> ArtifactRaw {
        ArtifactRaw {
            done: None,
            partof: partof.map(|p| NamesRaw { inner: p }),
            text: None,
        }
    }

    let raw_artifacts = ordermap!{
        name!("REQ-aaa") => with_partof(None),
        // test auto-parent
        name!("REQ-aaa-a") => with_partof(None),
        // test auto-partof (no parent)
        name!("SPC-aaa-a") => with_partof(None),
        name!("SPC-bbb") => with_partof(None),
        // test explcit-link + parent
        name!("SPC-bbb-p") => with_partof(Some(orderset![name!("REQ-aaa")])),
        // test explcit-link only
        name!("SPC-ccc") => with_partof(Some(orderset![name!("REQ-aaa")])),
    };

    let mut partofs = artifact::determine_partofs(&raw_artifacts);
    let mut expected = ordermap!{
        name!("REQ-aaa") => orderset![],
        name!("REQ-aaa-a") => orderset![name!("REQ-aaa")],
        name!("SPC-aaa-a") => orderset![name!("REQ-aaa-a")],
        name!("SPC-bbb") => orderset![],
        name!("SPC-bbb-p") => orderset![name!("SPC-bbb"), name!("REQ-aaa")],
        name!("SPC-ccc") => orderset![name!("REQ-aaa")],
    };
    sort_ordermap(&mut partofs);
    sort_ordermap(&mut expected);
    assert_eq!(expected, partofs);
}

// INTEROP TESTS

#[test]
/// #TST-data-artifact.empty
fn interop_project_empty() {
    run_interop_test(INTEROP_TESTS_PATH.join("empty"));
}

#[test]
/// #TST-data-artifact.design_only
fn interop_design_only() {
    run_interop_test(INTEROP_TESTS_PATH.join("design_only"));
}

#[test]
/// #TST-data-artifact.basic
fn interop_basic() {
    run_interop_test(INTEROP_TESTS_PATH.join("basic"));
}

#[test]
/// #TST-data-artifact.basic
fn interop_lints() {
    run_interop_test(INTEROP_TESTS_PATH.join("lints"));
}
