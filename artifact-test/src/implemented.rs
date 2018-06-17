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
/// Test the "implemented" (i.e. source code parsing) module.

use regex_generate;

use super::dev_prelude::*;
use super::raw_names::arb_names_raw;
use artifact_data::raw_names::NamesRaw;
use artifact_data::implemented::{join_locations, parse_locations};

// ------------------------------
// -- FUZZ METHODS

/// Generate a list of names along with randomly generated sub-names.
///
/// It is guaranteed that all names are represented.
pub fn random_impl_links<R: Rng + Clone>(
    rng: &mut R,
    names: &NamesRaw,
) -> IndexSet<(Name, Option<SubName>)> {
    let mut textgen =
        regex_generate::Generator::parse(r"\.([a-zA-Z0-9_]{1,10})", rng.clone()).unwrap();
    let mut buffer: Vec<u8> = Vec::with_capacity(10);
    let mut out = IndexSet::new();
    for name in names.iter() {
        // Base name is always included
        out.insert((name.clone(), None));
        if rng.next_f32() < 0.3 {
            // 30% chance ahere are no subnames
            continue;
        }
        for _ in 0..rng.gen_range(1, 5) {
            buffer.clear();
            textgen.generate(&mut buffer).unwrap();
            let sub = subname!(str::from_utf8(&buffer).unwrap());
            out.insert((name.clone(), Some(sub)));
        }
    }
    out
}

/// Generate random source code text with links to all the given `name[.sub]`s.
pub fn random_source_code<R: Rng + Clone>(
    rng: &mut R,
    locations: &IndexSet<(Name, Option<SubName>)>,
) -> String {
    let mut lines = random_lines(rng);
    if lines.is_empty() {
        lines.push(vec!["".into()]);
    }
    for &(ref name, ref sub) in locations.iter() {
        insert_word(rng, &mut lines, format!("#{}", name_ref_string(name, sub)));
    }
    lines.iter().map(|line| line.iter().join(" ")).join("\n")
}

/// Arbitrary single source code file
pub fn arb_source_code(
    size: usize,
) -> BoxedStrategy<(NamesRaw, IndexSet<(Name, Option<SubName>)>, String)> {
    arb_names_raw(size)
        .prop_perturb(|names, mut rng| {
            let locations = random_impl_links(&mut rng, &names);
            let code = random_source_code(&mut rng, &locations);
            (names, locations, code)
        })
        .boxed()
}

// METHODS

pub fn replace_links(raw: &str) -> String {
    raw.replace('%', "#")
}
