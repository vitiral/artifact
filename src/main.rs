
// # general crates
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate fern;

// # core crates
// LOC-core-loading-toml
// LOC-core-vars-lib
extern crate regex;
extern crate strfmt;
extern crate time;
extern crate toml;

// # ui-cmdline crates
extern crate clap;


use std::path::Path;

use clap::{Arg, App, ArgMatches};

pub mod core;
mod cmdline;

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
        output: vec![fern::OutputConfig::stdout()],
        level: level,
    };
    fern::init_global_logger(logger_config, log::LogLevelFilter::Trace).unwrap();
}

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("rsk")
        .version("0.0.1")
        .about("the requirements tracking tool made for developers")
        .arg(Arg::with_name("path")
             .value_name("PATH")
             .help("load from *.rsk file or from a directory recursively")
             .required(true))
        .arg(Arg::with_name("names")
             .value_name("NAMES")
             .help("artifact names to look up, specified in the form \
                    `REQ-foo-[bar, baz-[1,2]], SPC-foo-bar`, etc")
             .required(false))

        .arg(Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("sets the level of verbosity, use multiple (up to 3) to increase"))
        .arg(Arg::with_name("quiet")
             .short("q")
             .long("quiet")
             .help("if set no output will be printed"))
        .get_matches()
}

fn cmd() {
    let matches = get_matches();
    // initialze the logger
    let verbosity = match matches.occurrences_of("v") {
        v @ 0...3 => v,
        _ => {
            error!("verbosity cannot be higher than 3");
            return;
        }
    };
    let quiet = matches.is_present("quiet");
    init_logger(quiet, verbosity as u8);

    // load the artifacts
    let path = matches.value_of("path").unwrap();
    debug!("loading path: {:?}", path);
    let (artifacts, settings) = match core::load_path(Path::new(path)) {
        Ok(v) => v,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };
}


fn main() {
    cmd();
}
