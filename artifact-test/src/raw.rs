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
//! #TST-read-raw
//! This module is for testing the serialization and deserialization
//! of RAW artifacts.

use super::dev_prelude::*;
use super::family::{arb_topologically_sorted_names, rand_select_partof};
use super::implemented::random_impl_links;
use artifact_data::raw::{from_markdown, to_markdown, ArtifactRaw, TextRaw, ATTRS_END_RE,
                         NAME_LINE_RE};
use artifact_data::raw_names::NamesRaw;

// ------------------------------
// -- FUZZ METHODS

/// Convert randomly generated text to something useful for artifact.text field.
///
/// This is used mostly in case `\n# ART-name\n` is randomly inserted
pub fn lines_to_text_raw<R: Rng + Clone>(
    rng: &mut R,
    subnames: &IndexSet<SubName>,
    references: &[&(Name, Option<SubName>)],
    mut lines: Vec<Vec<String>>,
) -> Option<TextRaw> {
    for sub in subnames.iter() {
        insert_word(rng, &mut lines, format!("[[{}]]", sub.as_str()));
    }

    for &&(ref name, ref sub) in references.iter() {
        insert_word(
            rng,
            &mut lines,
            format!("[[{}]]", name_ref_string(name, sub)),
        );
    }

    // TODO: add link references
    let mut text: String = lines
        .iter()
        .map(|l| l.join(" "))
        .filter(|l| !(NAME_LINE_RE.is_match(l) || ATTRS_END_RE.is_match(l)))
        .join("\n");

    string_trim_right(&mut text);
    if text.is_empty() {
        None
    } else {
        if text.contains('\n') {
            text.push('\n');
        }
        Some(TextRaw(text))
    }
}

/// This returns a random set of artifacts for testing
/// serialization/deserialization.
///
/// These artifacts are NOT necessarily "valid", i.e. their `partof`
/// field is constructed entirely randomly so the links are probably invalid
pub fn arb_raw_artifacts(size: usize) -> BoxedStrategy<BTreeMap<Name, ArtifactRaw>> {
    arb_topologically_sorted_names(size)
        .prop_perturb(|(names, sorted_names), mut rng| {
            let impl_links = random_impl_links(&mut rng, &names);

            // TODO: this should probably use logic defined somewhere else
            // but that logic doesn't exist yet
            let mut subnames: IndexMap<Name, IndexSet<SubName>> =
                IndexMap::with_capacity(impl_links.len());
            for &(ref name, ref sub) in &impl_links {
                let insert_it = !subnames.contains_key(name);
                if insert_it {
                    subnames.insert(name.clone(), indexset![]);
                }
                if let Some(ref s) = *sub {
                    let mut subs = subnames.get_mut(name).unwrap();
                    subs.insert(s.clone());
                }
            }
            BTreeMap::from_iter(sorted_names.iter().enumerate().map(|(i, name)| {
                let done = if rng.next_f32() < 0.05 {
                    Some("TODO: use random string".to_string())
                } else {
                    None
                };
                let partof = {
                    let p = rand_select_partof(&mut rng, i, &sorted_names);
                    if p.is_empty() {
                        None
                    } else {
                        Some(NamesRaw::from(IndexSet::from_iter(p.iter().cloned())))
                    }
                };
                let lines = random_lines(&mut rng);
                let references = {
                    let num = rng.gen_range(0, impl_links.len());
                    let mut l = expect!(rand::seq::sample_iter(&mut rng, &impl_links, num));
                    rng.shuffle(&mut l);
                    l
                };
                let text = lines_to_text_raw(&mut rng, &subnames[name], &references, lines);
                let artraw = ArtifactRaw {
                    done: done,
                    partof: partof,
                    text: text,
                };
                (name.clone(), artraw)
            }))
        })
        .boxed()
}

// ------------------------------
// -- METHODS / ATTRIBUTES

pub trait ArtifactRawExt {
    fn empty() -> ArtifactRaw;
}

impl ArtifactRawExt for ArtifactRaw {
    fn empty() -> ArtifactRaw {
        ArtifactRaw {
            done: None,
            partof: None,
            text: None,
        }
    }
}

/// Sometimes I really love compilers.
///
/// There is some kind of lifetime BS if you try to use the function directly...
pub fn arts_from_toml_str(s: &str) -> StrResult<IndexMap<Name, ArtifactRaw>> {
    from_toml_str(s)
}

/// Sometimes I really love compilers.
///
/// There is some kind of lifetime BS if you try to use the function directly...
pub fn arts_from_json_str(s: &str) -> StrResult<IndexMap<Name, ArtifactRaw>> {
    from_json_str(s)
}
