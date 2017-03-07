use dev_prefix::*;
use types::*;
use ui::types::*;

/// format `Names` in a reasonable way
pub fn fmt_names(names: &[NameRc]) -> String {
    if names.is_empty() {
        return "".to_string();
    }
    names.iter().map(|n| &n.raw).cloned().collect::<Vec<_>>().join(", ")
}

/// use several configuration options and pieces of data to represent
/// how the artifact should be formatted
pub fn fmt_artifact(name: &NameRc,
                    artifacts: &Artifacts,
                    fmtset: &FmtSettings,
                    recurse: u8,
                    displayed: &mut Names)
                    -> FmtArtifact {
    let artifact = artifacts.get(name).unwrap();
    let mut out = FmtArtifact::default();
    out.long = fmtset.long;
    if fmtset.path {
        out.path = Some(artifact.path.clone());
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
        let partof = partof.drain(0..)
            .map(|n| FmtArtifact { name: n, ..FmtArtifact::default() })
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
            // return only the first "line" according to markdown
            let mut end = artifact.text.find('\n').unwrap_or_else(|| artifact.text.len());
            // TODO: Calculate Unicode width?
            const MAX_LINE_LEN: usize = 50;
            let should_add_ellipsis: bool;
            if end > MAX_LINE_LEN {
                should_add_ellipsis = true;
                // Allow space for '...'
                end = MAX_LINE_LEN - 3;
            } else {
                should_add_ellipsis = false;
            }
            // Find a Unicode-safe place to split
            // TODO: Split at a grapheme cluster?
            while !artifact.text.is_char_boundary(end) {
                end -= 1;
            }
            let mut s = artifact.text[..end].into();
            if should_add_ellipsis {
                s += "...";
            }
            out.text = Some(s);
        }
    }
    out.name = name.clone();
    out
}
