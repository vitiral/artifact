/*  rst: the requirements tracking tool made for developers
    Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the Lesser GNU General Public License as published 
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the Lesser GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/
//! methods to format the `FmtArtifact` object and write it to a stream

use super::types::*;

impl FmtArtifact {
    /// write the formatted version of the artifact to the
    /// cmdline writter
    #[allow(cyclomatic_complexity)]  // TODO: break this up
    pub fn write<W: io::Write> (&self, w: &mut W, cwd: &Path,
                                artifacts: &Artifacts,
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
        if settings.color {
            // #SPC-ls-color]
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
                } else if artifact.completed < 0. {
                    (Red.bold().blink().paint("!"), Red.bold().blink().paint(completed_str))
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
                    (Red.bold().blink().paint("!"), Red.bold().blink().paint(tested_str))
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
                if name.len() < 45 {
                    for _ in 0..(45 - name.len()) {
                        try!(w.write_all(" ".as_ref()));
                    }
                }
            }
        } else if nfno {
            try!(write!(w, "{}", &self.name.raw));
        } else {
            let d_sym = if artifact.completed >= 1. {"D"} else {"-"};
            let t_sym = if artifact.tested >= 1. {"T"} else {"-"};
            try!(write!(w, "|{}{}| {:>3}% {:>3}% | {:<45} ", d_sym, t_sym,
                        completed_str, tested_str, &self.name.raw));
        }

        if nfno {
            return Ok(());
        }

        // format the parts
        if let Some(ref parts) = self.parts {
            self.write_header(w, "\n * parts: ", settings);
            for (n, p) in parts.iter().enumerate() {
                if self.long {
                    w.write_all("\n    ".as_ref()).unwrap();
                }
                try!(p.write(w, cwd, artifacts, settings, indent + 1));
                if !self.long && n + 1 < parts.len() {
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
                try!(p.write(w, cwd, artifacts, settings, indent + 1));
            }
            self.write_end(w);
        }

        // format the location that where the implementation of this artifact can be found
        if let Some(ref loc) = self.loc {
            self.write_header(w, "\n * implemented-at: ", settings);
            if settings.color {
                try!(write!(w, "{}", Green.paint(loc.to_string())));
            } else {
                try!(w.write_all(loc.to_string().as_ref()));
            }
            try!(w.write_all(" ".as_ref()));
        }

        // format where the artifact is defined
        if let Some(ref path) = self.path {
            self.write_header(w, "\n * defined-at: ", settings);
            let path = utils::relative_path(path.as_path(), cwd);
            try!(write!(w, "{}", path.display()));
            self.write_end(w);
        }

        // format the text
        // TODO: use markdown to apply styles to the text
        if let Some(ref text) = self.text {
            self.write_header(w, "\n * text:\n    ", settings);
            let lines: Vec<_> = text.split('\n').collect();
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
               &self.loc, &self.text) {
            (&None, &None, &None, &None, &None) => true,
            _ => false,
        }
    }
}

pub fn write_table_header<W: io::Write> (
        w: &mut W,
        fmt_set: &FmtSettings,
        settings: &Settings) {
    let mut header = String::new();
    header.write_str("|  | DONE TEST | ARTIFACT NAME").unwrap();
    for _ in 0..33 {
        header.push(' ');
    }
    if fmt_set.parts {
        header.write_str("| PARTS   ").unwrap();
    }
    if fmt_set.partof {
        header.write_str("| PARTOF   ").unwrap();
    }
    if fmt_set.loc_path {
        header.write_str("| IMPLEMENTED   ").unwrap();
    }
    if fmt_set.path {
        header.write_str("| DEFINED   ").unwrap();
    }
    if fmt_set.text {
        header.write_str("| TEXT").unwrap();
    }
    header.push('\n');
    if settings.color {
        write!(w, "{}", Style::new().bold().paint(header)).unwrap();
    } else {
        write!(w, "{}", header).unwrap();
    }
}
