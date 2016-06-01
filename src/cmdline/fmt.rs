//! methods to format the FmtArtifact object and write it to a stream

use std::io;
use std::fmt::Write;
use std::iter::FromIterator;
use std::collections::HashSet;
use std::path;

use ansi_term::Colour::{Red, Blue, Green, Yellow};
use ansi_term::Style;

use core::{Settings, Artifacts, ArtName};
pub use core::fmt::*;

impl FmtArtifact {
    /// write the formatted version of the artifact to the
    /// writter
    pub fn write<W: io::Write> (&self, w: &mut W, artifacts: &Artifacts,
                                settings: &Settings, indent: u8)
                                -> io::Result<()> {
        if !self.name_only() {
            for _ in 0..(indent * 2) {
                try!(w.write(" ".as_ref()));
            }
        }
        let artifact = artifacts.get(&self.name).unwrap();
        // print the name and completeness, colorized
        let completed_str = ((artifact.completed * 100.) as i64).to_string();
        let tested_str = ((artifact.tested * 100.) as i64).to_string();
        if settings.color {
            let (d_sym, d_perc, t_sym, t_perc, name) = if (
                    artifact.completed >= 1. && artifact.tested >= 1.) {
                (Blue.paint("D"), Blue.paint(completed_str),
                 Blue.paint("T"), Blue.paint(tested_str),
                 Blue.paint(self.name.raw.as_str()))
            } else {
                let mut score = 0;
                let (d_sym, d_perc) = if artifact.completed >= 1. {
                    score += 2;
                    (Green.paint("D"), Green.paint(completed_str))
                } else if artifact.completed >= 0.5 {
                    score += 1;
                    (Yellow.paint("-"), Yellow.paint(completed_str))
                } else {
                    (Red.paint("-"), Red.paint(completed_str))
                };
                let (t_sym, t_perc) = if artifact.tested >= 1. {
                    score += 2;
                    (Green.paint("T"), Green.paint(tested_str))
                } else if artifact.tested >= 0.3 {
                    score += 1;
                    (Yellow.paint("-"), Yellow.paint(tested_str))
                } else {
                    (Red.paint("-"), Red.paint(tested_str))
                };
                let name = match score {
                    3 => Green,
                    2 | 1 => Yellow,
                    0 => Red,
                    _ => unreachable!(),
                }.paint(self.name.raw.as_str());
                (d_sym, d_perc, t_sym, t_perc, name)
            };
            if self.name_only() {
                try!(write!(w, "{}", name));
            } else {
                try!(write!(w, "|{}{}| ", d_sym, t_sym));
                // format completed %
                for _ in 0..(3 - d_perc.len()) {
                    try!(w.write(" ".as_ref()));
                }
                try!(write!(w, "{}% ", d_perc));
                // format tested %
                for _ in 0..(3 - t_perc.len()) {
                    try!(w.write(" ".as_ref()));
                }
                try!(write!(w, "{}% ", t_perc));
                try!(write!(w, "| {} ", name));
                if name.len() < 45 {
                    for _ in 0..(45 - name.len()) {
                        try!(w.write(" ".as_ref()));
                    }
                }
            }
        } else {
            if self.name_only() {
                try!(write!(w, "{}", &self.name.raw));
            } else {
                let d_sym = if artifact.completed >= 1. {"D"} else {"-"};
                let t_sym = if artifact.tested >= 1. {"T"} else {"-"};
                try!(write!(w, "|{}{}|{:>3}% {:>3}%| {:<45}", d_sym, t_sym,
                            completed_str, tested_str, &self.name.raw));
            }
        }

        if self.name_only() {
            return Ok(());
        }

        if let Some(ref parts) = self.parts {
            try!(w.write("| ".as_ref()));
            let mut first = true;
            for p in parts {
                if !first && p.name_only() {
                    try!(w.write(", ".as_ref()));
                }
                first = false;
                try!(p.write(w, artifacts, settings, indent + 1));
            }

        }
        try!(w.write("\n".as_ref()));
        Ok(())
    }

