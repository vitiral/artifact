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

//! Test serializing/deserializing raw names

use ergo::{json, toml, yaml};

use super::dev_prelude::*;
use super::name::arb_name;
use artifact_data::raw_names::NamesRaw;

pub fn arb_names_raw(size: usize) -> BoxedStrategy<NamesRaw> {
    prop::collection::hash_set(arb_name(), 0..size)
        .prop_map(|hs| NamesRaw::from(hs))
        .boxed()
}
