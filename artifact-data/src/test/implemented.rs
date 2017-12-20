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

use std::sync::mpsc::channel;
use rand;
use regex_generate;
use unicode_segmentation::UnicodeSegmentation;

use test::dev_prelude::*;
use test::raw_names::arb_names_raw;
use name::{self, Name, SubName, Type};
use raw_names::NamesRaw;
use path_abs::PathAbs;
use implemented::{join_locations, parse_locations, CodeLoc, ImplCode};
use lint;

// ------------------------------
// -- FUZZ METHODS

/// Generate a list of names along with randomly generated sub-names.
///
/// It is guaranteed that all names are represented.
pub fn random_impl_links<R: Rng + Clone>(
    rng: &mut R,
    names: &NamesRaw,
) -> HashSet<(Name, Option<SubName>)> {
    let mut textgen =
        regex_generate::Generator::parse(r"\.([a-zA-Z0-9_]{1,10})", rng.clone()).unwrap();
    let mut buffer: Vec<u8> = Vec::with_capacity(10);
    let mut out = HashSet::new();
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
    locations: &HashSet<(Name, Option<SubName>)>,
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
) -> BoxedStrategy<(NamesRaw, HashSet<(Name, Option<SubName>)>, String)> {
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

// SANITY

#[test]
fn sanity_parse_locations() {
    let example = r#"
This is some kind of text file.
There are links to code like below
%SPC-example %tst-example %TsT-ExAmPle

Some are not on the beginning of the line: %SPC-right
Some have a period after them like %SPC-period.
Multi: %SPC-one %SPC-two%SPC-three
Repeat: %SPC-repeat %SPC-repeat
REQ is valid: %REQ-valid
Also not valid: %REQ-.foo
Also not valid: %SPC--.foo
Also not valid: %SPC
Also not valid: %TST

Some are legitamate subnames:
%SPC-sub.name

And to the right:
    %SPC-right.sub
"#;
    let expected = &[
        (3, name!("SPC-example"), None),
        (3, name!("TST-example"), None),
        (3, name!("TST-example"), None),
        (5, name!("SPC-right"), None),
        (6, name!("SPC-period"), None),
        (7, name!("SPC-one"), None),
        (7, name!("SPC-two"), None),
        (7, name!("SPC-three"), None),
        (8, name!("SPC-repeat"), None),
        (8, name!("SPC-repeat"), None),
        (9, name!("REQ-valid"), None),
        (16, name!("SPC-sub"), Some(subname!(".name"))),
        (19, name!("SPC-right"), Some(subname!(".sub"))),
    ];
    let locations = parse_locations(replace_links(example).as_bytes()).unwrap();

    assert_eq!(expected, locations.as_slice());
}

#[test]
fn sanity_join_locations() {
    let (mut send_lints, lints) = channel();

    let file1 = PathAbs::fake("/fake/foo.py");
    let file2 = PathAbs::fake("/fake/bar.py");
    let file3 = PathAbs::fake("/fake/long/foo.py");

    let req_foo = name!("req-foo");
    let spc_bar = name!("spc-bar");
    let tst_baz = name!("tst-baz");

    let sub_a = subname!(".sub_a");
    let sub_b = subname!(".sub_b");

    let locations = &[
        (
            file1.clone(),
            vec![
                // 3 valid req-foo
                (1, req_foo.clone(), None),
                (2, req_foo.clone(), Some(sub_a.clone())),
                (3, req_foo.clone(), Some(sub_b.clone())),
                // immediate duplicate spc_bar
                (4, spc_bar.clone(), Some(sub_a.clone())),
                (5, spc_bar.clone(), Some(sub_a.clone())),
            ],
        ),
        (
            file2.clone(),
            vec![
                (1, tst_baz.clone(), None),
                (2, tst_baz.clone(), Some(sub_a.clone())),
            ],
        ),
        (
            file3.clone(),
            vec![
                // valid spc_bar
                (4, spc_bar.clone(), Some(sub_b.clone())),
                // additional duplicate spc_bar.a
                (5, spc_bar.clone(), Some(sub_a.clone())),
                // single duplicate of tst_baz
                (20, tst_baz.clone(), Some(sub_a.clone())),
            ],
        ),
    ];

    let mut expected = hashmap!{
        req_foo.clone() => ImplCode {
            primary: Some(CodeLoc::new(&file1, 1)),
            secondary: hashmap!{
                sub_a.clone() => CodeLoc::new(&file1, 2),
                sub_b.clone() => CodeLoc::new(&file1, 3),
            },
        },
        spc_bar.clone() => ImplCode {
            primary: None,
            secondary: hashmap!{
                sub_a.clone() => CodeLoc::new(&file3, 5),
                sub_b.clone() => CodeLoc::new(&file3, 4),
            },
        },
        tst_baz.clone() => ImplCode {
            primary: Some(CodeLoc::new(&file2, 1)),
            secondary: hashmap!{
                sub_a.clone() => CodeLoc::new(&file3, 20),
            },
        },
    };
    println!("getting joined");
    let mut joined = join_locations(&send_lints, locations);
    drop(send_lints);
    println!("got joined");

    let joined = BTreeMap::from_iter(joined.drain());
    let expected = BTreeMap::from_iter(expected.drain());
    assert_eq!(joined, expected);

    let lints: Vec<_> = lints.into_iter().collect();
    let create_lint = |path: &PathAbs, line, msg: &str| {
        lint::Lint {
            category: lint::Category::ParseCodeImplementations,
            path: Some(path.clone()),
            line: Some(line),
            msg: lint::Msg::Error(format!("duplicate detected: {}", msg)),
        }
    };

    let spc_bar_a_str = format!("{}{}", spc_bar.as_str(), sub_a.as_str());
    let tst_baz_a_str = format!("{}{}", tst_baz.as_str(), sub_a.as_str());
    let expected_lints = vec![
        create_lint(&file1, 4, &spc_bar_a_str),
        create_lint(&file1, 5, &spc_bar_a_str),

        // second file re-prints the last lint
        create_lint(&file1, 5, &spc_bar_a_str),
        create_lint(&file3, 5, &spc_bar_a_str),

        create_lint(&file2, 2, &tst_baz_a_str),
        create_lint(&file3, 20, &tst_baz_a_str),
    ];
    assert_eq!(lints, expected_lints);
}

// FUZZ TESTS

proptest! {
    #[test]
    fn fuzz_locations((ref _names, ref expected_locations, ref code_text) in arb_source_code(10)) {
        println!("## Code Text:\n{}", code_text);
        let locations = {
            let mut l: Vec<_> = parse_locations(code_text.as_bytes())
                .expect("parse");
            let mut l: Vec<_> = l.drain(0..)
                // drop the column
                .map(|(_, n, s)| (n, s))
                .collect();
            l.sort();
            l
        };
        let expected = {
            let mut l = Vec::from_iter(expected_locations.iter().cloned());
            l.sort();
            l
        };
        assert_eq!(locations, expected);
    }
}
