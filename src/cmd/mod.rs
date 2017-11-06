/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
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
pub mod check;
mod display;
mod fmt;
mod init;
mod tutorial;
mod update;
#[cfg(feature = "beta")]
mod plugin;

mod server;

#[cfg(test)]
mod tests;

pub fn get_loglevel(matches: &ArgMatches) -> (u8, bool) {
    let verbosity = match matches.occurrences_of("verbose") {
        v @ 0...3 => v,
        _ => {
            // v > 3
            eprintln!("WARN: verbosity cannot be higher than 3, defaulting to 3");
            3
        }
    } as u8;
    let quiet = matches.is_present("quiet");
    (verbosity, quiet)
}

#[cfg(feature = "beta")]
/// run beta commands here
fn run_beta<W>(_project: Option<&Project>, matches: &ArgMatches, w: &mut W) -> Result<u8>
where
    W: io::Write,
{
    if let Some(mat) = matches.subcommand_matches("plugin") {
        info!("Calling plugin command");
        let c = plugin::get_cmd(mat)?;
        plugin::run_cmd(&c, w)
    } else {
        Err(ErrorKind::NothingDone.into())
    }
}

#[cfg(not(feature = "beta"))]
/// run beta commands in the `[#cfg(feature = "beta")]` function
fn run_beta<W>(_: Option<&Project>, _: &ArgMatches, _: &mut W) -> Result<u8>
where
    W: io::Write,
{
    Err(ErrorKind::NothingDone.into())
}

pub fn cmd<W, I, T>(w: &mut W, args: I) -> Result<u8>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + clone::Clone,
    W: io::Write,
{
    let matches = match matches::get_matches(args) {
        Ok(m) => m,
        Err(e) => match e.kind {
            ClEk::HelpDisplayed | ClEk::VersionDisplayed => {
                eprint!("{}", e);
                return Ok(0);
            }
            _ => return Err(ErrorKind::CmdError(e.to_string()).into()),
        },
    };

    // initialze the logger
    let (v, q) = get_loglevel(&matches);
    logging::init_logger(q, v, true).unwrap();

    // if we are updating, just do that and exit
    if let Some(up) = matches.subcommand_matches("update") {
        info!("Calling the update command");
        let cmd = update::get_cmd(up);
        return Ok(update::run_cmd(&cmd).expect("update failed"));
    }

    let cwd = env::current_dir().unwrap();
    let work_tree = match matches.value_of("work-tree") {
        Some(w) => PathBuf::from(w),
        None => cwd.to_path_buf(),
    };
    if !work_tree.is_dir() {
        let msg = format!(
            "ERROR: work-tree {} is not a directory",
            work_tree.display()
        );
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

    // If plugin is selected, run it.
    // NB: plugin is a BETA command
    if matches.subcommand_matches("plugin").is_some() {
        return run_beta(None, &matches, w);
    }

    // load the artifacts
    let repo = match utils::find_repo(&work_tree) {
        Ok(r) => r,
        Err(_) => {
            let msg = "Could not find .art folder. Try running `art init`";
            return Err(ErrorKind::CmdError(msg.to_string()).into());
        }
    };
    debug!("Using repo dir {:?}", repo);

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
        let cmd = check::Cmd {
            color: types::COLOR_IF_POSSIBLE,
        };
        check::run_cmd(w, &work_tree, &project, &cmd)
    } else if let Some(mat) = matches.subcommand_matches("fmt") {
        info!("Calling the fmt command");
        let c = fmt::get_cmd(mat)?;
        println!("## running fmt");
        fmt::run_cmd(w, &repo, &project, &c).unwrap();
        Ok(0)
    } else if let Some(mat) = matches.subcommand_matches("export") {
        info!("Calling the export command");
        let c = export::get_cmd(mat)?;
        export::run_cmd(&cwd, &project, &c)
    } else if let Some(mat) = matches.subcommand_matches("serve") {
        let addr = server::get_cmd(mat);
        server::run_cmd(project.clone(), &addr);
        Ok(0)
    } else if match run_beta(Some(&project), &matches, w) {
        Ok(r) => return Ok(r),
        Err(err) => match *err.kind() {
            ErrorKind::NothingDone => false,
            _ => return Err(err),
        },
    } {
        unreachable!();
    } else {
        write!(
            w,
            "{} {}: use -h to show help",
            Green.bold().paint("artifact"),
            Green.paint(VERSION)
        ).unwrap();
        return Ok(0);
    }
}
