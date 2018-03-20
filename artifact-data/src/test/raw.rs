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

use raw::{from_markdown, to_markdown, ArtifactRaw, TextRaw, ATTRS_END_RE, NAME_LINE_RE};
use raw_names::NamesRaw;
use test::dev_prelude::*;
use test::family::{arb_topologically_sorted_names, rand_select_partof};
use test::implemented::random_impl_links;

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

impl ArtifactRaw {
    pub fn empty() -> ArtifactRaw {
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
fn arts_from_toml_str(s: &str) -> StrResult<IndexMap<Name, ArtifactRaw>> {
    from_toml_str(s)
}

/// Sometimes I really love compilers.
///
/// There is some kind of lifetime BS if you try to use the function directly...
fn arts_from_json_str(s: &str) -> StrResult<IndexMap<Name, ArtifactRaw>> {
    from_json_str(s)
}

// ------------------------------
// -- TESTS
#[test]
fn sanity_from_markdown() {
    let raw_1 = r#"
# REQ-foo

req-foo text

#REQ-bar
partof:
- SPC-baz
###
REQ-bar text
#REQ-empty
#    req-weird
done: yes this is done
partof:
- REQ-baz
- SPC-bar
###


"#.to_string();
    let exp_raw_1 = r#"# REQ-bar
partof: SPC-baz
###
REQ-bar text


# REQ-empty


# REQ-foo

req-foo text


# req-weird
done: yes this is done

partof:
- REQ-baz
- SPC-bar
###"#.to_string();
    let mut exp_1 = indexmap! {
        name!("REQ-foo") => ArtifactRaw {
            done: None,
            partof: None,
            text: Some(TextRaw("\nreq-foo text\n".into())),
        },
        name!("REQ-bar") => ArtifactRaw {
            done: None,
            partof: Some(names_raw!("SPC-baz")),
            text: Some(TextRaw("REQ-bar text".into())),
        },
        name!("REQ-empty") => ArtifactRaw::empty(),
        name!("req-weird") => ArtifactRaw {
            done: Some("yes this is done".into()),
            partof: Some(names_raw!("REQ-baz, SPC-bar")),
            text: None,
        },
    };
    exp_1.sort_keys();

    /// Redefined to have correct signature
    fn from_md(raw: &String) -> StrResult<IndexMap<Name, ArtifactRaw>> {
        let out = match from_markdown(raw.as_bytes()) {
            Ok(arts) => arts,
            Err(e) => return Err(e.to_string()),
        };

        // throw in a check that the roundtrip works
        let new_raw =
            serde_roundtrip("markdown", from_markdown_str, ::raw::to_markdown, &out).unwrap();
        println!("### Original Raw:\n{}<END>", raw);
        println!("### New Raw:\n{}<END>", new_raw);
        Ok(out)
    }

    let values = &[(raw_1, Some(exp_1.clone()))];
    assert_generic(from_md, values);

    // sanity: assert one of the examples has exact markdown
    assert_eq!(exp_raw_1, to_markdown(&exp_1));
}

proptest! {
    #[test]
    fn fuzz_artifacts_serde(ref orig in arb_raw_artifacts(20)) {
        let mut artifacts = IndexMap::with_capacity(orig.len());
        for (n, a) in orig.iter() {
            artifacts.insert(n.clone(), a.clone());
        }
        serde_roundtrip("markdown", from_markdown_str, ::raw::to_markdown, &artifacts).expect("md");
        serde_roundtrip("toml", arts_from_toml_str, to_toml_string, &artifacts).expect("toml");
        serde_roundtrip("json", arts_from_json_str, to_json_string, &artifacts).expect("json");
    }
}
