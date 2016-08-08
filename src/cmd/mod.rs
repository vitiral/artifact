//! cmdline: utilities and methods to display information on the
//! command line
//!
//! this module contains trait implementations and other sugar
//! to help with displaying rsk artifacts on the cmd line
//! as well as functions which map easily to cmdline methods
//! that the user may want to execute

use std::env;
use std::io;
use std::ffi::OsString;

use core;
use super::VERSION;

use clap::ArgMatches;
use ansi_term::Colour::Green;

mod types;
mod matches;
mod ls;
mod fmt;
mod init;
mod tutorial;
mod data;  // data mostly for the tutorial

#[cfg(test)]
mod tests;

use super::init_logger;

pub fn get_loglevel(matches: &ArgMatches) -> Option<(u8, bool)> {
    let verbosity = match matches.occurrences_of("v") {
        v @ 0...3 => v,
        _ => {
            println!("ERROR: verbosity cannot be higher than 3");
            return None;
        }
    } as u8;
    let quiet = matches.is_present("quiet");
    Some((verbosity, quiet))
}


pub fn cmd<'a, W, I, T>(w: &mut W, args: I)
    where I: IntoIterator<Item=T>,
          T: Into<OsString>,
          W: io::Write {
    let matches = match matches::get_matches(args) {
        Ok(m) => m,
        Err(e) => {
            write!(w, "{}", e).unwrap();
            return;
        }
    };

    // initialze the logger
    match get_loglevel(&matches) {
        Some((v, q)) => init_logger(q, v, true).unwrap(),
        None => return,
    };

    // If init is selected, do that
    let cwd = env::current_dir().unwrap();
    if let Some(_) = matches.subcommand_matches("init") {
        info!("Calling the init command");
        match init::do_init(&cwd) {
            Ok(_) => {},
            Err(e) => println!("ERROR: {}", e),
        }
        return;
    }

    // If tutorial is selected, do that
    if let Some(t) = matches.subcommand_matches("tutorial") {
        info!("Calling the tutorial command");
        let c = match tutorial::get_tutorial_cmd(t) {
            Ok(c) => c,
            Err(e) => {
                println!("ERROR: {}", e);
                return;
            },
        };
        tutorial::do_tutorial(c).unwrap();
        // match tutorial::do_tutorial(c) {
        //     Ok(_) => {},
        //     Err(e) => println!("ERROR: {}", e),
        // }
        return;
    }

    // load the artifacts
    let repo = match core::find_repo(cwd.as_path()) {
        Some(r) => r,
        None => {
            println!("Could not find .rsk folder. Try running `rsk init -t`");
            return;
        }
    };
    let cfg = repo.join(".rsk");
    debug!("using cfg dir {:?}", cfg);

    let (artifacts, settings) = match core::load_path(cfg.as_path()) {
        Ok(v) => v,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };

    if let Some(ls) = matches.subcommand_matches("ls") {
        info!("Calling the ls command");
        let (search, fmtset, search_set) = ls::get_ls_cmd(&ls).unwrap();
        ls::do_ls(w, &search, &artifacts, &fmtset, &search_set, &settings);
    } else {
        write!(w, "{} {}: use -h to show help",
               Green.bold().paint("rsk"),
               Green.paint(VERSION)).unwrap();
    }
}

