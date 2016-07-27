use std::io;
use std::io::Write;
use std::fmt::Write as FmtWrite;
use std::iter::FromIterator;
use std::process::exit;
use std::collections::HashSet;

use regex::{Regex, RegexBuilder};
use clap::{Arg, App, SubCommand, ArgMatches, AppSettings as AS};

use core::fmt::{FmtSettings, fmt_artifact};
use core::{Artifacts, ArtName, parse_names, Settings};
use cmdline::search::{VALID_SEARCH_FIELDS, SearchSettings, show_artifact};


pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    // TODO: implement -c and -t
    SubCommand::with_name("ls")
        .about("list artifacts according to various parameters")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
        .arg(Arg::with_name("search")
                 .help("artifact names given in form REQ-foo-[bar, baz-[1,2]] OR regexp pattern \
                        if -P is given")
                 .use_delimiter(false))
        .arg(Arg::with_name("pattern")
                 .short("p")
                 .help("SEARCH using a pearl regexp pattern in the given FIELDS instead of \
                        searching by name. Valid areas are: N=name, D=path, P=parts, O=partof, \
                        L=loc, R=refs, T=text, A=see '-A'")
                 .value_name("FIELDS")
                 .takes_value(true)
                 .max_values(1))
        .arg(Arg::with_name("long")
                 .short("l")
                 .help("print items in the 'long form'"))
        .arg(Arg::with_name("recursive")
                 .short("r")
                 .help("print the parts of the artifact up to the given depth (default 1)")
                 .value_name("DEPTH")
                 .takes_value(true)
                 .validator(|s| {
                     match s.parse::<u8>() {
                         Ok(_) => Ok(()),
                         Err(e) => Err(e.to_string()),
                     }
                 })
                 .default_value("0")
                 .max_values(1))
        .arg(Arg::with_name("completed")
                 .short("c")
                 .help("give a filter for the completedness in %. I.e. '<45'. '<' is the default \
                        if no comparison operator is given, '<1' is the default if no args are \
                        given")
                 .takes_value(true)
                 .default_value("1")
                 .max_values(1))
        .arg(Arg::with_name("tested")
                 .short("t")
                 .help("give a filter for the testedness in %. see '-c'")
                 .takes_value(true)
                 .default_value("1")
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
        .arg(Arg::with_name("refs")
                 .short("R")
                 .help("display the references to this artifact"))
        .arg(Arg::with_name("text")
                 .short("T")
                 .help("display the text description of this artifact (first line only if not -l)"))
}


/// get all the information from the user input
pub fn get_ls_cmd(matches: &ArgMatches)
                  -> Result<(String, FmtSettings, Option<SearchSettings>), String> {
    let mut settings = FmtSettings::default();
    settings.long = matches.is_present("long");
    settings.recurse = matches.value_of("recursive").unwrap().parse::<u8>().unwrap();
    settings.path = matches.is_present("path");
    settings.parts = matches.is_present("parts");
    settings.partof = matches.is_present("partof");
    settings.loc_path = matches.is_present("loc");
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
    } else if settings.long &&
       !(settings.path || settings.parts || settings.partof || settings.loc_path ||
         settings.refs || settings.text) {
        // if long is specified but no other display attributes are specified
        settings.path = true;
        settings.parts = true;
        settings.partof = true;
        settings.refs = true;
        settings.text = true;
    }
    let search_settings = match matches.value_of("pattern") {
        Some(p) => Some(try!(SearchSettings::from_str(p))),
        None => None,
    };

    let search = matches.value_of("search").unwrap_or("").to_string();

    debug!("ls settings: {:?}", settings);
    Ok((search, settings, search_settings))
}

/// perform the ls command given the inputs
pub fn do_ls(search: String,
             artifacts: &Artifacts,
             fmtset: &FmtSettings,
             search_set: &Option<SearchSettings>,
             settings: &Settings) {
    let mut dne: Vec<ArtName> = Vec::new();
    let mut names = Vec::new();
    let mut pat: Option<Regex> = None;
    let mut pat_case: Option<Regex> = None;
    if search_set.is_none() {
        // names to use are determined from the beginning
        names.extend(parse_names(&search).unwrap());
        names.sort();
        debug!("artifact names selected: {:?}", names);
    } else {
        // names to use are determined by filtering
        // the regexp
        let pat = RegexBuilder::new(&search)
                      .case_insensitive(true)
                      .compile();
        pat_case = Some(match pat {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid pattern: {}", e.to_string());
                exit(1);
            }
        });
    }
    if names.len() == 0 {
        names.extend(artifacts.keys().map(|n| n.clone()));
        names.sort();
    }

    let mut displayed: HashSet<ArtName> = HashSet::new();
    let mut stdout = io::stdout();
    for name in names {
        let art = match artifacts.get(&name) {
            Some(a) => a,
            None => {
                dne.push(name);
                continue;
            }
        };
        if let &Some(ref ss) = search_set {
            let pc = pat_case.as_ref().unwrap();
            if !show_artifact(&name, art, pc, ss) {
                continue;
            }
        }
        let f = fmt_artifact(&name, artifacts, fmtset, fmtset.recurse, &mut displayed);
        f.write(&mut stdout, artifacts, settings, 0).unwrap(); // FIXME: unwrap
    }
    if dne.len() > 0 {
        error!("The following artifacts do not exist: {:?}", dne);
        exit(1);
    }
}
