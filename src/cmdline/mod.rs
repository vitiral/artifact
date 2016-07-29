//! cmdline: utilities and methods to display information on the
//! command line
//!
//! this module contains trait implementations and other sugar
//! to help with displaying rsk artifacts on the cmd line
//! as well as functions which map easily to cmdline methods
//! that the user may want to execute

use std::env;
use std::ffi::OsString;
use std::collections::HashSet;


use core;
use super::VERSION;

use log;
use fern;
use clap::{ArgMatches, ErrorKind};
use ansi_term::Colour::Green;

mod matches;
mod ls;
mod fmt;
mod search;
mod init;

#[cfg(tests)]
mod tests;

pub fn init_logger(quiet: bool, verbosity: u8, stderr: bool) -> Result<(), fern::InitError> {
    let level = if quiet {log::LogLevelFilter::Off } else {
        match verbosity {
            0 => log::LogLevelFilter::Warn,
            1 => log::LogLevelFilter::Info,
            2 => log::LogLevelFilter::Debug,
            3 => log::LogLevelFilter::Trace,
            _ => unreachable!(),
        }
    };
    let output = if stderr {
        fern::OutputConfig::stderr()
    } else {
        fern::OutputConfig::stdout()
    };

    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("{}: {}", level, msg)
        }),
        output: vec![output],
        level: level,
    };
    fern::init_global_logger(logger_config, log::LogLevelFilter::Trace)
}



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


pub fn cmd<'a, I, T>(args: I)
        where I: IntoIterator<Item=T>, T: Into<OsString> {
    let matches = match matches::get_matches(args) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    // initialze the logger
    match get_loglevel(&matches) {
        Some((v, q)) => init_logger(q, v, true).unwrap(),
        None => return,
    };

    // load the artifacts
    let mut repo_names = HashSet::new();
    repo_names.insert(".rsk".to_string());
    let cwd = env::current_dir().unwrap();
    if let Some(_) = matches.subcommand_matches("init") {
        info!("Calling the init command");
        match init::do_init(&cwd, &repo_names) {
            Ok(_) => {},
            Err(e) => println!("ERROR: {}", e),
        }
        return;
    }
    let repo = match core::find_repo(cwd.as_path(), &repo_names) {
        Some(r) => r,
        None => {
            println!("Could not find .rsk folder. Try running `rsk init`");
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
        ls::do_ls(search, &artifacts, &fmtset, &search_set, &settings);
    } else {
        println!("{} {}: use -h to show help", Green.bold().paint("rsk"), Green.paint(VERSION));
    }
}

