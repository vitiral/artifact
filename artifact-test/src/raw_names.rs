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

//! Test serializing/deserializing raw names

use ergo::{json, toml, yaml};

use artifact_data::raw_names::NamesRaw;
use super::dev_prelude::*;
use super::name::arb_name;

pub fn arb_names_raw(size: usize) -> BoxedStrategy<NamesRaw> {
    prop::collection::hash_set(arb_name(), 0..size)
        .prop_map(|hs| NamesRaw::from(hs))
        .boxed()
}
