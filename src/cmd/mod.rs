/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
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
//! cmdline: utilities and methods to display information on the
//! command line
//!
//! this module contains trait implementations and other sugar
//! to help with displaying artifact artifacts on the cmd line
//! as well as functions which map easily to cmdline methods
//! that the user may want to execute

use dev_prefix::*;
use types::*;
use logging;
use user;
use utils;
use security;

use clap::{ArgMatches, ErrorKind as ClEk};
use ansi_term::Colour::Green;

mod export;
mod types;
mod matches;
mod ls;
mod check;
mod display;
mod fmt;
mod init;
mod tutorial;

#[cfg(feature="server")]
mod server;

#[cfg(test)]
mod tests;

pub fn get_loglevel(matches: &ArgMatches) -> Option<(u8, bool)> {
    let verbosity = match matches.occurrences_of("verbose") {
        v @ 0...3 => v,
        _ => {
            println!("ERROR: verbosity cannot be higher than 3");
            return None;
        }
    } as u8;
    let quiet = matches.is_present("quiet");
    Some((verbosity, quiet))
}

#[cfg(feature="server")]
fn run_server(project: &Project, matches: &ArgMatches) -> Result<u8> {
    if let Some(mat) = matches.subcommand_matches("serve") {
        let addr = server::get_cmd(mat);
        server::run_cmd(project.clone(), &addr);
        Ok(0)
    } else {
        Err(ErrorKind::NothingDone.into())
    }
}

#[cfg(not(feature="server"))]
fn run_server(_: &Project, _: &ArgMatches) -> Result<u8> {
    Err(ErrorKind::NothingDone.into())
}

pub fn cmd<W, I, T>(w: &mut W, args: I) -> Result<u8>
    where I: IntoIterator<Item = T>,
          T: Into<OsString> + clone::Clone,
          W: io::Write
{
    let matches = match matches::get_matches(args) {
        Ok(m) => m,
        Err(e) => {
            match e.kind {
                ClEk::HelpDisplayed | ClEk::VersionDisplayed => {
                    print!("{}", e);
                    return Ok(0);
                }
                _ => return Err(ErrorKind::CmdError(e.to_string()).into()),
            }
        }
    };

    // initialze the logger
    match get_loglevel(&matches) {
        Some((v, q)) => logging::init_logger(q, v, true).unwrap(),
        None => panic!("could not find loglevel"),
    };

    let cwd = env::current_dir().unwrap();
    let work_tree = match matches.value_of("work-tree") {
        Some(w) => PathBuf::from(w),
        None => cwd.to_path_buf(),
    };
    if !work_tree.is_dir() {
        let msg = format!("ERROR: work-tree {} is not a directory",
                          work_tree.display());
        return Err(ErrorKind::CmdError(msg).into());
    }

    // If init is selected, do that
    if matches.subcommand_matches("init").is_some() {
        info!("Calling the init command");
        init::run_cmd(&work_tree)?;
        return Ok(0);
    }

    // If tutorial is selected, do that
    if let Some(t) = matches.subcommand_matches("tutorial") {
        info!("Calling the tutorial command");
        let c = tutorial::get_cmd(t)?;
        tutorial::run_cmd(&work_tree, c).unwrap();
        return Ok(0);
    }

    // load the artifacts
    let repo = match utils::find_repo(&work_tree) {
        Ok(r) => r,
        Err(_) => {
            let msg = "Could not find .art folder. Try running `art init`";
            return Err(ErrorKind::CmdError(msg.to_string()).into());
        }
    };
    debug!("using repo dir {:?}", repo);

    let project = user::load_repo(&repo)?;

    // SPC-security: do security checks on the project
    security::validate(&repo, &project)?;

    debug!("settings={:?}", project.settings);

    if let Some(ls) = matches.subcommand_matches("ls") {
        info!("Calling the ls command");
        let cmd = ls::get_cmd(ls).unwrap();
        ls::run_cmd(w, &work_tree, &cmd, &project)
    } else if matches.subcommand_matches("check").is_some() {
        info!("Calling the check command");
        let cmd = check::Cmd { color: types::COLOR_IF_POSSIBLE };
        check::run_cmd(w, &work_tree, &project, &cmd)
    } else if let Some(mat) = matches.subcommand_matches("fmt") {
        info!("Calling the fmt command");
        let c = fmt::get_cmd(mat)?;
        fmt::run_cmd(w, &repo, &project, &c)
    } else if let Some(mat) = matches.subcommand_matches("export") {
        info!("Calling the export command");
        let c = export::get_cmd(mat)?;
        export::run_cmd(&cwd, &project, &c)
    } else if match run_server(&project, &matches) {
                  Ok(r) => return Ok(r),
                  Err(err) => {
                      match *err.kind() {
                          ErrorKind::NothingDone => false,
                          _ => return Err(err),
                      }
                  }
              } {
        unreachable!();
    } else {
        write!(w,
               "{} {}: use -h to show help",
               Green.bold().paint("artifact"),
               Green.paint(VERSION))
                .unwrap();
        return Ok(0);
    }
}
