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
extern crate artifact_test;
use artifact_test::*;

use artifact_data::raw::{from_markdown, to_markdown, ArtFileType, ArtifactRaw, TextRaw};
use artifact_data::raw_names::NamesRaw;
use artifact_test::raw::{arb_raw_artifacts, arts_from_json_str, arts_from_toml_str};

#[test]
fn sanity_filetype() {
    assert_eq!(
        ArtFileType::from_path(Path::new("/foo/bar.toml")),
        Some(ArtFileType::Toml)
    );
    assert_eq!(
        ArtFileType::from_path(Path::new("this-is-it.md")),
        Some(ArtFileType::Md)
    );
    assert_eq!(
        ArtFileType::from_path(Path::new("/what.json")),
        Some(ArtFileType::Json)
    );
    assert_eq!(ArtFileType::from_path(Path::new("noext")), None);
    assert_eq!(ArtFileType::from_path(Path::new("other.ext")), None);
}

#[test]
fn sanity_from_markdown() {
    // #ART-SKIP
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
    // #ART-DONE

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
        let new_raw = serde_roundtrip(
            "markdown",
            from_markdown_str,
            ::artifact_data::raw::to_markdown,
            &out,
        ).unwrap();
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
        serde_roundtrip("markdown", from_markdown_str, ::artifact_data::raw::to_markdown, &artifacts).expect("md");
        serde_roundtrip("toml", arts_from_toml_str, to_toml_string, &artifacts).expect("toml");
        serde_roundtrip("json", arts_from_json_str, to_json_string, &artifacts).expect("json");
    }
}
