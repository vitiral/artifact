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
//! #TST-data-raw
//! This module is for testing the serialization and deserialization
//! of RAW artifacts.

use rand::Rng;
use serde_json;
use name::{Name, Type};
use family::Names;
use raw::{from_markdown, to_markdown, TextRaw, ArtifactRaw, NAME_LINE_RE, ATTRS_END_RE};
use raw_names::NamesRaw;
use regex_generate;
use test::dev_prelude::*;
use test::family::{arb_topologically_sorted_names, rand_select_partof};

// ------------------------------
// -- FUZZ METHODS

/// Sanitize randomly generated text in-place.
///
/// This is used mostly in case `\n# ART-name\n` is randomly inserted
pub fn sanitize_rand_text_raw(lines: &[String]) -> Option<TextRaw> {
    let mut text: String = lines
        .iter()
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

/// Generate random text with (TODO) links to artifacts in it.
pub fn random_text_raw<R: Rng+Clone>(rng: &mut R, name: &Name) -> Option<TextRaw> {
    let mut text = String::new();
    let num_lines = rng.gen_range(0, 50);
    let lines = {
        let mut r = rng.clone();
        let mut textgen = regex_generate::Generator::parse(
            // Any unicode character can be in the "line" except:
            // - newline
            // - control characters
            // r"[^\n[:cntrl:]]{1,50}",

            // Any ascii except newline and cntrl
            r"(?i)[[:ascii:]&&[^\n[:cntrl:]]]{1,50}",
            rng,
        ).unwrap();
        let mut lines = Vec::new();
        for _ in 0..num_lines {
            if r.next_f32() < 0.2 {
                // 20% chance of blank line
                lines.push("".to_string());
                continue;
            }
            let mut line: Vec<u8> = Vec::with_capacity(100);
            textgen.generate(&mut line);
            lines.push(String::from_utf8(line).unwrap())
        }
        lines
    };
    // TODO: randomly insert references to other artifacts
    sanitize_rand_text_raw(&lines)
}

/// This returns a random set of artifacts for testing
/// serialization/deserialization.
///
/// These artifacts are NOT necessarily "valid", i.e. their `partof`
/// field is constructed entirely randomly so the links are probably invalid
pub fn arb_random_raw_artifacts(size: usize)
    -> BoxedStrategy<BTreeMap<Name, ArtifactRaw>>
{
    arb_topologically_sorted_names(size)
        .prop_perturb(|names, mut rng| {
            let names = Vec::from_iter(names.iter().cloned());
            BTreeMap::from_iter(names.iter()
                .enumerate()
                .map(|(i, n)| {
                    let done = if rng.next_f32() < 0.05 {
                        Some("TODO: use random string".to_string())
                    } else {
                        None
                    };
                    let partof = {
                        let p = rand_select_partof(&mut rng, i, &names);
                        if p.is_empty() {
                            None
                        } else {
                            Some(NamesRaw::from(HashSet::from_iter(p.iter().cloned())))
                        }
                    };
                    let text = random_text_raw(&mut rng, n);
                    let artraw = ArtifactRaw {
                        done: done,
                        partof: partof,
                        text: text,
                    };
                    (n.clone(), artraw)
                })
            )
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
fn arts_from_toml_str(s: &str) -> StrResult<BTreeMap<Name, ArtifactRaw>> {
    from_toml_str(s)
}

/// Sometimes I really love compilers.
///
/// There is some kind of lifetime BS if you try to use the function directly...
fn arts_from_json_str(s: &str) -> StrResult<BTreeMap<Name, ArtifactRaw>> {
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
    let exp_1 = btreemap! {
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

    /// Redefined to have correct signature
    fn from_md(raw: &String) -> StrResult<BTreeMap<Name, ArtifactRaw>> {
        let out = match from_markdown(raw.as_bytes()) {
            Ok(arts) => arts,
            Err(e) => return Err(e.to_string()),
        };

        // throw in a check that the roundtrip works
        let new_raw = serde_roundtrip("markdown", from_markdown_str, ::raw::to_markdown, &out).unwrap();
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
    #[ignore] // TODO: very slow
    #[cfg(not(feature = "cache"))]
    fn fuzz_raw_artifacts_serde(ref artifacts in arb_random_raw_artifacts(20)) {
        serde_roundtrip("markdown", from_markdown_str, ::raw::to_markdown, &artifacts).expect("md");
        serde_roundtrip("toml", arts_from_toml_str, to_toml_string, &artifacts).expect("toml");
        serde_roundtrip("json", arts_from_json_str, to_json_string, &artifacts).expect("json");
    }
}
