//! cmdline: utilities and methods to display information on the
//! command line
//!
//! this module contains trait implementations and other sugar
//! to help with displaying rsk artifacts on the cmd line
//! as well as functions which map easily to cmdline methods
//! that the user may want to execute

use std::path::Path;
use std::iter::FromIterator;

use clap::{Arg, App, SubCommand, ArgMatches};

use core;

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

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("rsk")
        .version("0.0.1")
        .about("the requirements tracking tool made for developers")
        .arg(Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("sets the level of verbosity, use multiple (up to 3) to increase")
             .global(true))
        .arg(Arg::with_name("quiet")
             .short("q")
             .long("quiet")
             .help("if set no output will be printed")
             .global(true))
        .subcommand(
            SubCommand::with_name("init")
                .about("initiailze the current directory and get help")
                .arg(Arg::with_name("path")
                     .value_name("PATH")
                     .help("initialzie the path, default is cwd")
                     .required(false))
        )
        .subcommand(SubCommand::with_name("ls")
            .about("list artifacts according to various parameters")
            .arg(Arg::with_name("artifacts")
                    .help("artifact names given in form REQ-foo-[bar, baz-[1,2]]")
                    .min_values(0))
            .arg(Arg::with_name("long")
                    .short("l")
                    .help("print items in the 'long form'"))
            .arg(Arg::with_name("recursive")
                    .help("print the parts of the artifact up to the given depth (default 1)")
                    .value_name("DEPTH")
                    .takes_value(true)
                    // .validator // TODO: make sure it is int
                    .default_value("0")
                    .max_values(1))
            .arg(Arg::with_name("all")
                 .short("A")
                 .help("activate all display flags. If this flag is set, specified flags will \
                        be *deactivated* instead of activated"))
            .arg(Arg::with_name("path")
                    .short("D")
                    .help("display the path where the artifact is defined"))
            .arg(Arg::with_name("parts")
                    .short("P")
                    .help("display the parts of the artifact"))
            .arg(Arg::with_name("partof")
                    .short("O")
                    .help("display the artifacts which this artifact is a partof"))
            .arg(Arg::with_name("refs")
                    .short("R")
                    .help("display the references to this artifact"))
            .arg(Arg::with_name("text")
                    .short("T")
                    .help("display the text description of this artifact \
                           (first line only if not -l)"))
        )
        .get_matches()
}

pub fn cmd() {
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
    let mut art_vec = Vec::from_iter(artifacts.iter());
    art_vec.sort_by_key(|a| a.0);
    println!("Artifacts:");
    // for (i, value) in art_vec.iter().enumerate() {
    //     let (n, a) = *value;
    //     println!(" {:<4} | {}", i, fmt::artifact_line(&n, &a));
    // }
}

