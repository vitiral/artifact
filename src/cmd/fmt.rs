/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use dev_prefix::*;
use types::*;
use cmd::types::*;
use user::{PathDiff, ProjectText};
use security;
use user;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("fmt")
        .about("Format your design documents")
        .settings(&SUBCMD_SETTINGS)
        //.arg(Arg:with_name("diff")
        //     .short("d")
        //     .help("Only get the diff printed to stdout"))
        .arg(Arg::with_name("list")
             .short("l")
             .help("List files that will be affected and exit"))
        .arg(Arg::with_name("diff")
             .short("d")
             .help("Print out the diff stdout and exit"))
        .arg(Arg::with_name("write")
             .short("w")
             .help("If a file's formatting is different from art fmt, \
                    overwrite it with art fmt's vesion"))
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Cmd {
    List,
    Diff,
    Write,
}

pub fn get_cmd(matches: &ArgMatches) -> Result<Cmd> {
    if matches.is_present("list") {
        Ok(Cmd::List)
    } else if matches.is_present("diff") {
        Ok(Cmd::Diff)
    } else if matches.is_present("write") {
        Ok(Cmd::Write)
    } else {
        Err(
            ErrorKind::CmdError("must give one option: -l, -d, -w".to_string()).into(),
        )
    }
}


/// format the toml files (or just print diffs)
/// partof: #SPC-cmd-fmt
pub fn run_cmd(w: &mut Write, repo: &Path, project: &Project, cmd: &Cmd) -> Result<u8> {
    let is_beta = FileType::Toml == project.settings.file_type;
    let ext = match project.settings.file_type {
        FileType::Toml => "toml",
        FileType::Markdown => "md",
    };
    let ptext = {
        let mut project = project.clone();
        for artifact in project.artifacts.values_mut() {
            artifact.def.set_extension(ext);
        }
        let mut files = HashSet::new();
        for path in &project.files {
            let mut path = path.clone();
            path.set_extension(ext);
            files.insert(path);
        }
        project.files = files;
        ProjectText::from_project(&project)?
    };
    let indent = if *cmd == Cmd::Diff {
        // str.repeat would be great....
        (0..50).map(|_| "#").collect::<String>() + "\n# "
    } else {
        "".to_string()
    };
    // check to make sure nothing has actually changed
    // see: TST-fmt
    let fmt_project = user::process_project_text(project.settings.clone(), &ptext).chain_err(|| {
        "internal fmt error: could not process project text.".to_string()
    })?;
    if !is_beta {
        project.equal(&fmt_project).chain_err(|| {
            "internal fmt error: formatted project has different data.".to_string()
        })?;
    }
    security::validate(repo, project)?;
    match *cmd {
        Cmd::List | Cmd::Diff => {
            // just list the files that will change
            let mut diff_detected = 0_u8;
            let project_diff = ptext.diff()?;
            for (path, value) in project_diff {
                match value {
                    PathDiff::NotUtf8 => {
                        return Err(
                            ErrorKind::InvalidUnicode(format!("{}", path.display())).into(),
                        )
                    }
                    PathDiff::None => {}
                    // TODO: need to handle case where file is deleted...
                    // neither of these should happen for our use case
                    PathDiff::DoesNotExist => {
                        panic!("unexpected new file: {}", path.display());
                    }
                    PathDiff::Changeset(changeset) => {
                        let disp = if *cmd == Cmd::Diff {
                            format!("{}", changeset)
                        } else {
                            "".to_string()
                        };
                        let header = Style::new()
                            .bold()
                            .paint(format!("{}{}", indent, path.display()));
                        write!(w, "{}\n{}", header, disp)?;
                        diff_detected = 1;
                    }
                }
            }
            Ok(diff_detected)
        }
        Cmd::Write => {
            // dump the ptext and then make sure nothing changed...
            ptext.dump()?;
            if is_beta {
                return Ok(0);
            }
            let new_project = match user::load_repo(&project.origin) {
                Ok(p) => p,
                Err(err) => {
                    // see: TST-fmt
                    error!(
                        "Something went horribly wrong! Your project may be
                            deleted and I'm really sorry! Please investigate
                            and open a ticket :( :( :("
                    );
                    return Err(err);
                }
            };
            // see: TST-fmt
            if let Err(err) = project.equal(&new_project) {
                error!(
                    "we tried formatting your project but something went
                        wrong and it has changed. We are very sorry :( :( \n
                        Please investigate and open a ticket, then you can
                        hopefully revert your design and .art folders back using
                        `git checkout .art design`"
                );
                Err(err)
            } else {
                info!("fmt was successful");
                Ok(0)
            }
        }
    }
}
