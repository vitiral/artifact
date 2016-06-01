//! cmdline: utilities and methods to display information on the
//! command line
//!
//! this module contains trait implementations and other sugar
//! to help with displaying rsk artifacts on the cmd line
//! as well as functions which map easily to cmdline methods
//! that the user may want to execute

use std::env;
use std::collections::HashSet;
use std::path::Path;
use std::iter::FromIterator;

use core;

use log;
#[cfg(not(test))] use fern;
use clap::ArgMatches;

mod matches;
mod ls;
mod fmt;

#[cfg(not(test))]
fn init_logger(quiet: bool, verbosity: u8) {
    let level = if quiet {log::LogLevelFilter::Off } else {
        match verbosity {
            0 => log::LogLevelFilter::Warn,
            1 => log::LogLevelFilter::Info,
            2 => log::LogLevelFilter::Debug,
            3 => log::LogLevelFilter::Trace,
            _ => unreachable!(),
        }
    };
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("{}: {}", level, msg)
        }),
        output: vec![fern::OutputConfig::stderr()],
        level: level,
    };
    fern::init_global_logger(logger_config, log::LogLevelFilter::Trace).unwrap();
}

#[cfg(test)]
fn init_logger(quiet: bool, verbosity: u8) {}


pub fn get_loglevel(matches: &ArgMatches) -> Option<(u8, bool)> {
    let verbosity = match matches.occurrences_of("v") {
        v @ 0...3 => v,
        _ => {
            error!("verbosity cannot be higher than 3");
            return None;
        }
    } as u8;
    let quiet = matches.is_present("quiet");
    Some((verbosity, quiet))
}


pub fn cmd() {
    let matches = matches::get_matches();
    // initialze the logger
    match get_loglevel(&matches) {
        Some((v, q)) => init_logger(q, v),
        None => return,
    };

    // load the artifacts
    let mut repo_names = HashSet::new();
    repo_names.insert(".rsk".to_string());
    let cwd = env::current_dir().unwrap();
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
        let (names, fmtset) = ls::get_ls_cmd(&ls).unwrap();
        ls::do_ls(names, &artifacts, &fmtset, &settings);
    }
}

