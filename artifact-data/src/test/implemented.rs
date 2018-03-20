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
/// #TST-read-impl
/// Test the "implemented" (i.e. source code parsing) module.

use regex_generate;

use test::dev_prelude::*;
use test::raw_names::arb_names_raw;
use test::framework::run_interop_tests;
use raw_names::NamesRaw;
use implemented::{join_locations, parse_locations};

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

// SANITY

#[test]
/// #TST-read-impl.parse
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
    let file = PathFile::mock("/fake/file.c");
    let mut expected: Vec<_> = vec![
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
    let expected: Vec<_> = expected
        .drain(0..)
        .map(|(line, name, sub)| (CodeLoc::new(&file, line), name, sub))
        .collect();
    let (send, locations) = ch::unbounded();
    parse_locations(&send, &file, replace_links(example).as_bytes()).unwrap();
    drop(send);
    let locations: Vec<_> = locations.into_iter().collect();

    assert_eq!(expected, locations);
}

#[test]
/// #TST-read-impl.join
fn sanity_join_locations() {
    let (send_lints, lints) = ch::unbounded();

    let file1 = PathFile::mock("/fake/foo.py");
    let file2 = PathFile::mock("/fake/bar.py");
    let file3 = PathFile::mock("/fake/long/foo.py");

    let req_foo = name!("req-foo");
    let spc_bar = name!("spc-bar");
    let tst_baz = name!("tst-baz");

    let sub_a = subname!(".sub_a");
    let sub_b = subname!(".sub_b");

    let locations = vec![
        // -- file1
        // 3 valid req-foo
        (CodeLoc::new(&file1, 1), req_foo.clone(), None),
        (
            CodeLoc::new(&file1, 2),
            req_foo.clone(),
            Some(sub_a.clone()),
        ),
        (
            CodeLoc::new(&file1, 3),
            req_foo.clone(),
            Some(sub_b.clone()),
        ),
        // immediate duplicate spc_bar
        (
            CodeLoc::new(&file1, 4),
            spc_bar.clone(),
            Some(sub_a.clone()),
        ),
        (
            CodeLoc::new(&file1, 5),
            spc_bar.clone(),
            Some(sub_a.clone()),
        ),
        // --file2
        (CodeLoc::new(&file2, 1), tst_baz.clone(), None),
        (
            CodeLoc::new(&file2, 2),
            tst_baz.clone(),
            Some(sub_a.clone()),
        ),
        // --file3
        (
            CodeLoc::new(&file3, 4),
            spc_bar.clone(),
            Some(sub_b.clone()),
        ),
        // additional duplicate spc_bar.a
        (
            CodeLoc::new(&file3, 5),
            spc_bar.clone(),
            Some(sub_a.clone()),
        ),
        // single duplicate of tst_baz
        (CodeLoc::new(&file3, 20), tst_baz.clone(), None),
    ];

    let expected = indexmap!{
        req_foo.clone() => ImplCode {
            primary: Some(CodeLoc::new(&file1, 1)),
            secondary: indexmap!{
                sub_a.clone() => CodeLoc::new(&file1, 2),
                sub_b.clone() => CodeLoc::new(&file1, 3),
            },
        },
        spc_bar.clone() => ImplCode {
            primary: None,
            secondary: indexmap!{
                sub_a.clone() => CodeLoc::new(&file3, 5),
                sub_b.clone() => CodeLoc::new(&file3, 4),
            },
        },
        tst_baz.clone() => ImplCode {
            primary: Some(CodeLoc::new(&file3, 20)),
            secondary: indexmap!{
                sub_a.clone() => CodeLoc::new(&file2, 2),
            },
        },
    };
    println!("getting joined");
    let joined = join_locations(&send_lints, locations);
    drop(send_lints);
    println!("got joined");

    assert_eq!(joined, expected);

    let lints: Vec<_> = lints.into_iter().collect();
    let create_lint = |path: &PathFile, line, msg: &str| lint::Lint {
        level: lint::Level::Error,
        category: lint::Category::ParseCodeImplementations,
        path: Some(path.clone().into()),
        line: Some(line),
        msg: format!("duplicate detected: {}", msg),
    };

    let spc_bar_a_str = format!("{}{}", spc_bar.as_str(), sub_a.as_str());
    let expected_lints = vec![
        create_lint(&file1, 4, &spc_bar_a_str),
        create_lint(&file1, 5, &spc_bar_a_str),
        // second file re-prints the last lint
        create_lint(&file1, 5, &spc_bar_a_str),
        create_lint(&file3, 5, &spc_bar_a_str),
        create_lint(&file2, 1, tst_baz.as_str()),
        create_lint(&file3, 20, tst_baz.as_str()),
    ];
    assert_eq!(lints, expected_lints);
}

// FUZZ TESTS

proptest! {
    #[test]
    /// #TST-read-impl.parse_fuzz
    fn fuzz_locations((ref _names, ref expected_locations, ref code_text) in arb_source_code(10)) {
        println!("## Code Text:\n{}", code_text);
        let file = PathFile::mock("/fake");
        let locations = {
            let (send, locations) = ch::unbounded();
            parse_locations(&send, &file, code_text.as_bytes())
                .expect("parse");
            drop(send);
            let mut l: Vec<_> = locations.into_iter()
                // drop the loc
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

// INTEROP TESTS

#[test]
fn interop_source_only() {
    run_interop_tests(INTEROP_TESTS_PATH.join("source_only"));
}

#[test]
fn interop_source_invalid() {
    run_interop_tests(INTEROP_TESTS_PATH.join("source_invalid"));
}
