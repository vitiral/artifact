/*  rst: the requirements tracking tool made for developers
    Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the Lesser GNU General Public License as published 
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the Lesser GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use super::types::*;
use super::fmt as cmdfmt;

/// Get the ls subcommand, which is what creates the command
/// for the cmdline
/// partof: #SPC-ls-args, #SPC-ls-display, #SPC-ls-pattern
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("ls")
        .about("list artifacts according to various parameters")
        .settings(&[AS::DeriveDisplayOrder, COLOR])
        .arg(Arg::with_name("search")
                 .help("artifact names given in form `REQ-foo-[bar, baz-[1,2]]` OR pearl regexp \
                        pattern if -p is given")
                 .use_delimiter(false))
        .arg(Arg::with_name("pattern")
                 .short("p")
                 .help("search FIELDS using pearl regexp SEARCH.")
                 .value_name("FIELDS")
                 .takes_value(true)
                 .max_values(1)
                 .min_values(0))
        .arg(Arg::with_name("long")
                 .short("l")
                 .help("print items in the 'long form'"))
        .arg(Arg::with_name("completed")
                 .short("c")
                 .help("filter by completeness (ie `<45`), < and > are inclusive, '>' == `>100`")
                 .takes_value(true))
        .arg(Arg::with_name("tested")
                 .short("t")
                 .help("give a filter for the testedness in %. see '-c'")
                 .takes_value(true))
        .arg(Arg::with_name("all")
                 .short("A")
                 .help("If set, additional flags will be *deactivated* instead of activated"))
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
        .arg(Arg::with_name("text")
                 .short("T")
                 .help("display the text description of this artifact (first line only if not -l)"))
        .arg(Arg::with_name("plain")
                 .long("plain")
                 .help("do not display color in the output"))

}

/// return (lt, percent) returning None when there is no value
pub fn _get_percent(s: &str) -> Result<(Option<bool>, Option<u8>), String> {
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
    match s.parse::<u8>() {
        Ok(v) => {
            if v <= 100 {
                Ok((lt, Some(v)))
            } else {
                Err("NUM must be between 0 and 100".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn get_percent(s: &str) -> Result<PercentSearch, String> {
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

    // invalid
    assert!(get_percent(">101").is_err());
    assert!(get_percent(">-1").is_err());
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

/// get all the information from the user input
pub fn get_ls_cmd(matches: &ArgMatches) -> Result<(String, FmtSettings, SearchSettings), String> {
    let mut fmt_set = FmtSettings::default();
    fmt_set.long = matches.is_present("long");
    // fmt_set.recurse = matches.value_of("recursive").unwrap().parse::<u8>().unwrap();
    fmt_set.path = matches.is_present("path");
    fmt_set.parts = matches.is_present("parts");
    fmt_set.partof = matches.is_present("partof");
    fmt_set.loc_path = matches.is_present("loc");
    fmt_set.text = matches.is_present("text");
    fmt_set.color = get_color(matches);
    if matches.is_present("all") {
        // reverse everything
        fmt_set.path = !fmt_set.path;
        fmt_set.parts = !fmt_set.parts;
        fmt_set.partof = !fmt_set.partof;
        fmt_set.loc_path = !fmt_set.loc_path;
        fmt_set.text = !fmt_set.text;
    } else if fmt_set.long &&
       !(fmt_set.path || fmt_set.parts || fmt_set.partof || fmt_set.loc_path || fmt_set.text) {
        // if long is specified but no other display attributes are specified
        fmt_set.path = true;
        fmt_set.parts = true;
        fmt_set.partof = true;
        fmt_set.loc_path = true;
        fmt_set.text = true;
    }

    // #SPC-ls-search
    let mut search_set = match (matches.is_present("pattern"), matches.value_of("pattern")) {
        (true, Some(p)) => try!(SearchSettings::from_str(p)),
        (true, None) => SearchSettings::from_str("N").unwrap(),
        (false, None) => SearchSettings::new(),
        _ => unreachable!(),
    };
    debug!("tested: {:?}", search_set.tested);
    if let Some(c) = matches.value_of("completed") {
        search_set.completed = try!(get_percent(c))
    }
    if let Some(t) = matches.value_of("tested") {
        debug!("got tested: {}", t);
        search_set.tested = try!(get_percent(t));
    }
    debug!("tested: {:?}", search_set.tested);

    let search = matches.value_of("search").unwrap_or("").to_string();

    debug!("ls search: {}, fmt_set: {:?}, search_set: {:?}",
           search,
           fmt_set,
           search_set);
    Ok((search, fmt_set, search_set))
}

#[allow(trivial_regex)]
/// perform the ls command given the inputs
pub fn do_ls<W: Write>(w: &mut W,
                       cwd: &Path,
                       search: &str,
                       fmt_set: &FmtSettings,
                       search_set: &SearchSettings,
                       project: &Project) -> i32 {
    let mut dne: Vec<ArtNameRc> = Vec::new();
    let mut names: Vec<ArtNameRc> = Vec::new();
    let artifacts = &project.artifacts;
    let mut settings = project.settings.clone();
    let mut fmt_set = (*fmt_set).clone();

    // load settings from cmdline inputs
    settings.color = fmt_set.color;

    let pat_case;
    if search_set.use_regex {
        // names to use are determined by filtering the regex
        let pat = RegexBuilder::new(search)
                      .case_insensitive(true)
                      .compile();
        pat_case = match pat {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid pattern: {}", e.to_string());
                return 1;
            }
        };
        names.extend(artifacts.keys().cloned());
        names.sort();
    } else {
        // names to use are determined from the beginning
        names.extend(match ArtNames::from_str(search) {
            Ok(n) => n,
            Err(e) => {
                error!("{}", e);
                return 1;
            }
        });
        names.sort();
        debug!("artifact names selected: {:?}", names);
        pat_case = Regex::new("").unwrap();
    }
    debug!("fmt_set empty: {}", fmt_set.is_empty());
    if names.is_empty() && search.is_empty() {
        names.extend(artifacts.keys().cloned());
        names.sort();
    }
    if fmt_set.is_empty() {
        fmt_set.parts = true;
        fmt_set.path = true;
    }


    if !fmt_set.long {
        cmdfmt::write_table_header(w, &fmt_set, &settings);
    }
    let mut displayed = ArtNames::new();
    for name in names {
        let art = match artifacts.get(&name) {
            Some(a) => a,
            None => {
                trace!("Name DNE: {}", name);
                dne.push(name);
                continue;
            }
        };
        if !ui::show_artifact(&name, art, &pat_case, search_set) {
            continue;
        }
        let f = ui::fmt_artifact(&name, artifacts, &fmt_set, fmt_set.recurse, &mut displayed);
        f.write(w, cwd, artifacts, &settings, 0).unwrap(); // FIXME: unwrap
    }
    if !dne.is_empty() {
        error!("The following artifacts do not exist: {:?}", dne);
        return 1;
    }
    0
}
