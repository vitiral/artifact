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
//! These are tests and helpers for testing family relations,
//! such as parts/partof.

use ergo::json;

use super::dev_prelude::*;
use super::name::arb_name;
use super::raw_names::arb_names_raw;
use artifact_data::raw_names::NamesRaw;

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
pub fn assert_collapsed_valid(values: &[(&str, Option<&str>, IndexSet<&str>)]) {
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

pub fn assert_collapsed_invalid(raw: &[&str]) {
    let errors = raw
        .iter()
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
