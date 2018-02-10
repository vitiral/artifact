use dev_prefix::*;
use types::*;
use ui::types::*;
use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;

/// format `Names` in a reasonable way
pub fn fmt_names(names: &[NameRc]) -> String {
    if names.is_empty() {
        return "".to_string();
    }
    names
        .iter()
        .map(|n| &n.raw)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ")
}

/// use several configuration options and pieces of data to represent
/// how the artifact should be formatted
pub fn fmt_artifact(
    name: &NameRc,
    artifacts: &Artifacts,
    fmtset: &FmtSettings,
    recurse: u8,
    displayed: &mut Names,
) -> FmtArtifact {
    let artifact = artifacts.get(name).unwrap();
    let mut out = FmtArtifact::default();
    out.long = fmtset.long;
    if fmtset.def {
        out.def = Some(artifact.def.clone());
    }
    if fmtset.parts {
        let mut parts: Vec<FmtArtifact> = Vec::new();
        for p in &artifact.parts {
            let mut part;
            if recurse == 0 || displayed.contains(p) {
                part = FmtArtifact::default();
                part.name = p.clone();
            } else {
                part = fmt_artifact(p, artifacts, fmtset, recurse - 1, displayed);
                displayed.insert(p.clone());
            }
            parts.push(part);
        }
        parts.sort_by_key(|p| p.name.clone()); // TODO: get around clone here
        out.parts = Some(parts);
    }
    if fmtset.partof {
        let mut partof = artifact.partof.iter().cloned().collect::<Vec<NameRc>>();
        partof.sort();
        let partof = partof
            .drain(0..)
            .map(|n| {
                FmtArtifact {
                    name: n,
                    ..FmtArtifact::default()
                }
            })
            .collect();
        out.partof = Some(partof);
    }
    if fmtset.loc_path {
        out.done = match artifact.done {
            Done::Code(ref l) => Some(l.to_string()),
            Done::Defined(ref d) => Some(d.clone()),
            Done::NotDone => Some("".to_string()),
        };
    }
    if fmtset.text {
        if fmtset.long {
            out.text = Some(artifact.text.clone());
        } else {
            const MAX_LINE_LEN: usize = 50;
            let line_end = artifact
                .text
                .find('\n')
                .unwrap_or_else(|| artifact.text.len());
            let width = UnicodeWidthStr::width(&artifact.text[..line_end]);
            if width > MAX_LINE_LEN {
                // We need to allow space for '...'
                let trimmed = trim_unicode_length(&artifact.text, MAX_LINE_LEN - 3);
                out.text = Some(trimmed.to_string() + "...");
            } else {
                out.text = Some(artifact.text[..line_end].into());
            }
        }
    }
    out.name = name.clone();
    out
}

fn trim_unicode_length(text: &str, length: usize) -> &str {
    let mut end = 0;
    let mut width_accum = 0;
    for (begin, cluster) in text.grapheme_indices(true) {
        end = begin;
        width_accum += UnicodeWidthStr::width(cluster);
        if width_accum > length {
            break;
        }
    }
    if width_accum <= length {
        text
    } else {
        &text[..end]
    }
}

#[test]
fn test_trim_unicode_length() {
    assert_eq!(trim_unicode_length("", 10), "");
    assert_eq!(trim_unicode_length("hello", 10), "hello");
    assert_eq!(trim_unicode_length("hello", 2), "he");
    assert_eq!(
        trim_unicode_length("H\u{200D}e\u{200D}l\u{200D}l\u{200D}o", 5),
        "H\u{200D}e\u{200D}l\u{200D}l\u{200D}o"
    );
    assert_eq!(
        trim_unicode_length("H\u{200D}e\u{200D}", 2),
        "H\u{200D}e\u{200D}"
    );
    // Hello, World!
    assert_eq!(trim_unicode_length("你好世界", 8), "你好世界");
    assert_eq!(trim_unicode_length("你好世界", 4), "你好");
    assert_eq!(trim_unicode_length("你好世界", 5), "你好");
    assert_eq!(
        trim_unicode_length("γειά σου κόσμος", 15),
        "γειά σου κόσμος"
    );
    assert_eq!(
        trim_unicode_length("γειά σου κόσμος", 8),
        "γειά σου"
    );
    assert_eq!(
        trim_unicode_length("γειά σου κόσμος", 3),
        "γει"
    );
    // ZALGO!
    let z = "Z̡͕̃͗͐ͩ͐̽A̶̱͉ͩ̒̀̒L̋̒ͮ̎͛G̨̖̯̖̲͇Ö̵̹͔̞̱͖̾̍";
    let r = "Z̡͕̃͗͐ͩ͐̽A̶̱͉ͩ̒̀̒L̋̒ͮ̎͛G̨̖̯̖̲͇Ö̵̹͔̞̱͖̾̍";
    assert_eq!(trim_unicode_length(z, 5), r);
}
