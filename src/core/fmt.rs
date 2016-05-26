use std::fmt::Write;
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
    write!(s, "[{}{}]",
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
