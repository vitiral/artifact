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
        .arg(Arg::with_name("search")
            .help("Artifact names given in the brace pattern form \
                   e.g. REQ-foo-[bar, baz-[1,2]] \
                   OR regexp pattern if -p is given. Regular expressions use \
                   the rust regular expression syntax \
                   https://doc.rust-lang.org/regex/regex/index.html#syntax")
            .use_delimiter(false))
        .arg(Arg::with_name("pattern")
            .short("p")
            .help("Search FIELDS using regexp SEARCH.")
            .value_name("FIELDS")
            .takes_value(true)
            .max_values(1)
            .min_values(0))
        .arg(Arg::with_name("long")
            .short("l")
            .help("Print items in the 'long form'"))
        .arg(Arg::with_name("completed")
            .value_name("COMPLETED")
            .short("c")
            .help("Filter by completeness (e.g. '<45'), < and > are inclusive. \
                   < is a shortcut for 0%, and > is a shortcut for 100%")
            .takes_value(true))
        .arg(Arg::with_name("tested")
            .value_name("TESTED")
            .short("t")
            .help("Filter by testedness in percent. See '-c'")
            .takes_value(true))
        .arg(Arg::with_name("all")
            .short("A")
            .help("If set, additional flags will be *deactivated* instead of activated"))
        .arg(Arg::with_name("path")
            .short("D")
            .help("Display the path where the artifact is defined"))
        .arg(Arg::with_name("parts")
            .short("P")
            .help("Display the parts of the artifact"))
        .arg(Arg::with_name("partof")
            .short("O")
            .help("Display the artifacts which this artifact is a partof"))
        .arg(Arg::with_name("loc")
            .short("L")
            .help("Display location name"))
        .arg(Arg::with_name("text")
            .short("T")
            .help("Display the first line text description of this artifact. \
                   Print the full description with -l, otherwise the first line."))
        .arg(Arg::with_name("plain")
            .long("plain")
            .help("Do not display color in the output"))
        .arg(Arg::with_name("type")
            .long("type")
            .value_name("TYPE")
            .takes_value(true)
            .help("Output type, default 'list'. Supported types: list, json"))
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
            return Err("percent must be of the form: [SIGN]NUM where NUM is between 0 and 100 and \
                        SIGN is an optional < or >"
                               .to_string())
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
        Ok(v) => {
            if v <= 100 && v >= -100 {
                Ok((lt, Some(v)))
            } else {
                Err("NUM must be between -100 and 100".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn get_percent(s: &str) -> result::Result<PercentSearch, String> {
    Ok(match _get_percent(s) {
           Ok((lt, perc)) => {
               if lt.is_none() && perc.is_none() {
                   PercentSearch {
                       lt: false,
                       perc: 100,
                   }
               } else if perc.is_none() {
        if lt.unwrap() {
            PercentSearch {
                lt: true,
                perc: 0,
            }
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
        PercentSearch {
            lt: lt,
            perc: perc,
        }
    }
           }
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
    assert_eq!(get_percent(""),
               Ok(PercentSearch {
                      lt: false,
                      perc: 100,
                  }));
    assert_eq!(get_percent("<"),
               Ok(PercentSearch {
                      lt: true,
                      perc: 0,
                  }));
    assert_eq!(get_percent(">"),
               Ok(PercentSearch {
                      lt: false,
                      perc: 100,
                  }));
    assert_eq!(get_percent("89"),
               Ok(PercentSearch {
                      lt: false,
                      perc: 89,
                  }));
    assert_eq!(get_percent(">89"),
               Ok(PercentSearch {
                      lt: false,
                      perc: 89,
                  }));
    assert_eq!(get_percent("<89"),
               Ok(PercentSearch {
                      lt: true,
                      perc: 89,
                  }));
    assert_eq!(get_percent(">-1"),
               Ok(PercentSearch {
                      lt: false,
                      perc: -1,
                  }));

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
    fmt_set.path = matches.is_present("path");
    fmt_set.parts = matches.is_present("parts");
    fmt_set.partof = matches.is_present("partof");
    fmt_set.loc_path = matches.is_present("loc");
    fmt_set.text = matches.is_present("text");
    fmt_set.color = get_color(matches);
    // #SPC-cmd-ls-display
    if matches.is_present("all") {
        // reverse everything
        fmt_set.path = !fmt_set.path;
        fmt_set.parts = !fmt_set.parts;
        fmt_set.partof = !fmt_set.partof;
        fmt_set.loc_path = !fmt_set.loc_path;
        fmt_set.text = !fmt_set.text;
    } else if fmt_set.long &&
              !(fmt_set.path || fmt_set.parts || fmt_set.partof || fmt_set.loc_path ||
                fmt_set.text) {
        // if long is specified but no other display attributes are specified
        fmt_set.path = true;
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
            let msg = format!("invalid type: {}", t);
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
pub fn run_cmd<W: Write>(mut w: &mut W, cwd: &Path, cmd: &Cmd, project: &Project) -> Result<u8> {
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
            let p = RegexBuilder::new(&cmd.pattern).case_insensitive(true).build();
            match p {
                Ok(p) => p,
                Err(e) => {
                    return Err(ErrorKind::CmdError(format!("Invalid pattern: {}", e)).into());
                }
            }
        } else {
            Regex::new("").unwrap()
        };
        names.drain(0..)
            .filter(|n| {
                        let a = artifacts.get(n).unwrap(); // we are guaranteed the name exists
                        ui::show_artifact(n, a, &pat, &cmd.search_settings)
                    })
            .collect()
    };
    debug!("artifact names selected: {:?}", names);
    if fmt_set.is_empty() {
        fmt_set.parts = true;
    }

    let mut displayed = Names::new();
    // #SPC-cmd-ls-type
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
            w.write_all(export::project_artifacts_to_json(project, Some(&names)).as_bytes())?;
        }
    }
    if !dne.is_empty() {
        return Err(ErrorKind::NameNotFound(format!("The following artifacts do not exist: {:?}",
                                                   dne))
                           .into());
    }
    Ok(0)
}
