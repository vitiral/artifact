/*  rst: the requirements tracking tool made for developers
 * Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
use dev_prefix::*;
use super::types::*;
use core::save::PathDiff;
use core::types::ProjectText;
use core::security;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("fmt")
        .about("format your design documents")
        //.arg(Arg:with_name("diff")
        //     .short("d")
        //     .help("only get the diff printed to stdout"))
        .arg(Arg::with_name("list")
             .short("l")
             .help("list files that will be affected and exit"))
        .arg(Arg::with_name("diff")
             .short("d")
             .help("print out the diff stdout and exit"))
        .arg(Arg::with_name("write")
             .short("w")
             .help("If a file's formatting is different from rst fmt,
                    overwrite it with rst fmt's vesion"))
        .settings(&[AS::DeriveDisplayOrder, COLOR])
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
        Err(ErrorKind::CmdError("must give one option: -l, -d, -w".to_string()).into())
    }
}


/// format the toml files (or just print diffs)
/// partof: #SPC-fmt
pub fn run_cmd(w: &mut Write, repo: &Path, project: &Project, cmd: &Cmd) -> Result<()> {
    let ptext = ProjectText::from_project(project)?;
    let indent = if *cmd == Cmd::Diff {
        // str.repeat would be great....
        (0..50).map(|_| "#").collect::<String>() + "\n# "
    } else {
        "".to_string()
    };
    // check to make sure nothing has actually changed
    // see: TST-fmt
    let fmt_project = core::process_project_text(&ptext).chain_err(
        || "internal fmt error: could not process project text.".to_string())?;
    project.equal(&fmt_project)
        .chain_err(|| "internal fmt error: formatted project has different data.".to_string())?;
    security::validate(repo, project)?;
    match *cmd {
        Cmd::List | Cmd::Diff => {
            // just list the files that will change
            let project_diff = ptext.diff()?;
            for (path, value) in project_diff {
                match value {
                    PathDiff::NotUtf8 => {
                        return Err(ErrorKind::InvalidUnicode(format!("{}", path.display())).into())
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
                        let header =
                            Style::new().bold().paint(format!("{}{}", indent, path.display()));
                        write!(w, "{}\n{}", header, disp)?;
                    }
                }
            }
            Ok(())
        }
        Cmd::Write => {
            // dump the ptext and then make sure nothing changed...
            ptext.dump()?;
            let new_project = match core::load_path(&project.origin) {
                Ok(p) => p,
                Err(err) => {
                    // see: TST-fmt
                    error!("Something went horribly wrong! Your project may be
                            deleted and I'm really sorry! Please investigate
                            and open a ticket :( :( :(");
                    return Err(err);
                }
            };
            // see: TST-fmt
            if let Err(err) = project.equal(&new_project) {
                error!("we tried formatting your project but something went
                        wrong and it has changed. We are very sorry :( :( \n
                        Please investigate and open a ticket, then you can
                        hopefully revert your design and .rst folders back using
                        `git checkout .rst design`");
                Err(err)
            } else {
                info!("fmt was successful");
                Ok(())
            }
        }
    }
}
