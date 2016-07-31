use std::ffi::OsString;

use clap::{Arg, App, SubCommand, ArgMatches, AppSettings as AS, Result as ClapResult};

use super::ls;
use super::init;

pub fn get_matches<'a, I, T>(args: I) -> ClapResult<ArgMatches<'a>>
    where I: IntoIterator<Item=T>, T: Into<OsString> {
    // [#SPC-ui-cmdline-cmd-help]
    App::new("rsk")
        .version("0.0.1")
        .about("the requirements tracking tool made for developers. Call `rsk init -t` for \
                a tutorial")
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
        .subcommand(init::get_subcommand())
        .subcommand(ls::get_subcommand())
        .get_matches_from_safe(args)
}
