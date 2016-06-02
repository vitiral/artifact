use std::io;
use std::io::Write;
use std::process::exit;
use std::collections::HashSet;

use clap::{Arg, App, SubCommand, ArgMatches, AppSettings as AS};

use core::fmt::{FmtSettings, fmt_artifact};
use cmdline::fmt::FmtArtifact;
use core::{Artifacts, ArtName, parse_names, Settings};


pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("ls")
        .about("list artifacts according to various parameters")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
        .arg(Arg::with_name("artifacts")
                .help("artifact names given in form REQ-foo-[bar, baz-[1,2]]")
                .use_delimiter(false))
        .arg(Arg::with_name("long")
                .short("l")
                .help("print items in the 'long form'"))
        .arg(Arg::with_name("recursive")
                .short("r")
                .help("print the parts of the artifact up to the given depth (default 1)")
                .value_name("DEPTH")
                .takes_value(true)
                .validator(|s| match s.parse::<u8>() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                })
                .default_value("0")
                .max_values(1))
        .arg(Arg::with_name("all")
                .short("A")
                .help("activate all display flags. If this flag is set, additional flags will \
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
        .arg(Arg::with_name("loc")
                .short("L")
                .help("display location name"))
        .arg(Arg::with_name("implemented")
                .short("I")
                .help("display the path where the artifact is implemented (LOC path)"))
        .arg(Arg::with_name("refs")
                .short("R")
                .help("display the references to this artifact"))
        .arg(Arg::with_name("text")
                .short("T")
                .help("display the text description of this artifact \
                        (first line only if not -l)"))
}


/// get all the information from the user input
pub fn get_ls_cmd(matches: &ArgMatches) -> Result<(Vec<ArtName>, FmtSettings), String> {
    let mut settings = FmtSettings::default();
    settings.long = matches.is_present("long");
    settings.recurse = matches.value_of("recursive").unwrap().parse::<u8>().unwrap();
    settings.path = matches.is_present("path");
    settings.parts = matches.is_present("parts");
    settings.partof = matches.is_present("partof");
    settings.loc_path = matches.is_present("implemented");
    settings.refs = matches.is_present("refs");
    settings.text = matches.is_present("text");
    if matches.is_present("all") {
        // reverse everything
        settings.path = !settings.path;
        settings.parts = !settings.parts;
        settings.partof = !settings.partof;
        settings.loc_path = !settings.loc_path;
        settings.refs = !settings.refs;
        settings.text = !settings.text;
    } else if settings.long && !(settings.path || settings.parts || settings.partof ||
                                 settings.loc_path || settings.refs || settings.text) {
        // if long is specified but no other display attributes are specified
        settings.path = true;
        settings.parts = true;
        settings.refs = true;
        settings.text = true;
    }

    let names = matches.value_of("artifacts").unwrap_or("");
    let mut artifacts = Vec::new();
    artifacts.extend(parse_names(names).unwrap());
    artifacts.sort();

    debug!("artifacts: {:?}", artifacts);
    debug!("ls settings: {:?}", settings);
    Ok((artifacts, settings))
}


pub fn do_ls(names: Vec<ArtName>, artifacts: &Artifacts, fmtset: &FmtSettings, settings: &Settings) {
    let mut dne: Vec<ArtName> = Vec::new();
    let mut names = names.clone();
    if names.len() == 0 {
        names.extend(artifacts.keys().map(|n| n.clone()));
    }
    names.sort();
    let mut displayed: HashSet<ArtName> = HashSet::new();
    let mut stdout = io::stdout();
    for name in names {
        let artifact = match artifacts.get(&name) {
            Some(a) => a,
            None => {
                dne.push(name);
                continue;
            }
        };
        let f = fmt_artifact(&name, artifacts, fmtset, fmtset.recurse, &mut displayed);
        f.write(&mut stdout, artifacts, settings, 0).unwrap(); // FIXME: unwrap
    }
    if dne.len() > 0 {
        error!("The following artifacts do not exist: {:?}", dne);
        exit(1);
    }
}
