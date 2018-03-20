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
//! #TST-read-family
//! These are tests and helpers for testing family relations,
//! such as parts/partof.

use ergo::json;

use raw_names::NamesRaw;
use test::dev_prelude::*;
use test::name::arb_name;
use test::raw_names::arb_names_raw;

// ------------------------------
// -- FUZZ METHODS

pub fn arb_names(size: usize) -> BoxedStrategy<Names> {
    prop::collection::hash_set(arb_name(), 0..size)
        .prop_map(|hs| Names::from(hs))
        .boxed()
}

/// Split names up into their types.
///
/// Returns vectors of (REQ, SPC, TST) names grouped by sorted-family
fn split_names(names: &NamesRaw) -> (Vec<Vec<Name>>, Vec<Vec<Name>>, Vec<Vec<Name>>) {
    let mut req = IndexSet::new();
    let mut spc = IndexSet::new();
    let mut tst = IndexSet::new();

    for name in names.iter().cloned() {
        match name.ty {
            Type::REQ => req.insert(name),
            Type::SPC => spc.insert(name),
            Type::TST => tst.insert(name),
        };
    }

    (group_family(req), group_family(spc), group_family(tst))
}

/// group names by their family (if they have any)
fn group_family(names: IndexSet<Name>) -> Vec<Vec<Name>> {
    let mut families = Vec::new();
    let mut remaining: IndexSet<Name> = IndexSet::from_iter(names.iter().cloned());

    for name in names.iter() {
        if !remaining.contains(name) {
            // already part of a family
            continue;
        }
        remaining.remove(name);
        let mut fam = vec![name.clone()];
        // TODO: well, this doesn't actually work... If I find a match
        // I need to scan for THAT match, etc etc until there are no more
        // matches... probably needs to be recursive.
        for other in remaining.iter() {
            if matches!(name.parent(), Some(_)) || matches!(other.parent(), Some(_)) {
                fam.push(other.clone());
            }
        }
        for n in fam.iter() {
            // It's okay that we remove `name` twice
            remaining.remove(n);
        }
        // sort in reverse order, children first
        fam.sort_unstable_by(|a, b| b.cmp(a));
        families.push(fam);
    }
    families
}

/// flatten families and simultaniously randomize them smartly
fn flatten_families<R: Rng>(rng: &mut R, mut families: Vec<Vec<Name>>) -> Vec<Name> {
    let mut out: Vec<Name> = Vec::new();
    for mut fam in families.drain(0..) {
        fam.reverse();
        // Each family contains sorted names relative to that family.
        // We can NOT change the sorted order relative to their siblings,
        // but we CAN change the order relative to other names.
        let mut last_parent_index = rng.gen_range(0, out.len() + 1);
        while !fam.is_empty() {
            // always select the "highest parent" remaining
            let parent = fam.pop().unwrap();
            out.insert(last_parent_index, parent);
            last_parent_index = rng.gen_range(
                // insert after the last parent
                last_parent_index + 1,
                // up to the full length
                out.len() + 1,
            );
        }
    }
    out
}

/// Return arbitrary names topologically sorted.
///
/// By topologically sorting it, we know that any node at `n` can be a `partof` any node with an
/// index less than `n`. This allows us to construct the graph randomly.
///
/// This takes into account that:
/// - children of parents **must** be to the "left" of their parents
/// - REQ must be to the left of SPC which must be to the left of TST
pub fn arb_topologically_sorted_names(size: usize) -> BoxedStrategy<(NamesRaw, Vec<Name>)> {
    arb_names_raw(size)
        // randomize the order
        .prop_perturb(|names, mut rng| {
            // split up names into types/families
            let (mut req, mut spc, mut tst) = split_names(&names);
            rng.shuffle(&mut req);
            rng.shuffle(&mut spc);
            rng.shuffle(&mut tst);

            // flatten and randomize a bit
            let mut req = flatten_families(&mut rng, req);
            let mut spc = flatten_families(&mut rng, spc);
            let mut tst = flatten_families(&mut rng, tst);

            req.extend(spc.drain(0..));
            req.extend(tst.drain(0..));
            (names, req)
        })
        // congrats, we now have a valid topologically sorted graph of names
        // we can select names to select from to create the partof elements
        .boxed()
}

