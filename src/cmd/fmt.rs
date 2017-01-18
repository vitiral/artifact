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
use core::save::{ProjectText, PathDiff};

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("fmt")
        .about("format your design documents")
        //.arg(Arg:with_name("diff")
        //     .short("d")
        //     .help("only get the diff printed to stdout"))
        .arg(Arg::with_name("list")
             .short("l")
             .help("only list files that will be affected"))
        .arg(Arg::with_name("write")
             .short("w")
             .help("If a file's formatting is different from rst fmt,
                    overwrite it with rst fmt's vesion.
                    NOTE: THIS COULD EAT YOUR LAUNDRY"))
        .settings(&[AS::DeriveDisplayOrder, COLOR])
}

pub enum Cmd {
    List,
    Write,
}

pub fn get_cmd(matches: &ArgMatches) -> Result<Cmd> {
    if matches.is_present("list") {
        Ok(Cmd::List)
        // } else if matches.is_present("diff") {
        //    Ok(Cmd::Diff)
    } else if matches.is_present("write") {
        Ok(Cmd::Write)
    } else {
        Err(ErrorKind::CmdError("must give one option: -l, -d, -w".to_string()).into())
    }
}


/// format the toml files (or just print diffs)
/// partof: #SPC-fmt
pub fn run_cmd(cfg: &Path, project: &Project, cmd: &Cmd) -> Result<()> {
    let ptext = ProjectText::from_project(project)?;
    match *cmd {
        Cmd::List => {
            // just list the files that will change
            let diff = ptext.diff()?;
            for (path, value) in diff {
                match value {
                    PathDiff::NotUtf8 => {
                        return Err(ErrorKind::InvalidUnicode(format!("{}", path.display())).into())
                    }
                    PathDiff::None => {}
                    _ => println!("{}", path.display()),
                }
            }
            Ok(())
        }
        Cmd::Write => {
            // dump the ptext and then make sure nothing changed...
            ptext.dump()?;
            let new_project = match core::load_path(cfg) {
                Ok(p) => p,
                Err(err) => {
                    error!("Something went horribly wrong! Your project may be
                            deleted and I'm really sorry! Please investigate
                            and open a ticket");
                    return Err(err);
                }
            };
            if let Err(err) = new_project.equal(project) {
                error!("we tried formatting your project but something went
                        wrong and it has changed. We are very sorry :( :( \n
                        Please investigate and open a ticket, then you can
                        revert your design and .rst folders back using
                        `git checkout .rst design`");
                Err(err)
            } else {
                info!("fmt was successful");
                Ok(())
            }
        }
    }
}
