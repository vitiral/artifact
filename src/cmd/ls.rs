/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use tabwriter::TabWriter;

use dev_prefix::*;
use types::*;
use cmd::types::*;
use cmd::display;
use export;

/// Get the ls subcommand, which is what creates the command
/// for the cmdline
/// see: SPC-cmd-ls
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("ls")
        .about("List artifacts according to various parameters")
        .settings(&SUBCMD_SETTINGS)
        .arg(
            Arg::with_name("search")
                .value_name("SEARCH")
                .help(
                    "Artifacts to search for. By default this is the full artifact name \
                    (i.e. REQ-foo-bar).\n\n\
                    If `-p FIELDS` is passed, SEARCH is the regexp and FIELDS \
                    are the fields to search.\n\
                    - N/name: search the \"name\" field (artifact name)\n\
                    - D/def: search the \"def\" field (see -D) \n\
                    - P/parts: search the \"parts\" field (see -P)\n\
                    - O/partof: search the \"partof\" field (see -O)\n\
                    - L/loc: search the \"loc\" field (see -L)\n\
                    - T/text: search the text field (see -T)\n\n\

                    Fields can be listed by all caps, or comma-separated lowercase. \
                    Both of these commands will list only artifacts with \"foobar\" \
                    in the name or text fields of all artifacts.\n    \
                    art ls foobar -p NT\n    \
                    art ls foobar -p name,text\n\n\

                    Regular expressions use the rust regular expression syntax, \
                    which is almost identical to perl/python with a few minor differences\n\
                    https://doc.rust-lang.org/regex/regex/index.html#syntax.\n\

                    ",
                )
                .use_delimiter(false),
        )
        .arg(
            Arg::with_name("pattern")
                .short("p")
                .long("pattern")
                .help("Filter FIELDS using SEARCH as a regexp. See SEARCH docs")
                .value_name("FIELDS")
                .takes_value(true)
                .max_values(1)
                .min_values(0),
        )
        .arg(
            Arg::with_name("long")
                .short("l")
                .long("long")
                .help("Print items in the 'long form'"),
        )
        .arg(
            Arg::with_name("completed")
                .value_name("COMPLETED")
                .short("c")
                .long("completed")
                .help(
                    "Filter by completeness (e.g. '<45'), < and > are inclusive. \
                     < is a shortcut for 0%, and > is a shortcut for 100%",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("tested")
                .value_name("TESTED")
                .short("t")
                .help("Filter by testedness in percent. See '-c'")
                .takes_value(true),
        )
        .arg(Arg::with_name("all").short("A").long("all").help(
            "\"all\" fields. If set, additional field flags will *deactivate* fields",
        ))
        .arg(
            Arg::with_name("def")
                .short("D")
                .long("def")
                .help("\"def\" field: file where the arfiact is defined"),
        )
        .arg(
            Arg::with_name("parts")
                .short("P")
                .long("parts")
                .help("\"parts\" field: children of the artifact"),
        )
        .arg(
            Arg::with_name("partof")
                .short("O")
                .long("partof")
                .help("\"partof\" field: parents of the artifact"),
        )
        .arg(Arg::with_name("loc").short("L").long("loc").help(
            "\"location\" field: code location where this artifact is implemented",
        ))
        .arg(Arg::with_name("text").short("T").long("text").help(
            "\"text\" field: use -l to display full text, otherwise just first line.",
        ))
        .arg(
            Arg::with_name("plain")
                .long("plain")
                .help("Do not display color in the output"),
        )
        .arg(
            Arg::with_name("type")
                .long("type")
                .value_name("TYPE")
                .takes_value(true)
                .help("Output type, default 'list'. Supported types: list, json"),
        )
    //.arg(Arg::with_name("file")
    //    .long("file")
    //    .takes_value(true)
    //    .help("Output to file instead of stdout"))
}

/// return (lt, percent) returning None when there is no value
pub fn _get_percent(s: &str) -> result::Result<(Option<bool>, Option<i8>), String> {
    let mut s = s;
    let mut lt = None;
    if s.is_empty() {
        return Ok((lt, None));
    }
    let mut had_sign = true;
    match s.chars().next().unwrap() {
        '<' => lt = Some(true),
        '>' => lt = Some(false),
        '0'...'9' => had_sign = false,
        _ => {
            return Err(
                "percent must be of the form: [SIGN]NUM where NUM is between 0 and 100 and \
                 SIGN is an optional < or >"
                    .to_string(),
            )
        }
    }
    if had_sign {
        // the first char was either < or >
        s = s.split_at(1).1;
        if s.is_empty() {
            return Ok((lt, None));
        }
    }
    if s.is_empty() {
        return Ok((lt, None));
    }
    match s.parse::<i8>() {
        Ok(v) => if v <= 100 && v >= -100 {
            Ok((lt, Some(v)))
        } else {
            Err("NUM must be between -100 and 100".to_string())
        },
        Err(e) => Err(e.to_string()),
    }
}

