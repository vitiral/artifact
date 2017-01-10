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

use dev_prefix::*;
use super::types::*;
use super::init;
use super::tutorial;
use super::ls;
use super::check;
use super::server;

pub fn get_matches<'a, I, T>(args: I) -> ClapResult<ArgMatches<'a>>
    where I: IntoIterator<Item=T>, 
          T: Into<OsString> + clone::Clone
{
    App::new("rst")
        .version(env!("CARGO_PKG_VERSION"))
        .about("the requirements tracking tool made for developers. Call `rst init -t` for \
                a tutorial")
        .author("https://github.com/vitiral/rst")
        .settings(&[AS::SubcommandRequiredElseHelp, AS::VersionlessSubcommands,
                    AS::DeriveDisplayOrder, COLOR])
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
        .arg(Arg::with_name("work-tree")
             .long("work-tree")
             .help("use a different working tree instead of cwd")
             .takes_value(true)
             .global(true))
        .subcommand(tutorial::get_subcommand())
        .subcommand(init::get_subcommand())
        .subcommand(ls::get_subcommand())
        .subcommand(check::get_subcommand())
        .subcommand(server::get_subcommand())
        .get_matches_from_safe(args)
}
