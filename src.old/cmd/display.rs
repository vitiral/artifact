
/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! methods to format the `FmtArtifact` object and write it to a stream

use dev_prefix::*;
use types::*;
use cmd::types::*;
use utils;

impl FmtArtifact {
    /// write the formatted version of the artifact to the
    /// cmdline writter
    ///
    /// #SPC-cmd-ls-color
    #[allow(cyclomatic_complexity)] // TODO: break this up
    pub fn write<W: io::Write>(
        &self,
        w: &mut W,
        cwd: &Path,
        artifacts: &Artifacts,
        color: bool,
        indent: u8,
    ) -> io::Result<()> {
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
                if color {
                    write!(w, "{}", Red.bold().blink().paint(self.name.raw.as_str())).unwrap();
                } else {
                    write!(w, "{}", self.name.raw).unwrap();
                }
                return Ok(());
            }
        };

        // format the completeness and name
        let completed_str = if artifact.completed < 0. {
            "-1".to_string()
        } else {
            ((artifact.completed * 100.) as u8).to_string()
        };
        let tested_str = if artifact.tested < 0. {
            "-1".to_string()
        } else {
            ((artifact.tested * 100.) as u8).to_string()
        };
        let completed_len = completed_str.len();
        let tested_len = tested_str.len();
        if color {
            let (d_sym, d_perc, t_sym, t_perc, name) =
                if artifact.completed >= 1. && artifact.tested >= 1. {
                    let name = if nfno {
                        Green.paint(self.name.raw.as_str())
                    } else {
                        Green.bold().underline().paint(self.name.raw.as_str())
                    };
                    (
                        Green.bold().paint("D"),
                        Green.bold().paint(completed_str),
                        Green.bold().paint("T"),
                        Green.bold().paint(tested_str),
                        name,
                    )
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
                    } else if artifact.completed < 0. {
                        (
                            Red.bold().blink().paint("!"),
                            Red.bold().blink().paint(completed_str),
                        )
                    } else {
                        (Red.bold().paint("-"), Red.bold().paint(completed_str))
                    };
                    let (t_sym, t_perc) = if artifact.tested >= 1. {
                        score += 2;
                        (Blue.bold().paint("T"), Blue.bold().paint(tested_str))
                    } else if artifact.tested >= 0.5 {
                        score += 1;
                        (Yellow.bold().paint("-"), Yellow.bold().paint(tested_str))
                    } else if artifact.tested < 0. {
                        (
                            Red.bold().blink().paint("!"),
                            Red.bold().blink().paint(tested_str),
                        )
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
                for _ in 0..(3 - completed_len) {
                    try!(w.write_all(" ".as_ref()));
                }
                try!(write!(w, "{}% ", d_perc));
                // format tested %
                for _ in 0..(3 - tested_len) {
                    try!(w.write_all(" ".as_ref()));
                }
                try!(write!(w, "{}% ", t_perc));
                try!(write!(w, "| {} ", name));
            }
        } else if nfno {
            try!(write!(w, "{}", &self.name.raw));
        } else {
            let d_sym = if artifact.completed >= 1. { "D" } else { "-" };
            let t_sym = if artifact.tested >= 1. { "T" } else { "-" };
            try!(write!(
                w,
                "|{}{}| {:>3}% {:>3}% | {}",
                d_sym,
                t_sym,
                completed_str,
                tested_str,
                &self.name.raw
            ));
        }

        if nfno {
            return Ok(());
        }

        // format the parts
        if let Some(ref parts) = self.parts {
            self.write_start(w, "\n * parts: ", color);
            for (n, p) in parts.iter().enumerate() {
                if self.long {
                    w.write_all("\n    ".as_ref()).unwrap();
                }
                try!(p.write(w, cwd, artifacts, color, indent + 1));
                if !self.long && n + 1 < parts.len() {
                    w.write_all(", ".as_ref()).unwrap();
                }
            }
        }

        // format the artifacts that are a partof this artifact
        if let Some(ref partof) = self.partof {
            self.write_start(w, "\n * partof: ", color);
            let mut first = true;
            for p in partof {
                if !first && p.name_only() {
                    try!(w.write_all(", ".as_ref()));
                }
                first = false;
                try!(p.write(w, cwd, artifacts, color, indent + 1));
            }
        }

        // format the location that where the implementation of this artifact can be found
        if let Some(ref done) = self.done {
            self.write_start(w, "\n * done: ", color);
            if color {
                try!(write!(w, "{}", Green.paint(done.as_ref())));
            } else {
                try!(w.write_all(done.as_ref()));
            }
            try!(w.write_all(" ".as_ref()));
        }

        // format where the artifact is defined
        if let Some(ref def) = self.def {
            self.write_start(w, "\n * defined-at: ", color);
            let def = utils::relative_path(def.as_path(), cwd);
            try!(write!(w, "{}", def.display()));
        }

        // format the text
        // TODO: use markdown to apply styles to the text
        if let Some(ref text) = self.text {
            self.write_start(w, "\n * text:\n", color);
            w.write_all(text.trim_right().as_ref()).unwrap();
            if self.long {
                w.write_all("\n".as_ref()).unwrap();
            }
        }

        try!(w.write_all("\n".as_ref()));
        Ok(())
    }

    fn write_start<W: io::Write>(&self, w: &mut W, msg: &str, color: bool) {
        if self.long {
            if color {
                write!(w, "{}", Green.paint(msg)).unwrap();
            } else {
                w.write_all(msg.as_ref()).unwrap();
            }
        } else {
            w.write_all("\t| ".as_ref()).unwrap();
        }
    }

    /// return whether this object is only the name
    /// if it is, it is formatted differently
    fn name_only(&self) -> bool {
        match (&self.def, &self.parts, &self.partof, &self.done, &self.text) {
            (&None, &None, &None, &None, &None) => true,
            _ => false,
        }
    }
}

pub fn write_table_header<W: io::Write>(w: &mut W, fmt_set: &FmtSettings) {
    let mut header = String::new();
    header.write_str("|  | DONE TEST | NAME").unwrap();
    if fmt_set.parts {
        header.write_str("\t| PARTS   ").unwrap();
    }
    if fmt_set.partof {
        header.write_str("\t| PARTOF   ").unwrap();
    }
    if fmt_set.loc_path {
        header.write_str("\t| IMPLEMENTED   ").unwrap();
    }
    if fmt_set.def {
        header.write_str("\t| DEFINED   ").unwrap();
    }
    if fmt_set.text {
        header.write_str("\t| TEXT").unwrap();
    }
    header.push('\n');
    if fmt_set.color {
        write!(w, "{}", Style::new().bold().paint(header)).unwrap();
    } else {
        write!(w, "{}", header).unwrap();
    }
}
