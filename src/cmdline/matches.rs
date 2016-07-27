use std::ffi::OsString;

use clap::{Arg, App, SubCommand, ArgMatches, AppSettings as AS, Result as ClapResult};

use super::ls;

pub fn get_matches<'a, I, T>(args: I) -> ClapResult<ArgMatches<'a>>
    where I: IntoIterator<Item=T>, T: Into<OsString> {
    App::new("rsk")
        .version("0.0.1")
        .about("the requirements tracking tool made for developers")
        .settings(&[AS::SubcommandRequiredElseHelp, AS::VersionlessSubcommands,
                    AS::DeriveDisplayOrder, AS::ColoredHelp])
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
        .subcommand(ls::get_subcommand())
        .get_matches_from_safe(args)
}