fn get_percent(s: &str) -> result::Result<PercentSearch, String> {
    Ok(match _get_percent(s) {
        Ok((lt, perc)) => if lt.is_none() && perc.is_none() {
            PercentSearch {
                lt: false,
                perc: 100,
            }
        } else if perc.is_none() {
            if lt.unwrap() {
                PercentSearch { lt: true, perc: 0 }
            } else {
                PercentSearch {
                    lt: false,
                    perc: 100,
                }
            }
        } else {
            let lt = match lt {
                None => false,
                Some(l) => l,
            };
            let perc = match perc {
                None => 100,
                Some(p) => p,
            };
            PercentSearch { lt: lt, perc: perc }
        },
        Err(e) => return Err(e),
    })
}

#[test]
fn test_get_percent() {
    // correct
    assert_eq!(_get_percent(""), Ok((None, None)));
    assert_eq!(_get_percent("<"), Ok((Some(true), None)));
    assert_eq!(_get_percent(">"), Ok((Some(false), None)));
    assert_eq!(_get_percent("<10"), Ok((Some(true), Some(10))));
    assert_eq!(_get_percent(">100"), Ok((Some(false), Some(100))));
    assert_eq!(_get_percent(">-100"), Ok((Some(false), Some(-100))));

    // test full struct
    assert_eq!(
        get_percent(""),
        Ok(PercentSearch {
            lt: false,
            perc: 100,
        })
    );
    assert_eq!(get_percent("<"), Ok(PercentSearch { lt: true, perc: 0 }));
    assert_eq!(
        get_percent(">"),
        Ok(PercentSearch {
            lt: false,
            perc: 100,
        })
    );
    assert_eq!(
        get_percent("89"),
        Ok(PercentSearch {
            lt: false,
            perc: 89,
        })
    );
    assert_eq!(
        get_percent(">89"),
        Ok(PercentSearch {
            lt: false,
            perc: 89,
        })
    );
    assert_eq!(get_percent("<89"), Ok(PercentSearch { lt: true, perc: 89 }));
    assert_eq!(
        get_percent(">-1"),
        Ok(PercentSearch {
            lt: false,
            perc: -1,
        })
    );

    // invalid
    assert!(get_percent(">101").is_err());
    assert!(get_percent("a").is_err());
    assert!(get_percent("<a").is_err());
}

#[cfg(not(windows))]
fn get_color(matches: &ArgMatches) -> bool {
    !matches.is_present("plain")
}

#[cfg(windows)]
fn get_color(matches: &ArgMatches) -> bool {
    false
}

#[derive(Debug, Eq, PartialEq)]
pub enum OutType {
    List, // default
    Json,
}

#[derive(Debug)]
pub struct Cmd {
    pub pattern: String,
    pub fmt_settings: FmtSettings,
    pub search_settings: SearchSettings,
    pub ty: OutType,
}

