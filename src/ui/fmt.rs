use super::types::*;

/// format ArtNames in a reasonable way
pub fn fmt_names(names: &Vec<&ArtName>) -> String {
    if names.len() == 0 {
        return "".to_string();
    }
    let mut s = String::new();
    for n in names {
        write!(s, "{}, ", n.raw).unwrap();
    }
    let len = s.len();
    s.truncate(len - 2); // remove last ", "
    s
}

/// use several configuration options and pieces of data to represent
/// how the artifact should be formatted
// [#SPC-core-fmt-func]
pub fn fmt_artifact(name: &ArtName, artifacts: &Artifacts, fmtset: &FmtSettings,
                recurse: u8, displayed: &mut HashSet<ArtName>) -> FmtArtifact {
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
            if recurse == 0 || displayed.contains(&p) {
                part = FmtArtifact::default();
                part.name = p.clone();
            } else {
                part = fmt_artifact(&p, artifacts, fmtset, recurse - 1, displayed);
                displayed.insert(p.clone());
            }
            parts.push(part);
        }
        parts.sort_by_key(|p| p.name.clone());  // TODO: get around clone here
        out.parts = Some(parts);
    }
    if fmtset.partof {
        let mut partof = artifact.partof.iter().map(|p| p.clone()).collect::<Vec<ArtName>>();
        partof.sort();
        let partof = partof.drain(0..)
            .map(|n| FmtArtifact{name: n, ..FmtArtifact::default()})
            .collect();
        out.partof = Some(partof);
    }
    if fmtset.loc_path {
        out.loc = artifact.loc.clone();
    }
    if fmtset.refs {
        out.refs = Some(artifact.refs.clone());
    }
    if fmtset.text {
        if fmtset.long {
            out.text = Some(artifact.text.clone());
        } else {
            // return only the first "line" according to markdown
            let mut s = String::new();
            for l in artifact.text.lines() {
                let l = l.trim();
                if l == "" {
                    break;
                }
                s.write_str(l).unwrap();
                s.push(' ');
            }
            out.text = Some(s);
        }
    }
    out.name = name.clone();
    out
}