/// Randomly select "valid" partof elements for the name at the index.
///
/// To be valid, this assumes that `names` is topographically sorted.
pub fn rand_select_partof<R: Rng>(rng: &mut R, index: usize, names: &[Name]) -> Vec<Name> {
    let amount = rng.gen_range(0, index + 1);
    rand::seq::sample_slice(rng, &names[0..index], amount)
        .iter()
        .map(|n| (*n).clone())
        .collect()
}

// ------------------------------
// -- METHODS AND HELPERS

/// assert (raw, expected_collapsed, expected).
///
/// If expected_collapsed is not given, raw=expected_collapsed
///
/// This does two steps:
/// - asserts that the result of expanding is the expected
/// - asserts that re-collapsing results in the original raw
fn assert_collapsed_valid(values: &[(&str, Option<&str>, IndexSet<&str>)]) {
    let errors = values
        .iter()
        .map(|&(r, e_col, ref e)| (r, e_col, e, expand_names(r)))
        .filter_map(|(raw, expect_col, expect, result)| match result {
            Ok(result) => {
                let result_raw = result
                    .iter()
                    .map(|n| n.raw.as_str())
                    .collect::<IndexSet<_>>();
                let mut difference = result_raw.difference(&expect).collect::<Vec<_>>();
                if !difference.is_empty() {
                    difference.sort();
                    return Some(format!(
                        "### {} DIFFERENT FROM EXPECTED: {:?}",
                        raw, difference,
                    ));
                }
                let result_collapse = collapse_names(&result);
                let expect_col = match expect_col {
                    Some(e) => e,
                    None => raw,
                };
                if expect_col != result_collapse {
                    return Some(format!(
                        "### Collapsed differ: {} != {}",
                        result_collapse, raw
                    ));
                }
                None
            }
            Err(_) => Some(format!("invalid raw: {}", raw)),
        })
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("Some collapsed names not valid:\n{:#?}", errors);
    }
}