/// get all the information from the user input
pub fn get_cmd(matches: &ArgMatches) -> Result<Cmd> {
    let mut fmt_set = FmtSettings::default();
    fmt_set.long = matches.is_present("long");
    // fmt_set.recurse = matches.value_of("recursive").unwrap().parse::<u8>().unwrap();
    fmt_set.def = matches.is_present("def");
    fmt_set.parts = matches.is_present("parts");
    fmt_set.partof = matches.is_present("partof");
    fmt_set.loc_path = matches.is_present("loc");
    fmt_set.text = matches.is_present("text");
    fmt_set.color = get_color(matches);
    // #SPC-cmd-ls-display
    if matches.is_present("all") {
        // reverse everything
        fmt_set.def = !fmt_set.def;
        fmt_set.parts = !fmt_set.parts;
        fmt_set.partof = !fmt_set.partof;
        fmt_set.loc_path = !fmt_set.loc_path;
        fmt_set.text = !fmt_set.text;
    } else if fmt_set.long &&
        !(fmt_set.def || fmt_set.parts || fmt_set.partof || fmt_set.loc_path || fmt_set.text)
    {
        // if long is specified but no other display attributes are specified
        fmt_set.def = true;
        fmt_set.parts = true;
        fmt_set.partof = true;
        fmt_set.loc_path = true;
        fmt_set.text = true;
    }

    // #SPC-cmd-ls-pattern
    let mut search_set = match (matches.is_present("pattern"), matches.value_of("pattern")) {
        (true, Some(p)) => SearchSettings::from_str(p)?,
        (true, None) => SearchSettings::from_str("N").unwrap(),
        (false, None) => SearchSettings::default(),
        _ => unreachable!(),
    };
    if let Some(c) = matches.value_of("completed") {
        search_set.completed = try!(get_percent(c))
    }
    if let Some(t) = matches.value_of("tested") {
        debug!("got tested: {}", t);
        search_set.tested = try!(get_percent(t));
    }

    let ty = match matches.value_of("type").unwrap_or("list") {
        "list" => OutType::List,
        "json" => OutType::Json,
        t => {
            let msg = format!("Invalid type: {}", t);
            return Err(ErrorKind::CmdError(msg).into());
        }
    };

    let cmd = Cmd {
        pattern: matches.value_of("search").unwrap_or("").to_string(),
        fmt_settings: fmt_set,
        search_settings: search_set,
        ty: ty,
    };
    debug!("ls search: {:?}", cmd);
    Ok(cmd)
}

#[allow(trivial_regex)]
/// perform the ls command given the inputs
pub fn run_cmd<W: Write>(w: &mut W, cwd: &Path, cmd: &Cmd, project: &Project) -> Result<u8> {
    let mut dne: Vec<NameRc> = Vec::new();
    let artifacts = &project.artifacts;
    let mut fmt_set = cmd.fmt_settings.clone();

    // no color when exporting
    if cmd.ty != OutType::List {
        fmt_set.color = false;
    }

    // get the names -- they will be filtered next
    let mut names: Vec<_> = if cmd.search_settings.use_regex || cmd.pattern.is_empty() {
        let mut names: Vec<_> = artifacts.keys().cloned().collect();
        names.sort();
        names
    } else {
        // names are exactly specified according to the partof syntax
        let want_names = match Names::from_str(&cmd.pattern) {
            Ok(n) => n,
            Err(e) => {
                error!("{}", e);
                return Err(ErrorKind::CmdError(format!("{}", e)).into());
            }
        };
        let mut names = Vec::new();
        for n in want_names {
            if artifacts.contains_key(n.as_ref()) {
                names.push(n);
            } else {
                dne.push(n)
            }
        }
        dne.sort();
        names.sort();
        names
    };

    // filter by various settings (not just pattern, also test/completeness %, etc)
    let names: Vec<_> = {
        let pat = if cmd.search_settings.use_regex {
            let p = RegexBuilder::new(&cmd.pattern)
                .case_insensitive(true)
                .build();
            match p {
                Ok(p) => p,
                Err(e) => {
                    return Err(
                        ErrorKind::CmdError(format!("Invalid pattern: {}", e)).into(),
                    );
                }
            }
        } else {
            Regex::new("").unwrap()
        };
        names
            .drain(0..)
            .filter(|n| {
                let a = artifacts.get(n).unwrap(); // we are guaranteed the name exists
                ui::show_artifact(n, a, &pat, &cmd.search_settings)
            })
            .collect()
    };
    debug!("Artifact names selected: {:?}", names);
    if fmt_set.is_empty() {
        fmt_set.parts = true;
    }

    let mut displayed = Names::new();
    match cmd.ty {
        OutType::List => {
            let mut tw = TabWriter::new(w);
            if !names.is_empty() && !fmt_set.long {
                display::write_table_header(&mut tw, &fmt_set);
            }
            for name in names {
                let f =
                    ui::fmt_artifact(&name, artifacts, &fmt_set, fmt_set.recurse, &mut displayed);
                f.write(&mut tw, cwd, artifacts, fmt_set.color, 0)?;
            }
            tw.flush()?; // this is necessary for actually writing the output
        }
        OutType::Json => {
            w.write_all(
                export::project_artifacts_to_json(project, Some(&names)).as_bytes(),
            )?;
        }
    }
    if !dne.is_empty() {
        let mut msg = format!("{:?}", dne);
        if !cmd.search_settings.use_regex {
            msg.push_str("\nHelp: consider using the -p / --pattern flag");
        }
        return Err(ErrorKind::NameNotFound(msg).into());
    }
    Ok(0)
}
