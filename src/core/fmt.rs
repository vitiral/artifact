use std::fmt::Write;
use std::iter::FromIterator;

use core::types::*;

/// format ArtNames in a reasonable way
pub fn names(names: &Vec<&ArtName>) -> String {
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

/// settings for what to format
pub struct FmtSettings {
    long: bool,
    recurse: u8,
    path: bool,
    parts: bool,
    partof: bool,
    loc_name: bool,
    loc_path: bool,
    text: bool,
    refs: bool,
    // completed: bool,
    // tested: bool,
}


/// return the formatted lines as a vec of (indent, value) tuples
fn _display_artifact(lines: &mut Vec<(u8, String)>, name: &ArtName,
                     artifact: &Artifact, settings: &FmtSettings, recurse: u8,
                     indent: u8) {
    let mut s = String::new();
    // The first line is always `[--] COMPLETED% TESTED% NAME`
    write!(s, "[{}{}] ",
           if artifact.completed >= 1. {"D"} else {"-"},
           if artifact.tested >= 1. {"T"} else {"-"}).unwrap();

    if artifact.completed < 0. {
        write!(s, " NC  ").unwrap();
    } else {
        write!(s, "{:4.1}%", artifact.completed * 100.).unwrap();
    }
    if artifact.tested < 0. {
        write!(s, " NC   ").unwrap();
    } else {
        write!(s, " {:4.1}%  ", artifact.tested * 100.).unwrap();
    }
    s.write_str(name.raw.as_str()).unwrap();
    if settings.long {
        lines.push((indent, s.clone()));
        s.clear();
    } else {
        s.write_str("| ").unwrap();
    }

    if settings.path {
        let path = artifact.path.to_string_lossy();
        if settings.long {
            write!(s, "path: {}", path.as_ref()).unwrap();
            lines.push((indent, s.clone()));
            s.clear();
        } else {
            s.write_str(path.as_ref()).unwrap();
            s.write_str("| ").unwrap();
        }
    }

    if settings.parts {
        let mut parts = Vec::from_iter(artifact.parts.iter());
        parts.sort();
        let parts = names(&parts);
        if settings.long {
            write!(s, "parts: {}", parts.as_str()).unwrap();
            lines.push((indent, s.clone()));
            s.clear();
        } else {
            s.write_str(parts.as_str()).unwrap();
            s.write_str("| ").unwrap();
        }
    }

    if settings.partof {
        let mut partof = Vec::from_iter(artifact.partof.iter());
        partof.sort();
        let partof = names(&partof);
        if settings.long {
            write!(s, "partof: {}", partof.as_str()).unwrap();
            lines.push((indent, s.clone()));
            s.clear();
        } else {
            s.write_str(partof.as_str()).unwrap();
            s.write_str("| ").unwrap();
        }
    }
    if !settings.long {
        lines.push((indent, s));
    }
}

/// fully configurable display of an artifact
pub fn display_artifact(name: &ArtName, artifact: &Artifact, settings: &FmtSettings)
                        -> String {
    let mut lines: Vec<(u8, String)> = Vec::new();
    _display_artifact(&mut lines, name, artifact, settings, settings.recurse, 0);
    "".to_string()
}


/// format most artifacts onto a single line, intended to be
/// displayed in a table, etc
pub fn artifact_line(name: &ArtName, artifact: &Artifact) -> String {
    let mut s = String::new();
    s.write_str(name.raw.as_str()).unwrap();
    let mut extra: i64 = 60 - s.len() as i64;
    while extra > 0 {
        s.push(' ');
        extra -= 1;
    }
    write!(s, "|{}{}",
           if artifact.completed >= 1. {"D"} else {"-"},
           if artifact.tested >= 1. {"T"} else {"-"}).unwrap();

    if artifact.completed < 0. {
        write!(s, "| ERROR ").unwrap();
    } else {
        write!(s, "| {:5.1} ", artifact.completed * 100.).unwrap();
    }
    if artifact.tested < 0. {
        write!(s, "| ERROR |").unwrap();
    } else {
        write!(s, "| {:5.1} |", artifact.tested * 100.).unwrap();
    }
    if artifact.ty == ArtType::SPC || artifact.ty == ArtType::TST {
        match &artifact.loc {
            &Some(ref l) => write!(s, " {}", l.loc).unwrap(),
            &None => write!(s, " NOT IMPLEMENTED").unwrap(),
        }
    }
    s
}
