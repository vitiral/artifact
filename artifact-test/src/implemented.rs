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
/// Test the "implemented" (i.e. source code parsing) module.
use regex_generate;

use super::dev_prelude::*;
use super::raw_names::arb_names_raw;
use artifact_data::implemented::{join_locations, parse_locations};
use artifact_data::raw_names::NamesRaw;

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
        regex_generate::Generator::parse(r"\.((tst-)?[a-zA-Z0-9_]{1,10})", rng.clone()).unwrap();
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
