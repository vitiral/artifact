//! methods to format the FmtArtifact object and write it to a stream

use std::io;
use std::fmt::Write;

use ansi_term::Colour::{Red, Blue, Green, Yellow};

use core::{Settings, Artifacts};
pub use core::fmt::*;

impl FmtArtifact {
    /// write the formatted version of the artifact to the
    /// writter
    /// [SPC-ui-cmdline-ls-flags-impl-formatting]
    pub fn write<W: io::Write> (&self, w: &mut W, artifacts: &Artifacts,
                                settings: &Settings, indent: u8)
                                -> io::Result<()> {
        let nfno = indent > 0 && self.name_only(); // not-first-name-only
        if !self.name_only() {
            for _ in 0..(indent * 2) {
                try!(w.write_all(" ".as_ref()));
            }
        }
        trace!("formatting artifact: {}", self.name);
        let artifact = match artifacts.get(&self.name) {
            Some(a) => a,
            None => {
                // invalid partof value
                if settings.color {
                    write!(w, "{}", Red.bold().blink().paint(self.name.raw.as_str())).unwrap();
                } else {
                    write!(w, "{}", self.name.raw).unwrap();
                }
                return Ok(());
            }
        };

        // format the completeness and name
        let completed_str = ((artifact.completed * 100.) as i64).to_string();
        let tested_str = ((artifact.tested * 100.) as i64).to_string();
        if settings.color {

            // [SPC-ui-cmdline-ls-color]
            let (d_sym, d_perc, t_sym, t_perc, name) = if artifact.completed >= 1. &&
                    artifact.tested >= 1. {
                let name = if nfno {
                    Green.paint(self.name.raw.as_str())
                } else {
                    Green.bold().underline().paint(self.name.raw.as_str())
                };
                (Green.bold().paint("D"), Green.bold().paint(completed_str),
                 Green.bold().paint("T"), Green.bold().paint(tested_str),
                 name)
            } else {
                let mut score = 0;
                let (d_sym, d_perc) = if artifact.completed >= 1. {
                    score += 3;
                    (Blue.bold().paint("D"), Blue.bold().paint(completed_str))
                } else if artifact.completed >= 0.7 {
                    score += 2;
                    (Yellow.bold().paint("-"), Yellow.bold().paint(completed_str))
                } else if artifact.completed >= 0.4 {
                    score += 1;
                    (Yellow.bold().paint("-"), Yellow.bold().paint(completed_str))
                } else {
                    (Red.bold().paint("-"), Red.bold().paint(completed_str))
                };
                let (t_sym, t_perc) = if artifact.tested >= 1. {
                    score += 2;
                    (Blue.bold().paint("T"), Blue.bold().paint(tested_str))
                } else if artifact.tested >= 0.5 {
                    score += 1;
                    (Yellow.bold().paint("-"), Yellow.bold().paint(tested_str))
                } else {
                    (Red.bold().paint("-"), Red.bold().paint(tested_str))
                };
                let name = match score {
                    3...4 => Blue,
                    1...2 => Yellow,
                    0 => Red,
                    _ => unreachable!(),
                };
                let sname = self.name.raw.as_str();
                let name = if nfno {
                    name.paint(sname)
                } else {
                    name.bold().underline().paint(sname)
                };
                (d_sym, d_perc, t_sym, t_perc, name)
            };
            if nfno {
                try!(write!(w, "{}", name));
            } else {
                try!(write!(w, "|{}{}| ", d_sym, t_sym));
                // format completed %
                for _ in 0..(3 - d_perc.len()) {
                    try!(w.write_all(" ".as_ref()));
                }
                try!(write!(w, "{}% ", d_perc));
                // format tested %
                for _ in 0..(3 - t_perc.len()) {
                    try!(w.write_all(" ".as_ref()));
                }
                try!(write!(w, "{}% ", t_perc));
                try!(write!(w, "| {} ", name));
                if name.len() < 45 {
                    for _ in 0..(45 - name.len()) {
                        try!(w.write_all(" ".as_ref()));
                    }
                }
            }
        } else {
            if nfno {
                try!(write!(w, "{}", &self.name.raw));
            } else {
                let d_sym = if artifact.completed >= 1. {"D"} else {"-"};
                let t_sym = if artifact.tested >= 1. {"T"} else {"-"};
                try!(write!(w, "|{}{}|{:>3}% {:>3}%| {:<45}", d_sym, t_sym,
                            completed_str, tested_str, &self.name.raw));
            }
        }

        if nfno {
            return Ok(());
        }

        // format the references
        if let Some(ref refs) = self.refs {
            self.write_header(w, "\n * refs: ", settings);
            let sep = if self.long {"\n    "} else {" "};
            for r in refs {
                try!(w.write_all(sep.as_ref()));
                try!(w.write_all(r.as_ref()));
            }
            self.write_end(w)
        }

        // format the parts
        if let Some(ref parts) = self.parts {
            self.write_header(w, "\n * parts: ", settings);
            let mut first = true;
            let mut num_written = 0;
            for p in parts {
                if self.long {
                    w.write_all("\n    ".as_ref()).unwrap();
                }
                try!(p.write(w, artifacts, settings, indent + 1));
                num_written += 1;
                if !self.long && num_written < parts.len() {
                    w.write_all(", ".as_ref()).unwrap();
                }
            }
            self.write_end(w)
        }

        // format the artifacts that are a partof this artifact
        if let Some(ref partof) = self.partof {
            self.write_header(w, "\n * partof: ", settings);
            let mut first = true;
            for p in partof {
                if !first && p.name_only() {
                    try!(w.write_all(", ".as_ref()));
                }
                first = false;
                try!(p.write(w, artifacts, settings, indent + 1));
            }
            self.write_end(w);
        }

        // format the location that where the implementation of this artifact can be found
        if self.loc.is_some() {
            self.write_header(w, "\n * implemented-at-loc: ", settings);
            let mut loc_str = String::new();
            write!(loc_str, "{:?}", self.loc);
            if settings.color {
                try!(write!(w, "{}", Green.paint(loc_str)));
            } else {
                try!(w.write_all(loc_str.as_ref()));
            }
            try!(w.write_all(" ".as_ref()));
        }

        // format where the artifact is defined
        if let Some(ref path) = self.path {
            self.write_header(w, "\n * defined-at: ", settings);
            try!(w.write_all(path.to_string_lossy().as_ref().as_ref()));
            self.write_end(w)
        }

        // format the text
        // TODO: use markdown to apply styles to the text
        if let Some(ref text) = self.text {
            self.write_header(w, "\n * text:\n    ", settings);
            let lines: Vec<_> = text.split("\n").collect();
            let text = lines.join("\n    ");
            w.write_all(text.as_ref()).unwrap();
        }

        try!(w.write_all("\n".as_ref()));
        Ok(())
    }

    fn write_header<W: io::Write> (&self, w: &mut W, msg: &str, settings: &Settings) {
        if self.long {
            if settings.color {
                write!(w, "{}", Green.paint(msg)).unwrap();
            } else {
                w.write_all(msg.as_ref()).unwrap();
            }
        } else {
            w.write_all("| ".as_ref()).unwrap();
        }
    }

    fn write_end<W: io::Write> (&self, w: &mut W) {
        w.write_all(" ".as_ref()).unwrap();
    }

    /// return whether this object is only the name
    /// if it is, it is formatted differently
    fn name_only(&self) -> bool {
        match (&self.path, &self.parts, &self.partof,
               &self.loc, &self.refs, &self.text) {
            (&None, &None, &None, &None, &None, &None) => true,
            _ => false,
        }
    }
}