    /// return whether this object is only the name
    /// if it is, it is formatted differently
    fn name_only(&self) -> bool {
        match (&self.path, &self.parts, &self.partof,
               &self.loc_name, &self.loc_path, &self.loc_valid,
               &self.refs, &self.text) {
            (&None, &None, &None, &None, &None, &None, &None, &None) => true,
            _ => false,
        }
    }
}

// // return the formatted lines as a vec of (indent, value) tuples
// fn _display_artifact(lines: &mut Vec<(u8, String)>, name: &ArtName,
//                      artifact: &Artifact, settings: &FmtSettings, recurse: u8,
//                      indent: u8) {
//     let mut s = String::new();
//     // The first line is always `[--] COMPLETED% TESTED% NAME`
//     write!(s, "[{}{}] ",
//            if artifact.completed >= 1. {"D"} else {"-"},
//            if artifact.tested >= 1. {"T"} else {"-"}).unwrap();

//     if artifact.completed < 0. {
//         write!(s, " NC  ").unwrap();
//     } else {
//         write!(s, "{:>4.0}%", artifact.completed * 100.).unwrap();
//     }
//     if artifact.tested < 0. {
//         write!(s, "  NC   ").unwrap();
//     } else {
//         write!(s, " {:>4.0}%  ", artifact.tested * 100.).unwrap();
//     }
//     if settings.long {
//         s.write_str(name.raw.as_str()).unwrap();
//         lines.push((indent, s.clone()));
//         s.clear();
//     } else {
//         write!(s, "{:<45}", name.raw).unwrap();
//     }

//     if settings.path {
//         let path = artifact.path.to_string_lossy();
//         if settings.long {
//             write!(s, "path: {}", path.as_ref()).unwrap();
//             lines.push((indent, s.clone()));
//             s.clear();
//         } else {
//             s.write_str("| ").unwrap();
//             s.write_str(path.as_ref()).unwrap();
//         }
//     }

//     if settings.parts {
//         let mut parts = Vec::from_iter(artifact.parts.iter());
//         parts.sort();
//         let parts = names(&parts);
//         if settings.long {
//             write!(s, "parts: {}", parts.as_str()).unwrap();
//             lines.push((indent, s.clone()));
//             s.clear();
//         } else {
//             s.write_str("| ").unwrap();
//             s.write_str(parts.as_str()).unwrap();
//         }
//     }

//     if settings.partof {
//         let mut partof = Vec::from_iter(artifact.partof.iter());
//         partof.sort();
//         let partof = names(&partof);
//         if settings.long {
//             write!(s, "partof: {}", partof.as_str()).unwrap();
//             lines.push((indent, s.clone()));
//             s.clear();
//         } else {
//             s.write_str("| ").unwrap();
//             s.write_str(partof.as_str()).unwrap();
//         }
//     }

//     if settings.loc_path {
//         if settings.long {
//             write!(s, "implemented: ").unwrap();
//         } else {
//             s.write_str("| ").unwrap()
//         }
//         if let Some(ref l) = artifact.loc {
//             s.write_str(l.path.to_string_lossy().as_ref()).unwrap();
//         } else {
//             // s.write_str("").unwrap();
//         }
//         if settings.long {
//             lines.push((indent, s.clone()));
//             s.clear();
//         }
//     }

//     if !settings.long {
//         lines.push((indent, s));
//     }
// }

// /// fully configurable display of an artifact
// pub fn display_artifact(name: &ArtName, artifact: &Artifact, settings: &FmtSettings)
//                         -> String {
//     let mut lines: Vec<(u8, String)> = Vec::new();
//     _display_artifact(&mut lines, name, artifact, settings, settings.recurse, 0);
//     let mut s = String::new();
//     for (indent, txt) in lines {
//         for _ in 0..indent {
//             s.push('|');
//         }
//         s.write_str(txt.as_str()).unwrap();
//         s.push('\n');
//     }
//     s
// }