fn assert_collapsed_invalid(raw: &[&str]) {
    let errors = raw.iter()
        .filter_map(|r| {
            if let Ok(_) = expand_names(r) {
                Some(r)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!(
            "Some collapsed names were valid when they shouldn't be:\n{:#?}",
            errors
        );
    }
}

/// take a list of names and collapse them into a single
/// string with format `REQ-foo-[bar, baz-boo], SPC-foo`
pub fn collapse_names(names: &IndexSet<Name>) -> String {
    let raw_pieces: Vec<Vec<String>> = {
        let mut pieces: Vec<Vec<String>> = names
            .iter()
            .map(|n| n.raw.split('-').map(|s| s.to_string()).collect())
            .collect();
        pieces.sort();
        pieces
    };

    let mut piece = NamePiece {
        raw: raw_pieces,
        prefix: String::new(),
        pieces: None,
    };
    piece.process();

    let mut collapsed = String::new();
    let is_last = match piece.pieces {
        None => true,
        Some(ref pieces) => pieces.len() > 1,
    };
    piece.collapse(&mut collapsed, is_last);
    collapsed
}

struct NamePiece {
    raw: Vec<Vec<String>>,
    prefix: String,
    pieces: Option<Vec<NamePiece>>,
}

impl NamePiece {
    /// note: raw must be sorted
    fn from(prefix: String, raw: Vec<Vec<String>>) -> NamePiece {
        NamePiece {
            raw: raw,
            prefix: prefix,
            pieces: None,
        }
    }

    /// recursively process the NamePiece until all pieces are just their prefix
    /// this works because:
    /// - we know raw is sorted, so we know all single item prefixes will appear
    ///     one after the other
    /// - from there we just need to go down the tree until all of the lowest
    ///     level pieces have only a prefix
    fn process(&mut self) {
        let mut prefix = "".to_string();
        let mut pieces: Vec<NamePiece> = vec![];
        for part in &self.raw {
            if part.len() == 1 {
                // it is already it's own piece
                pieces.push(NamePiece::from(part[0].clone(), vec![]));
                prefix = "".to_string();
            } else if part[0] == prefix {
                // found (at least) two parts with the same prefix
                // store the part in raw without it's prefix
                let i = pieces.len() - 1; // wow, you can't do this inline...
                pieces[i].raw.push(part.split_first().unwrap().1.to_vec())
            } else {
                // we found a new prefix, create a new piece to store it
                prefix = part[0].clone();
                let raw = part.iter().skip(1).cloned().collect();
                let piece = NamePiece::from(prefix.clone(), vec![raw]);
                pieces.push(piece);
            }
        }
        // we don't need the raw data anymore, it's all been copied somewhere else
        if !self.raw.is_empty() {
            self.raw = vec![];
        }
        if !pieces.is_empty() {
            for p in &mut pieces {
                p.process();
            }
            self.pieces = Some(pieces);
        }
    }

    /// once we have processed all the name pieces, we can collapse them
    /// into a single string
    fn collapse(&self, w: &mut String, is_last: bool) {
        if self.prefix.is_empty() {
            // this is the "head" Piece, it has no filler
            // just write out the pieces
            if let Some(ref pieces) = self.pieces {
                let last_i = pieces.len() - 1;
                for (i, piece) in pieces.iter().enumerate() {
                    piece.collapse(w, last_i == i);
                }
            }
            return;
        }
        w.write_str(&self.prefix).unwrap();
        if let Some(ref pieces) = self.pieces {
            // there are some names after you, more to write
            let last_i = pieces.len() - 1;
            if last_i == 0 {
                // if you only have one piece, then you are foo-bar-baz-etc
                w.write_str("-").unwrap();
            } else {
                // else you are foo-[bar, bar-baz-etc] (unless you are the beginning)
                w.write_str("-[").unwrap();
            }
            for (i, piece) in pieces.iter().enumerate() {
                piece.collapse(w, last_i == i);
            }
            if last_i != 0 {
                w.write_str("]").unwrap();
            }
        }
        if !is_last {
            w.write_str(", ").unwrap();
        }
    }
}

// ------------------------------
// -- TESTS

#[test]
/// #TST-read-family.parent
fn sanity_parent() {
    fn parent(name: &Name) -> StrResult<Name> {
        match name.parent() {
            Some(n) => Ok(n),
            None => Err("no parent".into()),
        }
    }

    assert_generic(
        parent,
        &[
            // no parents
            (name!("REQ-foo"), None),
            (name!("TST-a"), None),
            (name!("TST-23kjskljef32"), None),
            // has parents
            (name!("REQ-a-b"), Some(name!("REQ-a"))),
            (name!("REQ-A-B"), Some(name!("REQ-A"))),
            (name!("REQ-aasdf-bbSdf-DES"), Some(name!("REQ-aasdf-bbSdf"))),
        ],
    );
}

#[test]
/// #TST-read-family.auto_partof
fn sanity_auto_partof() {
    fn auto_partof(name: &Name) -> StrResult<Name> {
        match name.auto_partof() {
            Some(n) => Ok(n),
            None => Err("no auto partof".into()),
        }
    }
    assert_generic(
        auto_partof,
        &[
            (name!("REQ-foo"), None),
            (name!("REQ-a-b"), None),
            (name!("REQ-A-B"), None),
            (
                name!("spc-aasdf-bbSdf-DES"),
                Some(name!("REQ-aasdf-bbSdf-DES")),
            ),
            (name!("TSt-a"), Some(name!("SPC-a"))),
            (name!("TST-23kjskljef32"), Some(name!("SPC-23kjskljef32"))),
        ],
    );
}

#[test]
/// #TST-read-family.collapse
fn sanity_collapse_name() {
    let values = &[
        ("REQ-foo", None, indexset!["REQ-foo"]),
        ("REQ-[bar, foo]", None, indexset!["REQ-foo", "REQ-bar"]),
        (
            "REQ-[zay, bar-[baz, bom], foo]",
            Some("REQ-[bar-[baz, bom], foo, zay]"),
            indexset!["REQ-foo", "REQ-bar-baz", "REQ-bar-bom", "REQ-zay"],
        ),
        (
            "SPC-[foo, foo-bob, bar], REQ-baz, SPC-foo-baz",
            Some("REQ-baz, SPC-[bar, foo, foo-[baz, bob]]"),
            indexset![
                "REQ-baz",
                "SPC-bar",
                "SPC-foo",
                "SPC-foo-baz",
                "SPC-foo-bob",
            ],
        ),
    ];
    assert_collapsed_valid(values);
}

#[test]
/// #TST-read-family.collapse_invalid
fn sanity_collapse_name_invalid() {
    let values = &[
        "REQ",                       // invalid name
        "REQ-foo-[bar-, baz]",       // extra `-`
        "REQ-[foo, [bar]]",          // `[` can't appear by itself
        "SPC-foo[bar",               // no closing brace
        "SPC-foo]",                  // no opening brace
        "SPC-[foo]]",                // no opening brace
        "SPC-foo-[bar, [baz, bom]]", // brackets not after `-`
    ];
    assert_collapsed_invalid(values);
}

#[test]
fn sanity_auto_partofs() {
    let req_foo = name!("REQ-foo");
    let req_foo_bar = name!("REQ-foo-bar");
    let spc_foo = name!("SPC-foo");
    let tst_foo = name!("TST-foo");
    let tst_foo_bar = name!("TST-foo-bar");

    let spc_a_b = name!("SPC-a-b");
    let tst_a_b = name!("TST-a-b");

    let file = PathAbs::mock("/fake");

    let names = indexmap!{
        req_foo.clone() => file.clone(),
        req_foo_bar.clone() => file.clone(),
        spc_foo.clone() => file.clone(),
        tst_foo.clone() => file.clone(),
        tst_foo_bar.clone() => file.clone(),
        spc_a_b.clone() => file.clone(),
        tst_a_b.clone() => file.clone(),
    };

    let expected = indexmap!{
        req_foo.clone() => indexset![],
        req_foo_bar.clone() => indexset![req_foo.clone()],
        spc_foo.clone() => indexset![req_foo.clone()],
        tst_foo.clone() => indexset![spc_foo.clone()],
        // contains no auto -- it doesn't exist
        tst_foo_bar.clone() => indexset![tst_foo.clone()],
        spc_a_b.clone() => indexset![],
        tst_a_b.clone() => indexset![spc_a_b.clone()],
    };
    let auto = auto_partofs(&names);
    assert_eq!(expected, auto);
}

#[test]
fn sanity_strip_auto_partofs() {
    let mut result = indexset![
        name!("REQ-bar"),
        name!("REQ-foo"),
        name!("REQ-foo-bar"),
        name!("SPC-bar"),
        name!("SPC-foo"),
    ];
    let expected = indexset![name!("REQ-bar"), name!("REQ-foo"), name!("SPC-bar"),];
    strip_auto_partofs(&name!("SPC-foo-bar"), &mut result);
    assert_eq!(expected, result);
}

proptest! {
    #[test]
    /// #TST-read-family.fuzz_parent
    fn fuzz_name_parent(ref name in arb_name()) {
        // Basically do the same thing as the code but in a slightly different way
        let mut items = name.raw.split('-').map(|s| s.to_string()).collect::<Vec<_>>();
        if items.len() > 2 {
            items.pop();
            let expected_raw = items.join("-");
            let expected = Name::from_str(&expected_raw).unwrap();
            let result = name.parent().unwrap();
            assert_eq!(expected_raw, result.raw);
            assert_eq!(expected, result);
        } else {
            assert!(name.parent().is_none());
        }
    }

    #[test]
    /// #TST-read-family.fuzz_auto_partof
    fn fuzz_name_auto_partof(ref name in arb_name()) {
        let ty = match name.ty {
            Type::REQ => {
                assert!(name.auto_partof().is_none());
                return Ok(());
            },
            Type::SPC => "REQ",
            Type::TST => "SPC",
        };
        let mut items = name.raw.split('-').map(|s| s.to_string()).collect::<Vec<_>>();
        items[0] = ty.into();
        let expected_raw = items.join("-");
        let expected = Name::from_str(&expected_raw).unwrap();
        let result = name.auto_partof().unwrap();
        assert_eq!(expected_raw, result.raw);
        assert_eq!(expected, result);
    }

    #[test]
    /// #TST-read-family.fuzz_collapse
    fn fuzz_collapse_name_roundtrip(ref names in arb_names(25)) {
        let collapsed = collapse_names(names);
        let expanded = expand_names(&collapsed).expect("failed expanding names");
        assert_eq!(*names, Names::from(expanded))
    }

    #[test]
    /// #TST-read-family.fuzz_serde
    /// This actually creates expected json by first sorting the names
    fn fuzz_names_serde(ref names in arb_names(25)) {
        // construct expected json by sorting and formatting
        let expected_json = {
            let mut sorted = names.iter().cloned().collect::<Vec<_>>();
            sorted.sort();
            let strs = sorted.iter()
                .map(|s| format!("{:?}", s.as_str()))
                .collect::<Vec<_>>();
            format!("[{}]", strs.join(","))
        };
        // do serde-roundtrip as well
        let result_json = json::to_string(&names).unwrap();
        let result: Names = json::from_str(&result_json).unwrap();
        assert_eq!(*names, result);
        assert_eq!(expected_json, result_json);
    }
}
